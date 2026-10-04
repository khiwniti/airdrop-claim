use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token::{transfer, Mint, Token, TokenAccount, Transfer},
};

use crate::{constants::*, state::Campaign};

#[derive(Accounts)]
pub struct CloseCampaign<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,
    #[account(
        mut,
        seeds = [
            CAMPAIGN_SEED,
            authority.key().as_ref(),
            &campaign.id.to_le_bytes(),
        ],
        bump = campaign.bump,
        has_one = authority,
        has_one = mint,
        has_one = vault,
        close = authority,
    )]
    pub campaign: Account<'info, Campaign>,
    pub mint: Account<'info, Mint>,
    #[account(mut)]
    pub vault: Account<'info, TokenAccount>,
    #[account(
        init_if_needed,
        payer = authority,
        associated_token::mint = mint,
        associated_token::authority = authority,
    )]
    pub authority_ata: Account<'info, TokenAccount>,
    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
}

/// Authority sweeps any unclaimed remainder; the `close` constraint refunds
/// the campaign account rent. Claim receipts stay on-chain for auditability.
pub fn handle_close_campaign(ctx: Context<CloseCampaign>) -> Result<()> {
    let remainder = ctx.accounts.vault.amount;

    let campaign = &ctx.accounts.campaign;
    let authority_key = campaign.authority;
    let id_le = campaign.id.to_le_bytes();
    let bump = [campaign.bump];
    let signer_seeds: &[&[u8]] = &[CAMPAIGN_SEED, authority_key.as_ref(), &id_le, &bump];

    if remainder > 0 {
        transfer(
            CpiContext::new_with_signer(
                ctx.accounts.token_program.key(),
                Transfer {
                    from: ctx.accounts.vault.to_account_info(),
                    to: ctx.accounts.authority_ata.to_account_info(),
                    authority: ctx.accounts.campaign.to_account_info(),
                },
                &[signer_seeds],
            ),
            remainder,
        )?;
    }

    msg!("campaign closed, {} tokens recovered", remainder);
    Ok(())
}
