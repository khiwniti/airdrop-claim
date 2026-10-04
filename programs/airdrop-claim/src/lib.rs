pub mod constants;
pub mod error;
pub mod instructions;
pub mod merkle;
pub mod state;

use anchor_lang::prelude::*;

pub use constants::*;
pub use instructions::*;
pub use state::*;

declare_id!("2NepmY26ip3y5gPWaY5ZoAUUWNtqqMjDobiAyFtCj5Jf");

#[program]
pub mod airdrop_claim {
    use super::*;

    /// Create a merkle-root-gated airdrop campaign and fund its PDA vault.
    pub fn initialize(
        ctx: Context<Initialize>,
        id: u64,
        merkle_root: [u8; 32],
        deposit_amount: u64,
    ) -> Result<()> {
        crate::instructions::initialize::handle_initialize(ctx, id, merkle_root, deposit_amount)
    }

    /// Claim an allocation by presenting a merkle proof of (claimant, amount).
    pub fn claim(ctx: Context<Claim>, amount: u64, proof: Vec<[u8; 32]>) -> Result<()> {
        crate::instructions::claim::handle_claim(ctx, amount, proof)
    }

    /// Authority sweeps unclaimed tokens and closes the campaign account.
    pub fn close_campaign(ctx: Context<CloseCampaign>) -> Result<()> {
        crate::instructions::close_campaign::handle_close_campaign(ctx)
    }
}
