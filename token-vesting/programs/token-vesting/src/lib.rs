use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token::{self, Mint, Token, TokenAccount, Transfer as SplTransfer},
};

mod error;
mod state;

use error::TokenVestingError;
use state::TokenVesting;

declare_id!("5qRj7P1BnXSTnhWBi6YBEBSZoYax8wZd9K92kPsj7Xeq");

#[program]
pub mod token_vesting {

    use super::*;

    /// Initializes a vesting schedule for a beneficiary.
    /// Transfers `total_amount` of SOL to a vault PDA.
    pub fn initialize_vesting(
        ctx: Context<InitializeVesting>,
        mint: Pubkey,
        beneficiary: Pubkey,
        vesting_period: i64,
        duration: i64,
        total_amount: u64,
        index: u64,
    ) -> Result<()> {
        let initalize_vesting_account = &mut ctx.accounts.vesting_account;

        require!(total_amount > 0, TokenVestingError::MustBeGreaterThenZero);
        require!(
            duration > 0 && vesting_period > 0,
            TokenVestingError::InvalidTimestamp
        );
        require!(
            duration > vesting_period,
            TokenVestingError::VestingPeriodExceedsDuration
        );

        require!(
            duration % vesting_period == 0,
            TokenVestingError::DurationNotDivisible
        );
        let clock = Clock::get()?;
        let now = clock.unix_timestamp;

        initalize_vesting_account.start_time = now;
        initalize_vesting_account.mint = mint;
        initalize_vesting_account.beneficiary = beneficiary;
        initalize_vesting_account.vesting_period = vesting_period;
        initalize_vesting_account.duration = duration;
        initalize_vesting_account.total_amount = total_amount;
        initalize_vesting_account.passed_periods = 0;

        let admin_account_info = ctx.accounts.admin_ata.to_account_info();
        let vault_info = ctx.accounts.vault_ata.to_account_info();
        let token_program_info = ctx.accounts.token_program.to_account_info();
        let user_account_info = ctx.accounts.user.to_account_info();

        // Transfer SPL tokens to vault
        let cpi_ctx = CpiContext::new(
            token_program_info,
            SplTransfer {
                from: admin_account_info,
                to: vault_info,
                authority: user_account_info,
            },
        );
        token::transfer(cpi_ctx, total_amount)?;

        Ok(())
    }

    /// Claims vested tokens based on elapsed time since vesting started.
    /// Tokens are linearly distributed over the duration in discrete periods.
    pub fn claim_vested_token(ctx: Context<ClaimVestedToken>, index: u64) -> Result<()> {
        let vesting_account = &mut ctx.accounts.vesting_account;
        let clock = Clock::get()?;
        let now = clock.unix_timestamp;

        let time_lapsed = now - vesting_account.start_time;

        require!(time_lapsed > 0, TokenVestingError::VestingNotStarted);
        require!(
            time_lapsed <= vesting_account.duration,
            TokenVestingError::VestingEnded
        );

        // Calculate the amount to be claimed
        let period_passed = time_lapsed / vesting_account.vesting_period;

        let claimable_periods = period_passed - vesting_account.passed_periods;

        require!(
            claimable_periods > 0,
            TokenVestingError::VestingPeriodNotReached
        );

        let total_periods = vesting_account.duration / vesting_account.vesting_period;
        let amount_per_period = vesting_account.total_amount / total_periods as u64;

        let mut claimable_amount = amount_per_period * claimable_periods as u64;

        // Add leftover tokens to last claim
        if vesting_account.passed_periods + claimable_periods >= total_periods {
            let remaining_amount = vesting_account.total_amount % total_periods as u64;
            claimable_amount += remaining_amount;
        }
        vesting_account.passed_periods += claimable_periods;

        let beneficiary_key = ctx.accounts.beneficiary.key();

        // Create seed signer for beneficiary account
        let seed = [b"vesting", beneficiary_key.as_ref(), &index.to_le_bytes()];
        let signer = &[&seed[..]];
        let beneficiary_ata_account_info = ctx.accounts.beneficiary_ata.to_account_info();
        let vault_info = ctx.accounts.vault_ata.to_account_info();
        let token_program_info = ctx.accounts.token_program.to_account_info();
        let vesting_account_info = vesting_account.to_account_info();

        // Add the claimable amount to claimed account for tracking
        vesting_account.claimed_amount = vesting_account
            .claimed_amount
            .checked_add(claimable_amount)
            .ok_or(TokenVestingError::Overflow)?;

        require!(
            vesting_account.claimed_amount <= vesting_account.total_amount,
            TokenVestingError::OverClaimed
        );

        // Transfering the claimable token from vault to beneficiary
        let cpi_ctx = CpiContext::new_with_signer(
            token_program_info,
            SplTransfer {
                from: vault_info,
                to: beneficiary_ata_account_info,
                authority: vesting_account_info,
            },
            signer,
        );

        token::transfer(cpi_ctx, claimable_amount)?;

        Ok(())
    }
}

#[derive(Accounts)]
#[instruction(index: u64)]
pub struct ClaimVestedToken<'info> {
    #[account(mut)]
    beneficiary: Signer<'info>,

    #[account(
        mut,
        seeds=[b"vesting", beneficiary.key().as_ref(), &index.to_le_bytes()],
        bump
    )]
    pub vesting_account: Account<'info, TokenVesting>,
    /// CHECK: Is a accountinfo refering to publickey
    #[account(address = vesting_account.key())]
    pub vesting_authority: AccountInfo<'info>,
    #[account(
        mut,
        associated_token::mint = mint,
        associated_token::authority = vesting_authority,
    )]
    pub vault_ata: Account<'info, TokenAccount>,

    #[account(
        init,
        payer = beneficiary,
        associated_token::mint = mint,
        associated_token::authority = beneficiary,
    )]
    pub beneficiary_ata: Account<'info, TokenAccount>,

    #[account(mut)]
    pub mint: Account<'info, Mint>,

    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
    pub associated_token_program: Program<'info, anchor_spl::associated_token::AssociatedToken>,
}

#[derive(Accounts)]
#[instruction(mint: Pubkey,
        beneficiary: Pubkey,
        vesting_period: i64,
        duration: i64,
        total_amount: u64,
        index: u64)]
pub struct InitializeVesting<'info> {
    #[account(mut)]
    user: Signer<'info>,

    #[account(
        init,
        space=8+32+32+32+8+8+8,
        seeds=[b"vesting", beneficiary.key().as_ref(), &index.to_le_bytes()],
        payer = user,
        bump
    )]
    pub vesting_account: Account<'info, TokenVesting>,
    /// CHECK: Is a accountinfo refering to publickey
    #[account(address = vesting_account.key())]
    pub vesting_authority: AccountInfo<'info>,

    #[account(
        init,
        associated_token::mint = mint,
        associated_token::authority = vesting_authority,
        payer = user,
    )]
    pub vault_ata: Account<'info, TokenAccount>,
    #[account(mut)]
    pub mint: Account<'info, Mint>,
    #[account(mut)]
    pub admin_ata: Account<'info, TokenAccount>,
    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
}
