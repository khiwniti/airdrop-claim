//! Live devnet dry-run of the whole campaign lifecycle:
//! mint → fund authority ATA → initialize(root, deposit) → claim(proof) → close.
//! Requires the program deployed to devnet and the wallet in
//! ~/.config/solana/id.json funded with a little SOL.
//!
//! Usage: cargo run --example e2e_devnet

use anchor_lang::{
    prelude::Pubkey,
    solana_program::{instruction::Instruction, program_pack::Pack, system_instruction, system_program},
    InstructionData, ToAccountMetas,
};
use airdrop_claim::{
    constants::CAMPAIGN_SEED,
    merkle::{build_tree, leaf_hash},
};
use solana_keypair::Keypair;
use solana_rpc_client::rpc_client::RpcClient;
use solana_sdk::{signature::read_keypair_file, signer::Signer, transaction::Transaction};

// Campaign id: unix seconds so re-runs never collide with a previous campaign.
fn campaign_id() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs()
}

const ALLOCATION: u64 = 300; // payer claims its own allocation: self-claim path

fn send(rpc: &RpcClient, ixs: &[Instruction], payer: &Keypair, extra: &[&Keypair]) -> String {
    let bh = rpc.get_latest_blockhash().unwrap();
    let mut signers: Vec<&Keypair> = vec![payer];
    signers.extend_from_slice(extra);
    let tx = Transaction::new_signed_with_payer(ixs, Some(&payer.pubkey()), &signers, bh);
    rpc.send_and_confirm_transaction(&tx).unwrap().to_string()
}

fn token_balance(rpc: &RpcClient, account: &Pubkey) -> u64 {
    rpc.get_token_account_balance(account)
        .unwrap()
        .amount
        .parse()
        .unwrap()
}

fn main() {
    let url = std::env::var("SOLANA_RPC").unwrap_or_else(|_| "https://api.devnet.solana.com".into());
    if let Err(e) = run(url) {
        eprintln!("e2e failed: {e:?}");
        std::process::exit(1);
    }
}

fn run(url: String) -> Result<(), Box<dyn std::error::Error>> {
    let rpc = RpcClient::new(url);
    let payer = read_keypair_file(std::env::home_dir().unwrap().join(".config/solana/id.json"))
        .expect("fund ~/.config/solana/id.json on devnet first");
    println!("payer   : {}", payer.pubkey());
    println!("balance : {} lamports", rpc.get_balance(&payer.pubkey()).unwrap());

    let pid = airdrop_claim::id();
    let id = campaign_id();
    let mint_kp = Keypair::new();
    let mint = mint_kp.pubkey();
    let authority_ata =
        spl_associated_token_account::get_associated_token_address(&payer.pubkey(), &mint);
    let (campaign, _) = Pubkey::find_program_address(
        &[CAMPAIGN_SEED, payer.pubkey().as_ref(), id.to_le_bytes().as_ref()],
        &pid,
    );
    let vault_kp = Keypair::new();

    // merkle tree from a 1-row allocation (payer claims its own 300).
    let leaves = [leaf_hash(&payer.pubkey(), ALLOCATION)];
    let (root, proofs) = build_tree(&leaves).unwrap();

    let mint_rent = rpc
        .get_minimum_balance_for_rent_exemption(spl_token::state::Mint::LEN)
        .unwrap();
    let sig = send(
        &rpc,
        &[
            system_instruction::create_account(
                &payer.pubkey(),
                &mint,
                mint_rent,
                spl_token::state::Mint::LEN as u64,
                &spl_token::ID,
            ),
            spl_token::instruction::initialize_mint2(
                &spl_token::ID,
                &mint,
                &payer.pubkey(),
                None,
                6,
            )
            .unwrap(),
            spl_associated_token_account::instruction::create_associated_token_account(
                &payer.pubkey(),
                &payer.pubkey(),
                &mint,
                &spl_token::ID,
            ),
            spl_token::instruction::mint_to(
                &spl_token::ID,
                &mint,
                &authority_ata,
                &payer.pubkey(),
                &[],
                ALLOCATION,
            )
            .unwrap(),
        ],
        &payer,
        &[&mint_kp],
    );
    println!("mint+fund: {sig}  (mint {mint})");

    let init_ix = Instruction::new_with_bytes(
        pid,
        &airdrop_claim::instruction::Initialize {
            id,
            merkle_root: root,
            deposit_amount: ALLOCATION,
        }
        .data(),
        airdrop_claim::accounts::Initialize {
            authority: payer.pubkey(),
            campaign,
            mint,
            vault: vault_kp.pubkey(),
            authority_ata,
            token_program: anchor_spl::token::ID,
            associated_token_program: anchor_spl::associated_token::ID,
            system_program: system_program::ID,
        }
        .to_account_metas(None),
    );
    let sig = send(&rpc, &[init_ix], &payer, &[&vault_kp]);
    println!("init     : {sig}  (campaign {campaign}, vault {})", vault_kp.pubkey());
    assert_eq!(token_balance(&rpc, &vault_kp.pubkey()), ALLOCATION);

    let claimant_ata =
        spl_associated_token_account::get_associated_token_address(&payer.pubkey(), &mint);
    let (receipt, _) = Pubkey::find_program_address(
        &[b"claim", campaign.as_ref(), payer.pubkey().as_ref()],
        &pid,
    );
    let claim_ix = Instruction::new_with_bytes(
        pid,
        &airdrop_claim::instruction::Claim {
            amount: ALLOCATION,
            proof: proofs[0].clone(),
        }
        .data(),
        airdrop_claim::accounts::Claim {
            claimant: payer.pubkey(),
            campaign,
            mint,
            vault: vault_kp.pubkey(),
            claimant_ata,
            receipt,
            token_program: anchor_spl::token::ID,
            associated_token_program: anchor_spl::associated_token::ID,
            system_program: system_program::ID,
        }
        .to_account_metas(None),
    );
    if std::env::var_os("SKIP_CLAIM").is_some() {
        // UI owns the claim step; campaign stays funded and unclaimed.
        println!("claim    : SKIPPED (SKIP_CLAIM) — claimant {} amount {ALLOCATION}", payer.pubkey());
    } else {
        let sig = send(&rpc, &[claim_ix], &payer, &[]);
        println!("claim    : {sig}");
        assert_eq!(token_balance(&rpc, &claimant_ata), ALLOCATION);
        assert_eq!(token_balance(&rpc, &vault_kp.pubkey()), 0);
    }

    if std::env::var_os("KEEP_OPEN").is_some() {
        println!("\nKEEP_OPEN set — leaving campaign {campaign} on-chain. root: {root:?}");
        return Ok(());
    }

    let close_ix = Instruction::new_with_bytes(
        pid,
        &airdrop_claim::instruction::CloseCampaign {}.data(),
        airdrop_claim::accounts::CloseCampaign {
            authority: payer.pubkey(),
            campaign,
            mint,
            vault: vault_kp.pubkey(),
            authority_ata,
            token_program: anchor_spl::token::ID,
            associated_token_program: anchor_spl::associated_token::ID,
            system_program: system_program::ID,
        }
        .to_account_metas(None),
    );
    let sig = send(&rpc, &[close_ix], &payer, &[]);
    println!("close    : {sig}");
    assert_eq!(token_balance(&rpc, &authority_ata), ALLOCATION);
    assert!(rpc.get_account(&campaign).is_err(), "campaign must be closed");

    println!("\n✓ live devnet path verified: initialize → claim → close");
    Ok(())
}
