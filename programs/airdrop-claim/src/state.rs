use anchor_lang::prelude::*;

/// One airdrop campaign. PDA-owned vault holds the distribution tokens.
#[account]
#[derive(InitSpace)]
pub struct Campaign {
    /// Who created (and may close) the campaign.
    pub authority: Pubkey,
    /// SPL mint being distributed.
    pub mint: Pubkey,
    /// Token account holding unclaimed tokens; token authority is this PDA.
    pub vault: Pubkey,
    /// Root of the merkle tree over sha256(claimant || amount_le).
    pub merkle_root: [u8; 32],
    /// Authority-chosen campaign id (part of PDA seeds).
    pub id: u64,
    pub total_deposited: u64,
    pub total_claimed: u64,
    pub bump: u8,
}

/// Proof that a claimant already claimed in a campaign. Existence = claimed.
/// `init` on this PDA fails on a second claim; no bitmap needed on-chain.
#[account]
#[derive(InitSpace)]
pub struct ClaimReceipt {
    pub campaign: Pubkey,
    pub amount: u64,
    pub claimed_at: i64,
    pub bump: u8,
}
