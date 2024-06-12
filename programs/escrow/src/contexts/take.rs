use anchor_lang::{ prelude::*, solana_program::instruction };
use anchor_spl::{
    associated_token::AssociatedToken,
    token::{ close_account, transfer, CloseAccount, Mint, Token, TokenAccount, Transfer },
};

use crate::state::Escrow;

// Taker - First, Vault to taker. Then, taker_ata_b to maker_ata_b.

// Taker accepts terms, we send tokens from taker's ata for mint b to maker's ata for mint b
// Then we send tokens in vault to taker's ata on mint a
#[derive(Accounts)]
#[instruction(seed:u64)] //seed will be passed from client, we use instruction to use it within context
pub struct Take<'info> {
    #[account(mut)] // taker pays initialization fees, so is mutable
    pub taker: Signer<'info>, //taker signs the txn

    pub maker: SystemAccount<'info>,

    // Mint Accounts mint a : mint add of maker, mint_b : of taker
    pub mint_a: Account<'info, Mint>, //derive PDAs. to be stored in escrow
    pub mint_b: Account<'info, Mint>, //to be stored in escrow

    #[account(
        mut, //since we'll close it after the txn
        seeds = [b"escrow", escrow.maker.key.as_ref, escrow.seed.to_le_bytes().as_ref()], // by passing seed, we allow multiple escrows per user. seed is fetched from instruction macro here
        bump = escrow.bump,

        //Checks if these items are already stored in the account struct
        has_one = mint_a, //states that escrow needs to have mint acc with same address  as mint a here
        has_one = mint_b,
        has_one = maker,
        close = maker
    )]
    pub escrow: Account<'info, Escrow>,

    // ATAs
    #[account(
        mut, //mut since taker gets tokens from vault
        associated_token :: mint = mint_a,
        associated_token :: authority = escrow,  
    )]
    pub vault: Account<'info, TokenAccount>,

    // taker_ata_a - receives maker's tokens from mint_a
    #[account(
        init_if_needed, // since we don't know if the acc already exists
        payer = taker,
        associated_token::mint = mint_a,
        associated_token::authority = taker // taker is auth (wallet add for taker_ata_a)
    )]
    pub taker_ata_a: Account<'info, TokenAccount>,

    // taker_ata_b - sends taker's tokens from mint_b
    #[account(
        mut,
        associated_token::mint=mint_b,
        associated_token::authority=taker
    )]
    pub taker_ata_b: Account<'info, TokenAccount>,

    #[account(
        init_if_needed,
        payer = taker,
        associated_token::mint = mint_b,
        associated_token::authority = maker //checks if ata is derived from maker address
    )]
    pub maker_ata_b: Account<'info, TokenAccount>,

    // Programs - system prog, token prog, associated token prog
    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>, //ownership will be transferred to token program
    pub associated_token_program: Program<'info, AssociatedToken>,
}

// Take functionalities:
// 1. Deposit from taker ata b to maker ata b
// 2 .Withdraw from vault, send to taker. mint address : mint_a
// 3. Close vault
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

    pub fn deposit(&mut self) -> Result<()> {
        // token program coz spl
        let cpi_program = self.token_program.to_account_info();

        //taker_ata_b to maker_ata b
        let cpi_accounts = Transfer {
            from: self.taker_ata_b.to_account_info(),
            to: self.maker_ata_b.to_account_info(),
            authority: self.taker.to_account_info(),
        };

        let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);
        transfer(cpi_ctx, self.escrow.receive)?; //receive is defiend in escrow

        Ok(())
    }

    // transfer from vault to taker_ata_a
    pub fn withdraw_and_close(&mut self) -> Result<()> {
        // 1. Withdraw
        // token program coz spl
        let cpi_program = self.token_program.to_account_info();

        //taker_ata_b to maker_ata
        let cpi_accounts = Transfer {
            from: self.vault.to_account_info(),
            to: self.taker_ata_a.to_account_info(),
            authority: self.escrow.to_account_info(),
        };

        // Signer seeds - We need the authority of the program , program signs on behalf of PDA
        let signer_seeds = [
            &[
                b"escrow",
                self.maker.to_account_info().key.as_ref(),
                &self.escrow.seed.to_be_bytes()[..],
                &[self.escrow.bump],
            ],
        ];

        let cpi_ctx = CpiContext::new_with_signer(cpi_program, cpi_accounts, &signer_seeds);
        transfer(cpi_ctx, self.vault.amount)?; //we transfer all tokens in vault to taker

        // 2. Close Account
        let accounts = CloseAccount {
            account: self.vault.to_account_info(), //account we're closing
            destination: self.taker.to_account_info(), //where rent goes to
            authority: self.escrow.to_account_info(),
        };

        let cpi_ctx = CpiContext::new_with_signer(cpi_program, accounts, &signer_seeds);
        close_account(cpi_ctx);
        Ok(())
    }
}
