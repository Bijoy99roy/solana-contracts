use anchor_lang::prelude::*;

mod error;
mod state;

use error::TokenVestingError;
use state::TokenVesting;

declare_id!("5qRj7P1BnXSTnhWBi6YBEBSZoYax8wZd9K92kPsj7Xeq");

#[program]
pub mod token_vesting {
    use anchor_lang::system_program;

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

        let (_, vault_bump) = Pubkey::find_program_address(
            &[
                b"sol_vault",
                beneficiary.key().as_ref(),
                &index.to_le_bytes(),
            ],
            ctx.program_id,
        );
        initalize_vesting_account.vault_bump = vault_bump;

        let admin_account_info = ctx.accounts.user.to_account_info();
        let vault_info = ctx.accounts.vault.to_account_info();
        let system_program_info = ctx.accounts.system_program.to_account_info();

        // Transfering the total amount from admin to vault for storing
        let cpi_context = CpiContext::new(
            system_program_info,
            system_program::Transfer {
                from: admin_account_info,
                to: vault_info,
            },
        );

        system_program::transfer(cpi_context, total_amount)?;

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
        let seed = [b"vesting", beneficiary_key.as_ref(), &index.to_le_bytes()];
        let signer = &[&seed[..]];
        let beneficiary_account_info = ctx.accounts.beneficiary.to_account_info();
        let vault_info = ctx.accounts.vault.to_account_info();
        let system_program_info = ctx.accounts.system_program.to_account_info();
        vesting_account.claimed_amount = vesting_account
            .claimed_amount
            .checked_add(claimable_amount)
            .ok_or(TokenVestingError::Overflow)?;
        require!(
            vesting_account.claimed_amount <= vesting_account.total_amount,
            TokenVestingError::OverClaimed
        );
        // Transfering the claimable token from vault to beneficiary
        let cpi_context = CpiContext::new_with_signer(
            system_program_info,
            system_program::Transfer {
                from: vault_info,
                to: beneficiary_account_info,
            },
            signer,
        );

        system_program::transfer(cpi_context, claimable_amount)?;

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
    /// CHECK: This is a PDA derived vault account. Transferred to/from using CPI safely.
    #[account(
        mut,
        seeds = [b"sol_vault", vesting_account.beneficiary.key().as_ref(), &index.to_le_bytes()],
        bump=vesting_account.vault_bump
    )]
    pub vault: AccountInfo<'info>,
    pub system_program: Program<'info, System>,
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
    /// CHECK: This is a PDA derived vault account. Transferred to/from using CPI safely.
    #[account(
        mut,
        seeds = [b"sol_vault", beneficiary.key().as_ref(), &index.to_le_bytes()],
        bump
    )]
    pub vault: AccountInfo<'info>,
    pub system_program: Program<'info, System>,
}
