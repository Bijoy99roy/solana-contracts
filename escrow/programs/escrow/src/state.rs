use anchor_lang::prelude::*;

#[account]
pub struct EscrowAccount {
    pub initiator: Pubkey,
    pub party: Pubkey,
    pub amount: u64,
    pub is_fullfulled: bool,
    pub is_cancelled: bool,
    pub party_marked_delivered: bool,
    pub bump: u8,
    pub vault_bump: u8,
}
