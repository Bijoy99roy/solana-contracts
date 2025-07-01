use anchor_lang::prelude::*;

/// Represents a token vesting account.
///
/// Thsi account holds all the configuration needed to control how tokens are
/// gradually vested to a beneficiary over time

#[account]
pub struct TokenVesting {
    /// The vault PDA that owns and hold all tokens to be vested.
    /// This account should be controlled programatically and must not be accessed manually
    pub owner_vault: Pubkey,

    /// The beneficiary who will receive the vested tokens.
    pub beneficiary: Pubkey,

    /// The SPL token mint for the token being vested
    pub mint: Pubkey,

    /// The vesting period in seconds.
    /// Defines how frequently tokens become claimable (eg, Every 4 months)
    pub vesting_period: i64,

    /// The duration of the vesting schedule in seconds.
    /// Determines how long it will take for all tokens to fully vest.
    pub duration: i64,

    /// The total amount of tokens to be vested to the beneficiary
    /// Tokens will be linearly released over the duration based on the vesting period.
    pub total_amount: u64,

    /// Starting timestamp of vesting for a beneficiary
    pub start_time: i64,

    /// The amount the beneficiary has claimed so far
    pub claimed_amount: u64,

    /// How many periods has passed when beneficiary claimed the vested amount
    pub passed_periods: i64,

    /// The bump seed for the vault that stores solana
    pub vault_bump: u8,
}
