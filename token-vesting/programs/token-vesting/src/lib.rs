use anchor_lang::prelude::*;

declare_id!("5qRj7P1BnXSTnhWBi6YBEBSZoYax8wZd9K92kPsj7Xeq");

#[program]
pub mod token_vesting {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
        msg!("Greetings from: {:?}", ctx.program_id);
        Ok(())
    }
}

#[derive(Accounts)]
pub struct Initialize {}
