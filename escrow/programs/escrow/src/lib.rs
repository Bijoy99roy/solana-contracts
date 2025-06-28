use anchor_lang::prelude::*;

mod error;
mod state;
use state::EscrowAccount;
use error::EscrowError;
declare_id!("ASJyvDzyjhR9ACbKK8gui155Kk2VFLYhGQAqpK2SS788");

#[program]
pub mod escrow {
    use anchor_lang::system_program;

    

    use super::*;

    pub fn initialize_escrow(ctx: Context<InitializeEscrow>, amount:u64, escrow_index: u64) -> Result<()> {
        require!(amount > 0, EscrowError::InvalidAmount);
        let escrow_account = &mut ctx.accounts.escrow;

        escrow_account.initiator = ctx.accounts.payer.key();
        escrow_account.party = ctx.accounts.party.key();
        escrow_account.amount = amount;
        escrow_account.is_fullfulled = false;
        escrow_account.is_cancelled = false;
        escrow_account.party_marked_delivered = false;
        escrow_account.bump = ctx.bumps.escrow;
        escrow_account.index = escrow_index;
        let (_, vault_bump) = Pubkey::find_program_address(&[b"sol_vault", ctx.accounts.payer.key().as_ref(), &escrow_index.to_le_bytes()], ctx.program_id);
        escrow_account.vault_bump = vault_bump;

        let initiator_account_info = ctx.accounts.payer.to_account_info();
        let vault_info = ctx.accounts.vault.to_account_info();
        let system_program_info = ctx.accounts.system_program.to_account_info();

        let cpi_context = CpiContext::new(
            system_program_info,
            system_program::Transfer {
                from: initiator_account_info,
                to: vault_info
            }
        );

        system_program::transfer(cpi_context, amount)?;

        msg!("Escrow created successfully!!");
        msg!("Total pay is {} lamports", amount);
        Ok(())
    }

    pub fn mark_as_delivered(ctx: Context<MarkAsDelivered>) -> Result<()> {
        let escrow_account = &mut ctx.accounts.escrow;
        require_keys_eq!(escrow_account.party, ctx.accounts.party.key(), EscrowError::Unauthorized);
        require!(!escrow_account.is_cancelled, EscrowError::AlreadyCancelled);
        require!(!escrow_account.is_fullfulled, EscrowError::AlreadyFulfilled);
        escrow_account.party_marked_delivered = true;

        Ok(())
    }

    pub fn delivery_fulfilled(ctx: Context<DeliveryFullfilled>) -> Result<()> {
        let escrow_account = &mut ctx.accounts.escrow;

        require_keys_eq!(ctx.accounts.initiator.key(), escrow_account.initiator, EscrowError::Unauthorized);
        require!(escrow_account.party_marked_delivered, EscrowError::NotDelivered);
        require!(!escrow_account.is_cancelled, EscrowError::AlreadyCancelled);
        require!(!escrow_account.is_fullfulled, EscrowError::AlreadyFulfilled);

        let initiator_key = ctx.accounts.initiator.key();
        let seed = [b"sol_vault", initiator_key.as_ref(), &escrow_account.index.to_le_bytes(), &[escrow_account.vault_bump]];
        let signer = &[&seed[..]];

        let party_account_info = ctx.accounts.party.to_account_info();
        let vault_info = ctx.accounts.vault.to_account_info();
        let system_program_info = ctx.accounts.system_program.to_account_info();


        let cpi_context = CpiContext::new_with_signer(system_program_info, 
        system_program::Transfer{
            from: vault_info,
            to: party_account_info
        },
        signer
    );

        system_program::transfer(cpi_context, escrow_account.amount)?;
        escrow_account.is_fullfulled = true;
        msg!("Delivery successfull!!");
        msg!("{} lamports transfered to {}", escrow_account.amount, escrow_account.party);

        Ok(())
    }

