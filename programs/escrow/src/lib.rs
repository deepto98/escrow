use anchor_lang::prelude::*;

declare_id!("4MgR17BFn1H1PVGm9cNqE4D48MDumf84x6NWEYD6XE12");

//Import module parent folders
mod state; // imports all modules from state folder ie escrow here
mod contexts; // imports make and take from contexts

use contexts::*;
 
#[program]
pub mod escrow {
    use contexts::{Make, Take};

    use super::*;

    pub fn make(ctx: Context<Make>, seed: u64, deposit: u64, receive: u64) -> Result<()> {
        ctx.accounts.init(seed, receive, &ctx.bumps)?;
        ctx.accounts.deposit(deposit)?;

        Ok(())
    }

    pub fn take(ctx: Context<Take>) -> Result<()> {
        ctx.accounts.deposit()?;
        ctx.accounts.withdraw_and_close()?;

        Ok(())
    }
}

#[derive(Accounts)]
pub struct Initialize {}
