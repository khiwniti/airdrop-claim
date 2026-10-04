use {
    anchor_lang::{
        prelude::Pubkey,
        solana_program::{
            instruction::Instruction,
            program_pack::Pack,
            system_instruction, system_program,
        },
        AccountDeserialize, InstructionData, ToAccountMetas,
    },
    airdrop_claim::constants::{CAMPAIGN_SEED, CLAIM_SEED},
    anchor_spl,
    litesvm::LiteSVM,
    solana_keypair::Keypair,
    solana_message::{Message, VersionedMessage},
    solana_signer::Signer,
    solana_transaction::versioned::VersionedTransaction,
};

const PROGRAM_BYTES: &[u8] = include_bytes!(concat!(
    env!("CARGO_TARGET_TMPDIR"),
    "/../deploy/airdrop_claim.so"
));

const CAMPAIGN_ID: u64 = 7;
// Allocations: alice 300, bob 400, carol 500 (unclaimed), deposit 1200.
const ALLOCATIONS: &[(u8, u64)] = &[(0, 300), (1, 400), (2, 500)];
const DEPOSIT: u64 = 1200;
const MINT_DECIMALS: u8 = 6;

// ---------- merkle helpers (same code the program runs) ----------

use airdrop_claim::merkle::{build_tree, leaf_hash};

fn merkle_root(leaves: &[[u8; 32]]) -> [u8; 32] {
    build_tree(leaves).unwrap().0
}

fn merkle_proof(leaves: &[[u8; 32]], idx: usize) -> Vec<[u8; 32]> {
    build_tree(leaves).unwrap().1[idx].clone()
}

// ---------- harness ----------

struct Fixture {
    svm: LiteSVM,
    authority: Keypair,
    claimants: Vec<Keypair>,
    mint: Pubkey,
    authority_ata: Pubkey,
    campaign: Pubkey,
    vault: Pubkey,
    leaves: Vec<[u8; 32]>,
}

fn program_id() -> Pubkey {
    airdrop_claim::id()
}

fn campaign_pda(authority: &Pubkey) -> (Pubkey, u8) {
    Pubkey::find_program_address(
        &[
            CAMPAIGN_SEED,
            authority.as_ref(),
            CAMPAIGN_ID.to_le_bytes().as_ref(),
        ],
        &program_id(),
    )
}

fn receipt_pda(campaign: &Pubkey, claimant: &Pubkey) -> (Pubkey, u8) {
    Pubkey::find_program_address(
        &[CLAIM_SEED, campaign.as_ref(), claimant.as_ref()],
        &program_id(),
    )
}

fn send(svm: &mut LiteSVM, ixs: &[Instruction], payer: &Keypair, signers: &[&Keypair]) {
    let blockhash = svm.latest_blockhash();
    let msg = Message::new_with_blockhash(ixs, Some(&payer.pubkey()), &blockhash);
    let mut all: Vec<&Keypair> = vec![payer];
    all.extend_from_slice(signers);
    let tx = VersionedTransaction::try_new(VersionedMessage::Legacy(msg), &all).unwrap();
    svm.send_transaction(tx).unwrap();
}

fn token_balance(svm: &LiteSVM, account: &Pubkey) -> u64 {
    let data = svm.get_account(account).unwrap().data;
    spl_token::state::Account::unpack(&data).unwrap().amount
}