    pub fn cancel_escrow(ctx: Context<CancelEscrow>) -> Result<()>{
        let escrow_account= &mut ctx.accounts.escrow;
        require_keys_eq!(ctx.accounts.initiator.key(), escrow_account.initiator, EscrowError::Unauthorized);
        require!(!escrow_account.is_cancelled, EscrowError::AlreadyCancelled);
        require!(!escrow_account.is_fullfulled, EscrowError::AlreadyFulfilled);

        let initiator_key = ctx.accounts.initiator.key();
        let seed = [b"sol_vault", initiator_key.as_ref(), &escrow_account.index.to_le_bytes(), &[escrow_account.vault_bump]];
        let signer = &[&seed[..]];

        let initiator_account_info = ctx.accounts.initiator.to_account_info();
        let vault_info = ctx.accounts.vault.to_account_info();
        let system_program_info = ctx.accounts.system_program.to_account_info();


        let cpi_context = CpiContext::new_with_signer(system_program_info, 
        system_program::Transfer{
            from: vault_info,
            to: initiator_account_info
        },
        signer
    );

        system_program::transfer(cpi_context, escrow_account.amount)?;
        escrow_account.is_cancelled= true;
        msg!("Escrow cancelled!!");
        msg!("{} lamports transfered back to initator: {}", escrow_account.amount, escrow_account.initiator);
        Ok(())
    }
}

#[derive(Accounts)]
pub struct CancelEscrow<'info> {
    #[account(mut)]
    pub initiator: Signer<'info>,
    #[account(
        mut,
        seeds = [b"escrow", escrow.initiator.as_ref(), &escrow.index.to_le_bytes()],
        bump = escrow.bump
    )]
    pub escrow: Account<'info, EscrowAccount>,
    /// CHECK: This is a PDA derived vault account. Transferred to/from using CPI safely.
    #[account(
        mut,
        seeds = [b"sol_vault", escrow.initiator.as_ref(), &escrow.index.to_le_bytes()],
        bump = escrow.vault_bump
    )]
    pub vault: AccountInfo<'info>,
    pub system_program: Program<'info, System>
}

#[derive(Accounts)]
pub struct DeliveryFullfilled<'info> {
    #[account(mut)]
    pub initiator: Signer<'info>,
    #[account(mut)]
    pub party: Signer<'info>,
    #[account(
        mut,
        seeds = [b"escrow", escrow.initiator.as_ref(), &escrow.index.to_le_bytes()],
        bump = escrow.bump
    )]
    pub escrow: Account<'info, EscrowAccount>,
    /// CHECK: This is a PDA derived vault account. Transferred to/from using CPI safely.
    #[account(
        mut,
        seeds = [b"sol_vault", escrow.initiator.as_ref(), &escrow.index.to_le_bytes()],
        bump = escrow.vault_bump
    )]
    pub vault: AccountInfo<'info>,
    pub system_program: Program<'info, System>
}

#[derive(Accounts)]
pub struct  MarkAsDelivered<'info> {
    
    pub party: Signer<'info>,
    #[account(
    mut,
    seeds = [b"escrow", escrow.initiator.as_ref(), &escrow.index.to_le_bytes()],
    bump = escrow.bump
    )]
    pub escrow: Account<'info, EscrowAccount>
}

#[derive(Accounts)]
#[instruction(amount: u64, escrow_index: u64)]
pub struct InitializeEscrow<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,
    /// CHECK: Party is only used for storing the public key and not read/written directly.
    pub party: AccountInfo<'info>,
    #[account(
        init,
        payer=payer,
        space = 8 + 32 + 32 + 8 + 1 + 1 + 1 + 1 + 1 + 8,
        seeds = [b"escrow", payer.key().as_ref(), &escrow_index.to_le_bytes()],
        bump
        
    )]
    pub escrow: Account<'info, EscrowAccount>,
    /// CHECK: This is a PDA derived vault account. Transferred to/from using CPI safely.
    #[account(
        mut,
        seeds = [b"sol_vault", payer.key().as_ref(), &escrow_index.to_le_bytes()],
        bump
    )]
    pub vault: AccountInfo<'info>,
    pub system_program: Program<'info, System>
}


