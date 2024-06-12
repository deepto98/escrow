use anchor_lang::prelude::*;

// Escrow PDA
#[account]
pub struct Escrow {
    pub seed: u64, //to allow user to have more than 1 escrow. passed from client 
    //it is passed to the seeds in in the PDA derivation in make.rs : seeds = [b"escrow", maker.key.as_ref, seed.to_le_bytes().as_ref()], 
    //   seed is fetched from instruction macro there

    pub maker: Pubkey,

    // Mints are required because the escrow needs to perform checks over ATAs and the tokens which are sent
    // like- user is depositing the mint he's saying, and taker is sending the exact tokens the maker requested 
    pub mint_a: Pubkey, // mint of token of maker. 
    pub mint_b: Pubkey,
    
    // pub send:u64, //amount to be sent by mint a
    pub receive: u64, //amount to be received by mint b
   
    pub bump: u8, //bump of escrow PDA
}

impl Space for Escrow {
    const INIT_SPACE: usize = 8 + 8 + 3 * 32 + 8 + 1;
}