fn setup() -> Fixture {
    let mut svm = LiteSVM::new().with_default_programs();
    svm.add_program(program_id(), PROGRAM_BYTES).unwrap();

    let authority = Keypair::new();
    let claimants: Vec<Keypair> = (0..ALLOCATIONS.len()).map(|_| Keypair::new()).collect();
    svm.airdrop(&authority.pubkey(), 5_000_000_000).unwrap();
    for c in &claimants {
        svm.airdrop(&c.pubkey(), 500_000_000).unwrap();
    }

    // SPL mint, authority = `authority`.
    let mint_kp = Keypair::new();
    let mint = mint_kp.pubkey();
    let mint_rent = svm.minimum_balance_for_rent_exemption(spl_token::state::Mint::LEN);
    send(
        &mut svm,
        &[
            system_instruction::create_account(
                &authority.pubkey(),
                &mint,
                mint_rent,
                spl_token::state::Mint::LEN as u64,
                &spl_token::ID,
            ),
            spl_token::instruction::initialize_mint2(
                &spl_token::ID,
                &mint,
                &authority.pubkey(),
                None,
                MINT_DECIMALS,
            )
            .unwrap(),
        ],
        &authority,
        &[&mint_kp],
    );

    // Authority ATA funded with the full deposit.
    let authority_ata =
        spl_associated_token_account::get_associated_token_address(&authority.pubkey(), &mint);
    send(
        &mut svm,
        &[
            spl_associated_token_account::instruction::create_associated_token_account(
                &authority.pubkey(),
                &authority.pubkey(),
                &mint,
                &spl_token::ID,
            ),
            spl_token::instruction::mint_to(
                &spl_token::ID,
                &mint,
                &authority_ata,
                &authority.pubkey(),
                &[],
                DEPOSIT,
            )
            .unwrap(),
        ],
        &authority,
        &[],
    );

    let (campaign, _) = campaign_pda(&authority.pubkey());
    let leaves: Vec<[u8; 32]> = ALLOCATIONS
        .iter()
        .zip(&claimants)
        .map(|(&(_, amount), c)| leaf_hash(&c.pubkey(), amount))
        .collect();
    let root = merkle_root(&leaves);

    // Fresh keypair: the program's `init` allocates it as the vault token account.
    let vault_kp = Keypair::new();
    let vault = vault_kp.pubkey();

    let init_ix = Instruction::new_with_bytes(
        program_id(),
        &airdrop_claim::instruction::Initialize {
            id: CAMPAIGN_ID,
            merkle_root: root,
            deposit_amount: DEPOSIT,
        }
        .data(),
        airdrop_claim::accounts::Initialize {
            authority: authority.pubkey(),
            campaign,
            mint,
            vault,
            authority_ata,
            token_program: anchor_spl::token::ID,
            associated_token_program: anchor_spl::associated_token::ID,
            system_program: system_program::ID,
        }
        .to_account_metas(None),
    );
    // vault token account needs creation: program does `init` with payer=authority.
    send(&mut svm, &[init_ix], &authority, &[&vault_kp]);

    Fixture {
        svm,
        authority,
        claimants,
        mint,
        authority_ata,
        campaign,
        vault,
        leaves,
    }
}

fn claim_ix(fx: &Fixture, idx: usize, amount: u64, proof: Vec<[u8; 32]>) -> Instruction {
    let claimant = &fx.claimants[idx];
    let claimant_ata =
        spl_associated_token_account::get_associated_token_address(&claimant.pubkey(), &fx.mint);
    let (receipt, _) = receipt_pda(&fx.campaign, &claimant.pubkey());
    Instruction::new_with_bytes(
        program_id(),
        &airdrop_claim::instruction::Claim { amount, proof }.data(),
        airdrop_claim::accounts::Claim {
            claimant: claimant.pubkey(),
            campaign: fx.campaign,
            mint: fx.mint,
            vault: fx.vault,
            claimant_ata,
            receipt,
            token_program: anchor_spl::token::ID,
            associated_token_program: anchor_spl::associated_token::ID,
            system_program: system_program::ID,
        }
        .to_account_metas(None),
    )
}

// ---------- tests ----------

/// Every off-chain-built proof must verify against the on-chain verifier.
#[test]
fn all_built_proofs_verify_against_merkle_logic() {
    let claimants: Vec<Pubkey> = (0..7).map(|_| Pubkey::new_unique()).collect();
    let amounts: Vec<u64> = vec![1, 300, 400, 500, 10_000, u64::MAX / 2, 7];
    let leaves: Vec<[u8; 32]> = claimants
        .iter()
        .zip(&amounts)
        .map(|(p, a)| leaf_hash(p, *a))
        .collect();
    let (root, proofs) = build_tree(&leaves).unwrap();
    assert_eq!(proofs.len(), leaves.len());
    for (i, (p, a)) in claimants.iter().zip(&amounts).enumerate() {
        assert!(
            airdrop_claim::merkle::verify_proof(&root, p, *a, &proofs[i]),
            "proof {i} must verify against the on-chain verifier"
        );
        let mut bad = proofs[i].clone();
        bad[0] = [0xAB; 32];
        assert!(!airdrop_claim::merkle::verify_proof(&root, p, *a, &bad));
    }
    assert!(build_tree(&[]).is_none());
}

