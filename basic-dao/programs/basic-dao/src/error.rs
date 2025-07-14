use anchor_lang::prelude::*;

#[error_code]
pub enum DaoError {
    #[msg("Voting power is below the required threshold")]
    InsufficientVotingpower,
    #[msg("Proposal voting period has expired.")]
    ProposalExpired,
    #[msg("Proposal already executed")]
    AlreadyExecuted,
    #[msg("Proposal is still active")]
    ProposalStillActive,
    #[msg("Qyoram not met")]
    QuoramNotMet,
    #[msg("Stored and provided recipient didn't match")]
    InvalidRecipient,
    #[msg("Member power is below required threshold to create proposal")]
    InsufficientProposalCreationPower,
}
