use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token::{transfer, Mint, Token, TokenAccount, Transfer},
};

use crate::{constants::*, state::Campaign};

#[derive(Accounts)]
#[instruction(id: u64, merkle_root: [u8; 32])]
pub struct Initialize<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,
    #[account(
        init,
        payer = authority,
        space = 8 + Campaign::INIT_SPACE,
        seeds = [CAMPAIGN_SEED, authority.key().as_ref(), &id.to_le_bytes()],
        bump
    )]
    pub campaign: Account<'info, Campaign>,
    pub mint: Account<'info, Mint>,
    #[account(
        init,
        payer = authority,
        token::mint = mint,
        token::authority = campaign,
    )]
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

pub fn handle_initialize(
    ctx: Context<Initialize>,
    id: u64,
    merkle_root: [u8; 32],
    deposit_amount: u64,
) -> Result<()> {
    let campaign = &mut ctx.accounts.campaign;
    campaign.authority = ctx.accounts.authority.key();
    campaign.mint = ctx.accounts.mint.key();
    campaign.vault = ctx.accounts.vault.key();
    campaign.merkle_root = merkle_root;
    campaign.id = id;
    campaign.total_deposited = deposit_amount;
    campaign.total_claimed = 0;
    campaign.bump = ctx.bumps.campaign;

    // Seed the vault from the authority's ATA.
    transfer(
        CpiContext::new(
            ctx.accounts.token_program.key(),
            Transfer {
                from: ctx.accounts.authority_ata.to_account_info(),
                to: ctx.accounts.vault.to_account_info(),
                authority: ctx.accounts.authority.to_account_info(),
            },
        ),
        deposit_amount,
    )?;

    msg!("campaign {} initialized, {} tokens locked", id, deposit_amount);
    Ok(())
}
