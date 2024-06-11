use anchor_lang::{ prelude::*, solana_program::instruction };
use anchor_spl::{
    associated_token::AssociatedToken,
    token::{ transfer, Mint, Token, TokenAccount, Transfer },
};

use crate::state::Escrow;

#[derive(Accounts)]
#[instruction(seed:u64)] //seed will be passed from client, we use instruction to use it within context
pub struct Make<'info> {
    #[account(mut)]
    pub maker: Signer<'info>, //person initializing escrow and paying fees and signing SPL tokens, has to be a signer
    pub mint_a: Account<'info, Mint>, //derive PDAs
    pub mint_b: Account<'info, Mint>,
    #[account(
        init,
        payer = maker,
        seeds = [b"escrow", maker.key.as_ref, seed.to_le_bytes().as_ref()], // by passing seed, we allow multiple escrows per user
        bump,
        space = Escrow::INIT_SPACE
    )]
    pub escrow: Account<'info, Escrow>,
    // ATAs - maker and vault
    #[account(
        mut,
        associated_token :: mint = mint_a,
        associated_token :: authority = maker,//checks if ata is derived from maker address
    )]
    pub maker_ata: Account<'info, TokenAccount>,

    #[account(
        init,
        payer = maker,
        associated_token::mint = mint_a,
        associated_token::authority = escrow //checks if ata is derived from maker address
    )]
    pub vault: Account<'info, TokenAccount>,

    // Programs - system prog, token prog, associated token prog
    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>, //ownership will be transferred to token program
    pub associated_token_program: Program<'info, AssociatedToken>,
}

impl<'info> Make<'info> {
    // Initialize state of escrow and associated
    pub fn init(&mut self, seed: u64, receive: u64, bumps: &MakeBumps) -> Result<()> {
        // set_inner helps us init the Escrow account struct
        self.escrow.set_inner(Escrow {
            seed,
            maker: self.maker.key(),
            mint_a: self.mint_a.key(),
            mint_b: self.mint_b.key(),
            receive,
            bump: bumps.escrow,
        });
        Ok(())
    }

    // Function for maker to deposit
    pub fn deposit(&mut self, deposit: u64) -> Result<()> {
        // For transferring sol, SystemProgram is target of CPI
        // For transferring SPL tokens,TokenProgram is the targeT
        let cpi_program = self.token_program.to_account_info();

        let cpi_accounts = Transfer {
            from: self.maker_ata.to_account_info(),
            to: self.vault.to_account_info(),
            authority: self.maker.to_account_info(),
        };

        let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);
        transfer(cpi_ctx, deposit)?;

        Ok(())
    }
}
