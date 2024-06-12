use anchor_lang::{ prelude::*, solana_program::instruction };
use anchor_spl::{
    associated_token::AssociatedToken,
    token::{ transfer, Mint, Token, TokenAccount, Transfer },
};

use crate::state::Escrow;

#[derive(Accounts)]
#[instruction(seed:u64)] //seed will be passed from client, we use instruction to use it within context
pub struct Make<'info> {
    #[account(mut)] //mut because maker  (is payer) for initializing the account - sol change
    pub maker: Signer<'info>, //person initializing escrow and paying fees and signing SPL tokens, has to be a signer

    pub mint_a: Account<'info, Mint>, //derive PDAs. to be stored in escrow
    pub mint_b: Account<'info, Mint>, //to be stored in escrow

    #[account(
        init,
        payer = maker,
        // Seeds - if we only used [b"escrow", maker.key.as_ref], only one escrow would be possible
        // as all PDAs derived used the seed would be the same
        // by passing seed, we allow multiple escrows per user. seed is fetched from instruction macro here
        seeds = [b"escrow", maker.key.as_ref(), seed.to_le_bytes().as_ref()],
        bump, //initialize with canonical bump
        space = Escrow::INIT_SPACE //Space is what the payer pays
    )]
    pub escrow: Account<'info, Escrow>,

    // ATAs required - maker and vault

    // Maker ATA for token A - Tokens from Mint A will be transferred from Maker ATA A to the vault
    // An ATA is derived from a wallet address(maker) and a mint address (mint_a)
    #[account( 
        //no init because maker_ata_a should already exist 
        mut, //mut since we transfer tokens from the ata 
        associated_token :: mint = mint_a, //ensuring ata is derived from Mint A
        associated_token :: authority = maker,//checks if ata is derived from maker address
    )]
    pub maker_ata_a: Account<'info, TokenAccount>,

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
    // Initialize   escrow account with data
    pub fn init(&mut self, seed: u64, receive: u64, bumps: &MakeBumps) -> Result<()> {
        // set_inner helps us init the inner account of the Escrow. 
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

    // Function for maker to deposit from maker_ata_a to vault
    pub fn deposit(&mut self, deposit: u64) -> Result<()> {
        // For transferring sol, SystemProgram is target of CPI. Because System prog owns system account, which can transfer sol
        // For transferring SPL tokens,TokenProgram is the target. Because tokens are possesed by ATAs which are owned by Token Program
        let cpi_program = self.token_program.to_account_info();

        let cpi_accounts = Transfer {
            from: self.maker_ata_a.to_account_info(),
            to: self.vault.to_account_info(),
            authority: self.maker.to_account_info(), //Auth required for ATA to ATA CPI. Auth is of the from acc
        };

        let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);
        transfer(cpi_ctx, deposit)?;

        Ok(())
    }
}

// After deposit, the maker has deposited tokens   in the vault and said he wants x amount of mint b.
// Now a taker has to accept the deal
// transfer of mint b tokens from taker ata  to maker ata
// then transfer tokens from vault to taker
// then we close the state account
