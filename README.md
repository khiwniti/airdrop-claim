# airdrop-claim

Merkle-gated SPL token airdrop distributor — Colosseum Crypto World's Fair entry.
Submission deadline: **2026-10-12** (see `../SPRINT.md`, Sprint 3.1).

## Design
- `initialize(id, merkle_root, deposit_amount)` — authority creates a campaign PDA
  `[b"campaign", authority, id_le]` and funds its PDA-vault from an ATA.
- `claim(amount, proof)` — claimant signs; leaf = `sha256(claimant || amount_le)`;
  sorted-pair merkle verification against `campaign.merkle_root`; payout from the
  vault signed by the campaign PDA. A `ClaimReceipt` PDA `[b"claim", campaign, claimant]`
  is `init`ed in the same instruction → a second claim always fails (no bitmap).
- `close_campaign()` — authority (enforced via `has_one` + seeds) sweeps the unclaimed
  remainder; `close = authority` refunds campaign rent. Receipts remain for audit.

Off-chain: build the tree with `merkle::{leaf_hash, build_tree}` (same code the program
runs — single source of truth) or via the CSV tool:

```bash
cargo run --example build_tree -- allocations.csv proofs.json
# CSV: `address,amount` in base units → prints root (base58), writes per-claimant proofs.
```

## Build & test
```bash
anchor build        # SBF compile + IDL
cargo test          # litesvm integration suite (no validator needed)
```

### Test coverage
| Test | Asserts |
|------|---------|
| happy_path_claim_updates_state | payout, vault debit, receipt + counter updated |
| double_claim_is_rejected | 2nd claim fails, balance unchanged |
| forged_amount_fails_merkle_check | wrong amount → `InvalidMerkleProof` |
| third_leaf_proof_and_close_sweeps_remainder | odd-leaf duplicate-last proof; remainder swept; account closed |

## Claim portal (app/)
Static page (`python3 -m http.server 8080` from `app/`) — connects Phantom, loads a
campaign, reads a `proofs.json` (from `build_tree`), and submits `claim`. web3.js is
vendored at `app/vendor/web3.js` so the page works without a bundler or CDN.

## Deployment
**Program is live on Solana devnet**: `2NepmY26ip3y5gPWaY5ZoAUUWNtqqMjDobiAyFtCj5Jf`
(IDL on-chain; [explorer](https://explorer.solana.com/address/2NepmY26ip3y5gPWaY5ZoAUUWNtqqMjDobiAyFtCj5Jf?cluster=devnet)).
Claim portal live at https://khiwniti.github.io/airdrop-claim/ (default RPC: devnet).
Judge/demo campaign (view state; allocation is the demo wallet's): `GwyoYCDCUusLByLBjHpe1c9ABVZWV3GW6MTpUnkbtnEK`.
Devnet lifecycle signatures: init `qRmsp54W…`, claim `5LFaGeWU…`, close `4FJPkbUD…`,
portal-claim `3qhWKrzB…` (all finalized on devnet).

End-to-end lifecycle driver (`examples/e2e_devnet.rs`), defaults to public devnet;

```bash
solana-test-validator --reset --quiet                       # local, unlimited faucet
anchor deploy                                               # localnet per Anchor.toml
SOLANA_RPC=http://127.0.0.1:8899 cargo run --example e2e_devnet
# or against public devnet once the faucet funds ~/.config/solana/id.json:
SOLANA_RPC=https://api.devnet.solana.com cargo run --example e2e_devnet
```

Verified on local validator (2026-10-04): mint+fund, initialize, claim (self-claim,
receipt written), close with empty-vault sweep — all assertions green.
