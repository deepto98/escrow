use anchor_lang::prelude::*;

declare_id!("4MgR17BFn1H1PVGm9cNqE4D48MDumf84x6NWEYD6XE12");

mod state; // use state module

#[program]
pub mod escrow {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
        Ok(())
    }
}

#[derive(Accounts)]
pub struct Initialize {}
