use anchor_lang::prelude::*;

/// Represent VoteReceipt account
/// This hold the configuration for a vote receipt for a user per proposal

#[account]
pub struct VoteReceipt {
    pub proposal: Pubkey,
    pub voter: Pubkey,
    pub voted_yes: bool,
    pub voting_power: u64,
    pub bump: u8,
}
impl VoteReceipt {
    pub const MAX_SIZE: usize = 32 + 32 + 1 + 8 + 1;
}
