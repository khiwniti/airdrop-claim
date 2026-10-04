use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token::{transfer, Mint, Token, TokenAccount, Transfer},
};

use crate::{
    constants::*,
    error::ErrorCode,
    merkle::verify_proof,
    state::{Campaign, ClaimReceipt},
};

#[derive(Accounts)]
pub struct Claim<'info> {
    /// The claimant must sign: only they can grant themselves tokens.
    #[account(mut)]
    pub claimant: Signer<'info>,
    #[account(
        mut,
        seeds = [
            CAMPAIGN_SEED,
            campaign.authority.as_ref(),
            &campaign.id.to_le_bytes(),
        ],
        bump = campaign.bump,
        has_one = mint,
        has_one = vault,
    )]
    pub campaign: Account<'info, Campaign>,
    pub mint: Account<'info, Mint>,
    #[account(mut)]
    pub vault: Account<'info, TokenAccount>,
    #[account(
        init_if_needed,
        payer = claimant,
        associated_token::mint = mint,
        associated_token::authority = claimant,
    )]
    pub claimant_ata: Account<'info, TokenAccount>,
    /// Double-claim guard: second claim re-inits an existing account → fails.
    #[account(
        init,
        payer = claimant,
        space = 8 + ClaimReceipt::INIT_SPACE,
        seeds = [CLAIM_SEED, campaign.key().as_ref(), claimant.key().as_ref()],
        bump
    )]
    pub receipt: Account<'info, ClaimReceipt>,
    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
}

pub fn handle_claim(ctx: Context<Claim>, amount: u64, proof: Vec<[u8; 32]>) -> Result<()> {
    let campaign = &ctx.accounts.campaign;
    require!(
        verify_proof(
            &campaign.merkle_root,
            &ctx.accounts.claimant.key(),
            amount,
            &proof
        ),
        ErrorCode::InvalidMerkleProof
    );

    let receipt = &mut ctx.accounts.receipt;
    receipt.campaign = campaign.key();
    receipt.amount = amount;
    receipt.claimed_at = Clock::get()?.unix_timestamp;
    receipt.bump = ctx.bumps.receipt;

    let authority = campaign.authority;
    let id_le = campaign.id.to_le_bytes();
    let bump = [campaign.bump];
    let signer_seeds: &[&[u8]] = &[CAMPAIGN_SEED, authority.as_ref(), &id_le, &bump];

    transfer(
        CpiContext::new_with_signer(
            ctx.accounts.token_program.key(),
            Transfer {
                from: ctx.accounts.vault.to_account_info(),
                to: ctx.accounts.claimant_ata.to_account_info(),
                authority: ctx.accounts.campaign.to_account_info(),
            },
            &[signer_seeds],
        ),
        amount,
    )?;

    ctx.accounts.campaign.total_claimed = ctx
        .accounts
        .campaign
        .total_claimed
        .checked_add(amount)
        .ok_or(ErrorCode::InvalidMerkleProof)?; // unreachable in practice; vault would fail first

    msg!("claimed {}", amount);
    Ok(())
}
