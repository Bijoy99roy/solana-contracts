use anchor_lang::prelude::*;

use anchor_spl::token::TokenAccount;

use crate::error::DaoError;
use crate::state::{dao::DaoState, vote_receipt::VoteReceipt};
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
    pub action_target: Pubkey,
    pub bump: u8,
    pub description: String,
}

impl Proposal {
    pub const MAX_SIZE: usize = 32 + 32 + 8 + 8 + 8 + 8 + 1 + 8 + 1 + (4 + 200);

    pub fn initilize(
        &mut self,
        dao: &Account<DaoState>,
        token_account: &Account<TokenAccount>,
        proposer: Pubkey,
        clock: &Clock,
        duration: i64,
        action_amount: u64,
        action_target: Pubkey,
        description: String,
    ) -> Result<()> {
        require!(
            token_account.amount >= dao.min_proposal_creation_threshold,
            DaoError::InsufficientProposalCreationPower
        );
        self.dao = dao.key();
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

    pub fn cast_vote(
        &mut self,
        token_account: &Account<TokenAccount>,
        voter: &Pubkey,
        vote_yes: bool,
        dao: &DaoState,
        vote_receipt: &mut Account<VoteReceipt>,
        proposal_key: Pubkey,
    ) -> Result<()> {
        // Apply square root to reduce the influence of whale without punishing then completely
        let voting_power = (token_account.amount as f64).sqrt().floor() as u64;
        require!(
            voting_power >= dao.min_voting_threshold,
            DaoError::InsufficientVotingpower
        );
        let clock = Clock::get()?;
        require!(
            clock.unix_timestamp < self.end_time,
            DaoError::ProposalExpired
        );

        if vote_yes {
            self.yes_votes = self.yes_votes.checked_add(voting_power).unwrap();
        } else {
            self.no_votes = self.no_votes.checked_add(voting_power).unwrap();
        }
        vote_receipt.proposal = proposal_key;
        vote_receipt.voter = *voter;
        vote_receipt.voted_yes = vote_yes;
        vote_receipt.voting_power = voting_power;
        Ok(())
    }
}
