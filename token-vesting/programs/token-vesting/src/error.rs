use anchor_lang::prelude::*;

#[error_code]
pub enum TokenVestingError {
    /// The PDA(vault) don't have enough tokens
    #[msg("Not enough tokens to vest")]
    NotEnoughToken,

    /// Attempted to withdraw more token than vested
    #[msg("Requested amount exceeds vested amount")]
    InvalidTokenAmount,

    /// Mismatch in mint between in vault vs config
    #[msg("Token mint mismatch")]
    MintMismatch,

    /// The vesting duration is aleady completed.
    #[msg("Vesting is already complete.")]
    VestingCompleted,

    /// Vault PDA derivation or authority mismatch
    #[msg("Vault authority mismatch")]
    VaultAuthorityMismatch,

    /// The beneficiary is not authorized to claim this vesting
    #[msg("Unauthorized beneficiary")]
    UnauthorizedBeneficiary,

    /// The token alloted in current period has been claimed.
    #[msg("Tokens already claimed for this period")]
    AlreadyClaimed,

    /// The provided vesting period is more than duration. (vesting period < duration) always
    #[msg("Vesting period is exceeing duration")]
    VestingPeriodExceedsDuration,

    /// The total amount for vesting for a beneficiary must always be greater than zero
    #[msg("Total amount must be greater than zero")]
    MustBeGreaterThenZero,

    /// Timestamp has some invalid nature
    #[msg("Provided timestamp is invalid!!")]
    InvalidTimestamp,

    /// If duration is divided by vesting period the remainder is not zero
    #[msg("Duration is not divisible by vesting period")]
    DurationNotDivisible,
}
