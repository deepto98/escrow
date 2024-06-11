use anchor_lang::prelude::*;

// Escrow PDA
#[account]
pub struct Escrow {
    pub seed: u64, //to allow user to have more than 1 escrow
    pub maker: Pubkey,
    pub mint_a: Pubkey,
    pub mint_b: Pubkey,
    // pub send:u64, //amount to be sent by mint a
    pub receive: u64, //amount to be received by mint b
    pub bump: u8, //bump of escrow PDA
}

impl Space for Escrow {
    const INIT_SPACE: usize = 8 + 8 + 3 * 32 + 8 + 1;
}
