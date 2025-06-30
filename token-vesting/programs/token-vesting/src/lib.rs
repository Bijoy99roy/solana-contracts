use anchor_lang::prelude::*;

mod error;
mod state;

use error::TokenVestingError;
use state::TokenVesting;
use state::UserIndex;

declare_id!("5qRj7P1BnXSTnhWBi6YBEBSZoYax8wZd9K92kPsj7Xeq");

#[program]
pub mod token_vesting {
    use super::*;

    pub fn initialize_vesting(
        ctx: Context<InitializeVesting>,
        mint: Pubkey,
        beneficiary: Pubkey,
        vesting_period: i64,
        duration: i64,
        total_amount: u64,
    ) -> Result<()> {
        let initalize_vesting_account = &mut ctx.accounts.vesting_account;
        let user_index = &mut ctx.accounts.user_index;
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
        initalize_vesting_account.mint = mint;
        initalize_vesting_account.beneficiary = beneficiary;
        initalize_vesting_account.vesting_period = vesting_period;
        initalize_vesting_account.duration = duration;
        initalize_vesting_account.total_amount = total_amount;

        user_index.current_index += 1;
        Ok(())
    }

    pub fn initialize_user_index(ctx: Context<InitializeUserIndex>) -> Result<()> {
        let user_index = &mut ctx.accounts.user_index;
        user_index.current_index = 0;

        Ok(())
    }

    pub fn claim_vested_token(ctx: Context<ClaimVestedToken>) -> Result<()> {
        Ok(())
    }
}

#[derive(Accounts)]
pub struct ClaimVestedToken<'info> {
    #[account(mut)]
    beneficiary: Signer<'info>,
    #[account(
        mut,
        seeds=[b"user_index", beneficiary.key().as_ref()],
        bump,
    )]
    pub user_index: Account<'info, UserIndex>,
    #[account(
        mut,
        seeds=[b"vesting", beneficiary.key().as_ref(), &user_index.current_index.to_le_bytes()],
        bump
    )]
    pub vesting_account: Account<'info, TokenVesting>,
}

#[derive(Accounts)]
pub struct InitializeUserIndex<'info> {
    #[account(mut)]
    pub user: Signer<'info>,
    pub beneficiary: SystemAccount<'info>,
    #[account(
        init,
        seeds=[b"user_index", beneficiary.key().as_ref()],
        bump,
        payer = user,
        space = 8+8
    )]
    pub user_index: Account<'info, UserIndex>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
#[instruction(mint: Pubkey,
        beneficiary: Pubkey,
        vesting_period: i64,
        duration: i64,
        total_amount: u64,)]
pub struct InitializeVesting<'info> {
    #[account(mut)]
    user: Signer<'info>,
    #[account(
        mut,
        seeds=[b"user_index", beneficiary.key().as_ref()],
        bump,
    )]
    pub user_index: Account<'info, UserIndex>,
    #[account(
        init,
        space=8+32+32+32+8+8+8,
        seeds=[b"vesting", beneficiary.key().as_ref(), &user_index.current_index.to_le_bytes()],
        payer = user,
        bump
    )]
    pub vesting_account: Account<'info, TokenVesting>,
    pub system_program: Program<'info, System>,
}
