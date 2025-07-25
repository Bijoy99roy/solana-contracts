use anchor_lang::prelude::*;

/// Represents a DAO account
/// This account holds all the basic configuration needed to start a dao
#[account]
pub struct DaoState {
    pub authority: Pubkey,
    pub token_mint: Pubkey,
    pub quoram: u64,
    pub proposal_duration: i64,
    pub min_voting_threshold: u64,
    pub min_proposal_creation_threshold: u64,
    pub bump: u8,
    pub vault_bump: u8,
}

impl DaoState {
    pub const MAX_SIZE: usize = 32 + 32 + 8 + 8 + 8 + 8 + 1 + 1;

    pub fn inialize(
        &mut self,
        authority: Pubkey,
        token_mint: Pubkey,
        quoram: u64,
        proposal_duration: i64,
        min_voting_threshold: u64,
        min_proposal_creation_threshold: u64,
        bump: u8,
        vault_bump: u8,
    ) {
        self.authority = authority;
        self.token_mint = token_mint;
        self.quoram = quoram;
        self.proposal_duration = proposal_duration;
        self.min_voting_threshold = min_voting_threshold;
        self.min_proposal_creation_threshold = min_proposal_creation_threshold;
        self.bump = bump;
        self.vault_bump = vault_bump;
    }
}
