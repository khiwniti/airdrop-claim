# Pitch script (≤2 min, on camera)

> 0:00–0:20 — Hook
"Every Solana airdrop season, the same thing happens: thousands of users rush to claim
tokens, and the claim page itself is the attack surface. This June, a fake Jupiter claim
site drained wallets. Meanwhile every team — Jupiter, Kamino, Meteora — hand-rolls the
same token distributor from scratch. We built the boring primitive so nobody has to."

> 0:20–1:00 — What it is
"airdrop-claim is a permissionless merkle-gated airdrop distributor on Solana. A team
creates a campaign in one transaction: funds go into a vault owned by a PDA. Claimants
present a merkle proof of sha256(claimant, amount) and pay themselves out — signed by
their own wallet, verified on-chain. A claim receipt PDA makes double-claiming
structurally impossible, not just checked. Leftovers are recoverable; receipts are a
free on-chain audit trail."

> 1:00–1:30 — Proof of work (scroll the demo while speaking)
"The whole thing is verified by execution: five integration tests over an in-process
validator; a live end-to-end run that caught a real bug before shipping; and a headless
browser smoke test that caught two more — including a typo'd program ID that would have
failed every claim. The merkle tree builder and the on-chain verifier are literally the
same Rust module, so what you generate off-chain is what the chain checks."

> 1:30–1:50 — Why us
"I directed the product; an AI coding agent implemented it under my direction, and every
claim about what works is backed by a transaction signature, not a slide. That rigor is
the product's entire pitch: claim safety as a protocol property."

> 1:50–2:00 — Ask
"Next: devnet demo campaign, wallet-adapter UI polish, Token-2022 support. Funded or not,
this ships — the code is public and the demo script runs in three commands."
