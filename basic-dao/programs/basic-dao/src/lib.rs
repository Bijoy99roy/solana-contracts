use anchor_lang::prelude::*;
use anchor_spl::token::{Mint, Token, TokenAccount, Transfer as SplTransfer};

use crate::state::{dao::DaoState, proposal::Proposal, vote_receipt::VoteReceipt};

mod error;
mod state;

declare_id!("Ew1sk5zpdc32FkkJqtXjKypLuWJRCBLTcPKxCwH75hFz");

#[program]
pub mod basic_dao {

    use anchor_spl::token;

    use crate::error::DaoError;

    use super::*;

    pub fn initialize_dao(
        ctx: Context<InitializeDaoContext>,
        quoram: u64,
        proposal_duration: i64,
        min_voting_threshold: u64,
        min_proposal_creation_threshold: u64,
        token_allocation: u64,
    ) -> Result<()> {
        let (_, dao_bump) = Pubkey::find_program_address(
            &[b"dao", ctx.accounts.token_mint.key().as_ref()],
            ctx.program_id,
        );
        let (_, vault_bump) = Pubkey::find_program_address(
            &[b"vault", ctx.accounts.token_mint.key().as_ref()],
            ctx.program_id,
        );
        ctx.accounts.dao.inialize(
            ctx.accounts.authority.key(),
            ctx.accounts.token_mint.key(),
            quoram,
            proposal_duration,
            min_voting_threshold,
            min_proposal_creation_threshold,
            dao_bump,
            vault_bump,
        );

        let authority_ata_account_info = ctx.accounts.authority_ata.to_account_info();
        let vault_ata_account_info = ctx.accounts.vault_ata.to_account_info();
        let authority_account_info = ctx.accounts.authority.to_account_info();
        let token_program_info = ctx.accounts.token_program.to_account_info();

        let cpi_context = CpiContext::new(
            token_program_info,
            SplTransfer {
                from: authority_ata_account_info,
                to: vault_ata_account_info,
                authority: authority_account_info,
            },
        );
        token::transfer(cpi_context, token_allocation)?;

        msg!(
            "Dao initalized successfully with {} tokens",
            token_allocation
        );

        Ok(())
    }

    pub fn create_proposal(
        ctx: Context<CreateProposal>,
        proposal_index: u64,
        description: String,
        
        action_amount: u64,
        action_target: Pubkey,
    ) -> Result<()> {
        let clock = Clock::get()?;
        ctx.accounts.proposal.initilize(
            &ctx.accounts.dao,
            &ctx.accounts.proposer_token_account,
            ctx.accounts.proposer.key(),
            &clock,
            ctx.accounts.dao.proposal_duration,
            action_amount,
            action_target,
            description,
        )?;
        Ok(())
    }

    pub fn cast_vote(ctx: Context<CastVoteContext>, vote_yes:bool) -> Result<()> {
        let proposal_key = ctx.accounts.proposal.key();
        ctx.accounts.proposal.cast_vote(
            &ctx.accounts.voter_token_account, 
            &ctx.accounts.voter.key(), 
            vote_yes, 
            &ctx.accounts.dao, 
            &mut ctx.accounts.vote_receipt, 
            proposal_key
        )?;
        Ok(())
    }

    pub fn execute_proposal(ctx: Context<ExecuteProposalContext>) -> Result<()> {
        let proposal = &mut ctx.accounts.proposal;
        let dao = &ctx.accounts.dao;
        let clock = Clock::get()?;

        require!(!proposal.executed, DaoError::AlreadyExecuted);
        require!(clock.unix_timestamp > proposal.end_time, DaoError::ProposalStillActive);
        require!(proposal.yes_votes >= dao.quoram, DaoError::QuoramNotMet);

        let vault_ata_account_info = ctx.accounts.vault_ata.to_account_info();
        let recepient_token_account_info = ctx.accounts.recipient_token_account.to_account_info();
        let authority_account_info = ctx.accounts.dao.to_account_info();
        require_keys_eq!(
            proposal.action_target,
            ctx.accounts.recipient_token_account.key(),
            DaoError::InvalidRecipient
        );
        let seed = [b"dao", dao.token_mint.as_ref(), &[dao.bump]];
        let signer = &[&seed[..]];

        let cpi_context = CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(), 
            SplTransfer{
                from: vault_ata_account_info,
                to: recepient_token_account_info,
                authority: authority_account_info
            }, 
            signer);

        token::transfer(cpi_context, proposal.action_amount)?;
        proposal.executed = true;
        Ok(())
    }

    
}

#[derive(Accounts)]
pub struct ExecuteProposalContext<'info> {
    pub dao: Account<'info, DaoState>,

    #[account(mut)]
    pub proposal: Account<'info, Proposal>,

    #[account(
        mut,
        seeds = [b"vault", dao.token_mint.key().as_ref()],
        bump=dao.vault_bump,
        token::mint = dao.token_mint.key(),
        token::authority = dao.key(),

    )]
    pub vault_ata: Account<'info, TokenAccount>,

    #[account(mut)]
    pub recipient_token_account: Account<'info, TokenAccount>,

    pub token_program: Program<'info, Token>,
}

#[derive(Accounts)]
pub struct CastVoteContext<'info> {
    #[account(mut)]
    pub voter: Signer<'info>,

    #[account(
        mut,
        seeds = [b"dao", dao.token_mint.key().as_ref()],
        bump = dao.bump,
    )]
    pub dao: Account<'info, DaoState>,

    #[account(mut)]
    pub proposal: Account<'info, Proposal>,

    #[account(
        init,
        payer = voter,
        space = 8 + VoteReceipt::MAX_SIZE,
        seeds = [b"vote", proposal.key().as_ref(), voter.key().as_ref()],
        bump,
    )]
    pub vote_receipt: Account<'info, VoteReceipt>,
    #[account(
        constraint = voter_token_account.owner == voter.key(),
        constraint = voter_token_account.mint == dao.token_mint
    
    )]
    pub voter_token_account: Account<'info, TokenAccount>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
#[instruction(proposal_index: u64)]
pub struct CreateProposal<'info> {
    #[account(mut)]
    pub proposer: Signer<'info>,
    #[account(
        mut,
        seeds = [b"dao", dao.token_mint.key().as_ref()],
        bump = dao.bump,
    )]
    pub dao: Account<'info, DaoState>,

    #[account(
        init,
        payer = proposer,
        space = 8 + Proposal::MAX_SIZE,
        seeds = [b"proposal", dao.key().as_ref(), proposer.key().as_ref(), &proposal_index.to_le_bytes()],
        bump,
    )]
    pub proposal: Account<'info, Proposal>,
    #[account(
        constraint = proposer_token_account.owner == proposer.key(),
        constraint = proposer_token_account.mint == dao.token_mint
    
    )]
    pub proposer_token_account: Account<'info, TokenAccount>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct InitializeDaoContext<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,
    #[account(
        init,
        payer = authority,
        space = 8  + DaoState::MAX_SIZE,
        seeds = [b"dao", token_mint.key().as_ref()],
        bump
    )]
    pub dao: Account<'info, DaoState>,
    #[account(
        init,
        seeds = [b"vault", token_mint.key().as_ref()],
        bump,
        token::mint = token_mint,
        token::authority = dao,
        payer = authority,
    )]
    pub vault_ata: Account<'info, TokenAccount>,
    #[account(mut)]
    pub authority_ata: Account<'info, TokenAccount>,

    pub token_mint: Account<'info, Mint>,
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}
