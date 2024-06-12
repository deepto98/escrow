use anchor_lang::{ prelude::*, solana_program::instruction };
use anchor_spl::{
    associated_token::AssociatedToken,
    token::{ transfer, Mint, Token, TokenAccount, Transfer },
};

use crate::state::Escrow;

// Taker accepts terms, we send tokens from taker's ata for mint b to maker's ata for mint b
// Then we send tokens in vault to taker's ata on mint a
#[derive(Accounts)]
#[instruction(seed:u64)] //seed will be passed from client, we use instruction to use it within context
pub struct Take<'info> {
    #[account(mut)] //mut because maker pays for init - sol change
    pub taker: Signer<'info>, //person initializing escrow and paying fees and signing SPL tokens, has to be a signer

    pub maker: SystemAccount<'info>,

    // Mint Accounts
    pub mint_a: Account<'info, Mint>, //derive PDAs. to be stored in escrow
    pub mint_b: Account<'info, Mint>, //to be stored in escrow

    #[account(
        seeds = [b"escrow", escrow.maker.key.as_ref, escrow.seed.to_le_bytes().as_ref()], // by passing seed, we allow multiple escrows per user. seed is fetched from instruction macro here
        bump = escrow.bump,
        has_one = mint_a, //states that escrow needs to have mint acc with same addressmas mint a here
        has_one = mint_b
    )]
    pub escrow: Account<'info, Escrow>,

    // ATAs
    #[account(
        mut, //mut since taker gets tokens from vault
        associated_token :: mint = mint_a,
        associated_token :: authority = escrow,  
    )]
    pub vault: Account<'info, TokenAccount>,

    #[account(
        init_if_needed,
        payer = taker,
        associated_token::mint = mint_b,
        associated_token::authority = maker //checks if ata is derived from maker address
    )]
    pub maker_ata: Account<'info, TokenAccount>,

    #[account(
        init_if_needed,
        payer = taker,
        associated_token::mint = mint_a,
        associated_token::authority = taker
    )]
    pub taker_ata_a: Account<'info, TokenAccount>,

    #[account(
        mut,
        associated_token::mint=mint_b,
        associated_token::authority=taker
    )]
    pub taker_ata_b: Account<'info, TokenAccount>,

    // Programs - system prog, token prog, associated token prog
    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>, //ownership will be transferred to token program
    pub associated_token_program: Program<'info, AssociatedToken>,
}

impl<'info> Take<'info> {
    // pub fn take(&mut self, seed: u64) -> Result<()> {
    //     self.escrow.send = send;
    //     self.vault.amount = send;
    //     self.taker_ata_a.amount = self.taker_ata_a.amount
    //         .checked_add(self.escrow.receive)
    //         .ok_or(ErrorCode::MathError)?;

    //     self.taker_ata_b.amount = self.taker_ata_b.amount
    //         .checked_add(send)
    //         .ok_or(ErrorCode::MathError)?;
    //     Ok(())
    // }

    pub fn transfer(&mut self) -> Result<()> {
        // token program coz spl
        let cpi_program = self.token_program.to_account_info();

        //taker_ata_b to maker_ata
        let cpi_accounts = Transfer {
            from: self.taker_ata_b.to_account_info(),
            to: self.maker_ata.to_account_info(),
            authority: self.taker.to_account_info(),
        };

        let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);
        transfer(cpi_ctx, self.escrow.receive)?;

        Ok(())
    }

    // vault to taker_ata_a
    pub fn withdraw (&mut self) -> Result<()> {
        // token program coz spl
        let cpi_program = self.token_program.to_account_info();

        //taker_ata_b to maker_ata
        let cpi_accounts = Transfer {
            from: self.vault.to_account_info(),
            to: self.taker_ata_a.to_account_info(),
            authority: self.escrow.to_account_info(),
        };

        let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);
        transfer(cpi_ctx, self.escrow.receive)?;

        Ok(())
    }
}
