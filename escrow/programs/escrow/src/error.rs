use anchor_lang::prelude::*;

#[error_code]
pub enum EscrowError {
    #[msg("Unauthorized operation")]
    Unauthorized,
    #[msg("Too early to cancel")]
    TooEarly,
    #[msg("Request already fulfulled")]
    AlreadyFulfilled,
    #[msg("Escrow already cancelled")]
    AlreadyCancelled,
    #[msg("Party has not yet marked delivery")]
    NotDelivered,
    #[msg("Invalid amount")]
    InvalidAmount,
}