#[test]
fn happy_path_claim_updates_state() {
    let mut fx = setup();
    assert_eq!(token_balance(&fx.svm, &fx.vault), DEPOSIT);

    let alice = 0usize;
    let ix = claim_ix(&fx, alice, ALLOCATIONS[alice].1, merkle_proof(&fx.leaves, alice));
    let payer = &fx.claimants[alice];
    send(&mut fx.svm, &[ix], payer, &[]);

    let ata =
        spl_associated_token_account::get_associated_token_address(&fx.claimants[alice].pubkey(), &fx.mint);
    assert_eq!(token_balance(&fx.svm, &ata), 300);
    assert_eq!(token_balance(&fx.svm, &fx.vault), DEPOSIT - 300);

    let (receipt_key, _) = receipt_pda(&fx.campaign, &fx.claimants[alice].pubkey());
    let data = fx.svm.get_account(&receipt_key).unwrap().data;
    let mut slice: &[u8] = &data;
    let receipt = airdrop_claim::state::ClaimReceipt::try_deserialize(&mut slice).unwrap();
    assert_eq!(receipt.amount, 300);
    assert_eq!(receipt.campaign, fx.campaign);

    let cdata = fx.svm.get_account(&fx.campaign).unwrap().data;
    let mut slice: &[u8] = &cdata;
    let campaign = airdrop_claim::state::Campaign::try_deserialize(&mut slice).unwrap();
    assert_eq!(campaign.total_claimed, 300);
}

#[test]
fn double_claim_is_rejected() {
    let mut fx = setup();
    let ix = claim_ix(&fx, 0, 300, merkle_proof(&fx.leaves, 0));
    send(&mut fx.svm, std::slice::from_ref(&ix), &fx.claimants[0], &[]);

    let ix2 = claim_ix(&fx, 0, 300, merkle_proof(&fx.leaves, 0));
    let payer = &fx.claimants[0];
    let blockhash = fx.svm.latest_blockhash();
    let msg = Message::new_with_blockhash(&[ix2], Some(&payer.pubkey()), &blockhash);
    let tx = VersionedTransaction::try_new(VersionedMessage::Legacy(msg), &[payer]).unwrap();
    assert!(fx.svm.send_transaction(tx).is_err());
    // balance unchanged after rejection
    let ata = spl_associated_token_account::get_associated_token_address(
        &fx.claimants[0].pubkey(),
        &fx.mint,
    );
    assert_eq!(token_balance(&fx.svm, &ata), 300);
}

#[test]
fn forged_amount_fails_merkle_check() {
    let mut fx = setup();
    // alice pretends her allocation is 301 — leaf no longer matches the tree.
    let ix = claim_ix(&fx, 0, 301, merkle_proof(&fx.leaves, 0));
    let payer = &fx.claimants[0];
    let blockhash = fx.svm.latest_blockhash();
    let msg = Message::new_with_blockhash(&[ix], Some(&payer.pubkey()), &blockhash);
    let tx = VersionedTransaction::try_new(VersionedMessage::Legacy(msg), &[payer]).unwrap();
    let err = fx.svm.send_transaction(tx);
    assert!(err.is_err());
    assert!(format!("{:?}", err.unwrap_err()).contains("InvalidMerkleProof"));
}

#[test]
fn third_leaf_proof_and_close_sweeps_remainder() {
    let mut fx = setup();
    // bob (index 1, multi-level proof due to 3-leaf tree) claims 400.
    let ix = claim_ix(&fx, 1, 400, merkle_proof(&fx.leaves, 1));
    send(&mut fx.svm, &[ix], &fx.claimants[1], &[]);

    let before = token_balance(&fx.svm, &fx.authority_ata);
    let close_ix = Instruction::new_with_bytes(
        program_id(),
        &airdrop_claim::instruction::CloseCampaign {}.data(),
        airdrop_claim::accounts::CloseCampaign {
            authority: fx.authority.pubkey(),
            campaign: fx.campaign,
            mint: fx.mint,
            vault: fx.vault,
            authority_ata: fx.authority_ata,
            token_program: anchor_spl::token::ID,
            associated_token_program: anchor_spl::associated_token::ID,
            system_program: system_program::ID,
        }
        .to_account_metas(None),
    );
    send(&mut fx.svm, &[close_ix], &fx.authority, &[]);

    // remainder 1200 - 400 = 800 swept; campaign account closed
    assert_eq!(
        token_balance(&fx.svm, &fx.authority_ata),
        before + DEPOSIT - 400
    );
    assert!(fx.svm.get_account(&fx.campaign).is_none());
}
