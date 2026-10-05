# Pitch Deck: airdrop-claim

Per pitch-deck-creator framework:
- **Purpose**: Colosseum Crypto World's Fair — investor-grade judges, accelerator slots, $840K prizes
- **Audience**: hackathon judges (operators + VCs)
- **Duration**: 3-minute pitch (maps to PITCH.md timing)
- **Key ask**: deploy the standard — prize + accelerator mentoring to drive ecosystem adoption

---

## Slide 1: Title
**Headline**: airdrop-claim — claim safety as a protocol property
**Visual**: `logo.png` — merkle tree draining into "CLAIM → WALLET"
**Speaker notes**: "This is airdrop-claim: a merkle-gated airdrop distributor where the claim page can no longer steal your wallet, because payout is verified on-chain and signed by you."

## Slide 2: Problem
**Headline**: The claim page is the #1 wallet-drainer vector on Solana
**Body**:
- Fake Jupiter claim campaign drained wallets (June 2026)
- Fake airdrop sites only work because claimants must trust unknown pages
- Meanwhile every team hand-rolls its own token distributor each season
**Visual**: Chainabuse airdrop-scam categories screenshot; fake-JUP headline
**Speaker notes**: "Airdrop claims chain two bad patterns together: users signing transactions on unknown sites, and teams rebuilding distribution plumbing every time. Both are solved problems on other chains — on Solana, not yet."

## Slide 3: Solution
**Headline**: Proof-verified, claimant-signed payouts — trust replaced by account math
**Body**:
- Team funds a campaign PDA vault in one transaction
- Leaf = sha256(claimant ‖ amount), sorted-pair merkle verified in-program
- Double-claim structurally impossible (receipt PDA `init`)
- Leftovers recoverable; receipts are a free on-chain audit trail
**Visual**: 3-step flow diagram — fund vault → proof → payout CPI
**Speaker notes**: "The claimant never approves anything. Their proof pins their address and amount; the program simulates the transfer itself from a PDA-owned vault. There is no 'unlimited approve' to phish."

## Slide 4: Market
**Headline**: Every Solana distribution season is this product's market
**Body**:
- Jupiter, Jito, Drift, Kamino, Meteora — multi-billion-dollar combined distributions
- 10,000+ builders shipped 2,857 hackathon projects last Colosseum round — each is a future team needing this
- Points programs inside the ecosystem still treat distribution as homework
**Visual**: bar chart of historic Solana airdrop sizes vs claim-scam losses
**Speaker notes**: "We don't pretend to size this as a market — it isn't one. It's an ecosystem primitive: anyone running a distribution needs it. The TAM slide for a primitive is the entire distribution budget of Solana."

## Slide 5: Product / Demo
**Headline**: Three instructions, all verified by execution, live demo runs in 3 commands
**Body**:
- `initialize(id, root, deposit)` → `claim(amount, proof)` → `close_campaign`
- CSV→tree CLI shares the exact same merkle code as the on-chain verifier
- 5/5 litesvm tests; live-validator e2e; browser claim smoke test — bugs caught live, not in prod
**Visual**: demo video `demo.mp4` frame or screen recording; test output `5 passed`
**Speaker notes**: "Our demo is 36 seconds: connect a wallet, load a campaign, claim 300 tokens, refresh and see 300/300 and the on-chain receipt. No mock server — the validator you see in the signature log is real."

## Slide 6: Why it's safe by construction
**Headline**: We moved security out of the instruction body and into the account model
**Body**:
- Voting hard: no "owner-only" function to rug — only claim and sweep
- Claim tx contains no mutability flags beyond vault/ATA/receipt — drainers can't hide calls
- Receipt PDA is one-per-claimant; replay attacks fail at account init
**Visual**: side-by-side "phishing claim page" vs "airdrop-claim flow" showing what's signed
**Speaker notes**: "The way you make a claim site not exploitable isn't better UI or warnings — it's a claim transaction that can't contain anything dangerous. That's what the account layout enforces."

## Slide 7: How we got here (credibility)
**Headline**: AI-assisted, human-directed, just like everything else built fast this cycle
**Body**:
- Directed scope and design; implementation paired with an AI coding agent
- All claims verified by execution — every bug in the deck was caught before it shipped
- Weeks of work that the old way would have taken a small team
**Visual**: `SUBMISSION.md` + `PITCH.md` thumbnails; commit graph
**Speaker notes**: "Transparency slide: an AI coding agent wrote this code under my direction. What matters to judges is whether it works — every one of these claims has a transaction signature or a test name behind it."

## Slide 8: Go-to-market
**Headline**: The claim portal is the wedge; protocol adoption is the prize
**Body**:
- Portal ships today as a static page — any existing campaign can use it
- Next: embeddable claim widget so distributors stop hosting their own claim pages
- Long-term: this becomes the default reference implementation for Solana airdrops
**Visual**: embed snippet mock + claim-portal screenshot
**Speaker notes**: "Distribution tooling lives and dies on trust. Our path is: portals trust us first, then protocols just reference us. The moment any DRainer relies on our page, that page has to be un-phishable — which is what the protocol enforces."

## Slide 9: Roadmap / Ask
**Headline**: Three concrete gaps, one funding ask
**Body**:
- Devnet demo campaign live the moment faucet funding clears — blocked today on rate limits
- Token-2022 + wallet-adapter UI polish
- Ask: hackathon prize → fund audit + devnet launch; accelerator mentoring for ecosystem adoption
**Visual**: SUBMISSION.md roadmap bullets as a checklist
**Speaker notes**: "I'm not asking you to fund a dream — I'm asking you to finish plumbing that already works end-to-end on a local validator, so it can run in public in front of the people who need it."

## Slide 10: Contact
**Headline**: Public, auditable, reproducible in under five minutes
**Body**:
- Repo: github.com/khiwniti/airdrop-claim (public; SUBMISSION.md = pitch, PITCH.md = script)
- `cargo test` → 5/5 pass — no validator needed, no excuses
**Visual**: repo README screenshot
**Speaker notes**: "If you have doubts, run it. `anchor build`, `cargo test`. If it doesn't do what I said, the repo is public — you can see exactly what changed."

---

## Quality checklist (framework self-audit)
- [x] Every headline is a takeaway, not a label ("The claim page is the #1 drainer", not "Problem")
- [x] One idea per slide
- [x] Body under 30 words per slide where tight
- [x] Visual suggested on every slide
- [x] Speaker notes on every slide
- [x] Clear ask on the ask slide (prize → audit + devnet launch)

## 3-minute video timing map (PITCH.md)
| Script section | Slides |
|---|---|
| 0:00–0:20 Hook | 2, 3 |
| 0:20–1:00 What it is | 3, 5 |
| 1:00–1:30 Proof of work | 5 |
| 1:30–1:50 Why us | 7 |
| 1:50–2:00 Ask | 9 |

## Package contents
- `DECK.md` (this) — full deck with speaker notes
- `SUBMISSION.md` — form answers (pitch, problem, team, tech list)
- `PITCH.md` — timed 2-minute video script
- `logo.png` — 800×800 project graphic
- `demo.mp4` (`../demo.mp4`) — 36s live product recording
- `README.md` — how to run the demo yourself
