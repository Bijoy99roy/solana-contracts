use anchor_lang::prelude::*;

/// The following account represents a Proposal account
/// This holds all the configuration needed for a proposal to form

#[account]
pub struct Proposal {
    pub dao: Pubkey,
    pub proposer: Pubkey,
    pub yes_votes: u64,
    pub no_votes: u64,
    pub start_time: i64,
    pub end_time: i64,
    pub executed: bool,
    pub action_amount: u64,
    pub bump: u8,
    pub description: String,
}

impl Proposal {
    pub const MAX_SIZE: usize = 32 + 32 + 8 + 8 + 8 + 8 + 1 + 8 + 1 + (4 + 200);

    pub fn initilize(
        &mut self,
        dao: Pubkey,
        proposer: Pubkey,
        clock: &Clock,
        duration: i64,
        action_amount: u64,
        action_target: Pubkey
        description: String
    ) -> Result<()> {
        self.dao = dao;
        self.proposer = proposer;
        self.description = description;
        self.yes_votes = 0;
        self.no_votes = 0;
        self.start_time = clock.unix_timestamp;
        self.end_time = clock.unix_timestamp + duration;
        self.executed = false;
        self.action_amount = action_amount;
        self.action_target = action_target;
        Ok(())
    }

    
}
