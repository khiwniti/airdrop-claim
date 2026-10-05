# airdrop-claim — Colosseum Crypto World's Fair Submission
> Deadline: **2026-10-12** · track: Developer tooling / DeFi primitives
> Submission portal: https://colosseum.com/worldsfair

## One-liner
Permissionless merkle-gated SPL token airdrop distributor: campaigns are created and
funded in one transaction, claims are self-service (proof-verified, double-claim
impossible), and unclaimed tokens are recoverable — a boring, auditable primitive
every Solana airdrop season needs.

## Why it matters
- Every major Solana airdrop (Jupiter, Jito, Drift, Kamino, Meteora) redistributes
  tokens to thousands of wallets; teams hand-roll distributors each time.
- Claim tooling today is drainer-bait: fake claim sites are the #1 vector
  (fake-JUP campaign, 2026-06). A claimant-signed, root-verified claim flow
  removes the "connect wallet to unknown page" footgun by design.

## Design (what's on-chain)
| Instruction | Effect | Security |
|---|---|---|
| `initialize(id, merkle_root, deposit)` | campaign PDA + PDA-vault funded | seeds bind authority; vault authority is the PDA |
| `claim(amount, proof)` | PDA-signed payout to claimant ATA | leaf `sha256(claimant ‖ amount_le)`, sorted-pair merkle, claimant must sign, `ClaimReceipt` PDA `init` makes double-claim structurally impossible |
| `close_campaign` | authority sweeps remainder, rent refunded | `has_one` + seeds; receipts persist for audit |

Claim receipts double as an on-chain audit trail (who claimed what, when) at zero
extra cost — no bitmap, no indexer needed.

## Proof of work
- `anchor build` clean; **5/5 litesvm integration tests** (happy path, double-claim
  rejection, forged-amount → `InvalidMerkleProof`, odd-leaf duplicate-last proof,
  every built proof roundtrip-verified against the on-chain verifier).
- **Live validator e2e**: mint → initialize → claim → close, balance assertions at
  every step; run caught and fixed a real close-revert bug (zero-remainder sweep).
- **Browser claim-UI smoke**: real signed claim tx against the running validator;
  UI caught its own empty-proof and program-id bugs before shipping.
- **Devnet production run (2026-10-05)**: program deployed with IDL
  (`2NepmY26ip3y5gPWaY5ZoAUUWNtqqMjDobiAyFtCj5Jf`), full lifecycle finalized on
  public devnet (init `qRmsp54W…`, claim `5LFaGeWU…`, close `4FJPkbUD…`), plus a
  browser-portal devnet claim (`3qhWKrzB…`). Portal live at
  https://khiwniti.github.io/airdrop-claim/
- CSV tool (`examples/build_tree.rs`) generates root + per-claimant proofs from an
  allocation CSV using the *same* hashing code the program runs — single source of
  truth, zero drift between off-chain tree and on-chain verification.

## Roadmap if funded
1. ~~Public devnet deployment~~ **shipped 2026-10-05** — program, IDL, live portal
   (https://khiwniti.github.io/airdrop-claim/), demo campaign
   `GwyoYCDCUusLByLBjHpe1c9ABVZWV3GW6MTpUnkbtnEK`.
2. Wallet-adapter UI polish + multi-campaign explorer view.
3. Token-2022 support (transfer hooks, confidential transfers) — distribution
   primitive for the next generation of Solana tokens.
4. Merkle-tree refresh instruction (append-only campaigns without redeploy).

## Demo script (3 min)
1. `cargo test` — 5 tests pass in seconds, no validator needed.
2. `solana-test-validator` + `anchor deploy` + `cargo run --example e2e_devnet`
   — full lifecycle live with signatures printed.
3. Open `app/` portal — connect wallet, load campaign, claim from `proofs.json`,
   show `claimed=300/300` and the on-chain receipt.
