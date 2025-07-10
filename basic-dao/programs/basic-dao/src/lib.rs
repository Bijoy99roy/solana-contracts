use anchor_lang::prelude::*;

declare_id!("Ew1sk5zpdc32FkkJqtXjKypLuWJRCBLTcPKxCwH75hFz");

#[program]
pub mod basic_dao {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
        msg!("Greetings from: {:?}", ctx.program_id);
        Ok(())
    }
}

#[derive(Accounts)]
pub struct Initialize {}
