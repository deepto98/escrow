import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { TOKEN_PROGRAM_ID, createMint, getAssociatedTokenAddressSync, getOrCreateAssociatedTokenAccount, mintTo } from "@solana/spl-token";
import { Escrow } from "../target/types/escrow";
import NodeWallet from "@coral-xyz/anchor/dist/cjs/nodewallet";
import { randomBytes } from "crypto";
import { ASSOCIATED_PROGRAM_ID } from "@coral-xyz/anchor/dist/cjs/utils/token";

describe("escrow", () => {
  // Configure the client to use the local cluster.
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);

  const program = anchor.workspace.Escrow as Program<Escrow>;

  const wallet = provider.wallet as NodeWallet;

  // Generate maker and taker keypairs
  const maker = anchor.web3.Keypair.generate();
  const taker = anchor.web3.Keypair.generate();

  // Declare mint accounts and ATAs 
  let mintA: anchor.web3.PublicKey;
  let mintB: anchor.web3.PublicKey;

  let makerAtaA: anchor.web3.PublicKey;
  let makerAtaB: anchor.web3.PublicKey;

  let takerAtaA: anchor.web3.PublicKey;
  let takerAtaB: anchor.web3.PublicKey;

  // seed for escrow
  const seed = new anchor.BN(randomBytes(8));
  // Create PDA for escrow
  const escrow = anchor.web3.PublicKey.findProgramAddressSync(
    [Buffer.from("escrow"), maker.publicKey.toBuffer(), seed.toArrayLike(Buffer, "le", 8)],
    program.programId
  )[0];

  // 1.  First airdrop 1 sol to both maker and taker
  it("Aidrop Sol to maker and taker", async () => {
    const tx = await provider.connection.requestAirdrop(maker.publicKey, 1000000000);
    await provider.connection.confirmTransaction(tx);
    console.log("Maker balance: ", await provider.connection.getBalance(maker.publicKey));

    const tx2 = await provider.connection.requestAirdrop(taker.publicKey, 1000000000);
    await provider.connection.confirmTransaction(tx2);
    console.log("Taker balance: ", await provider.connection.getBalance(taker.publicKey));
  });

  // 2. Create mint accounts
  it("Create Tokens and Mint Tokens", async () => {
    mintA = await createMint(provider.connection, wallet.payer, provider.publicKey, provider.publicKey, 6);
    console.log("Mint A: ", mintA.toBase58());
    mintB = await createMint(provider.connection, wallet.payer, provider.publicKey, provider.publicKey, 6);
    console.log("Mint B: ", mintB.toBase58());

    // Create ATAs
    makerAtaA = (await getOrCreateAssociatedTokenAccount(provider.connection, wallet.payer, mintA, maker.publicKey)).address;
    console.log("makerAtaA: ", makerAtaA);

    makerAtaB = (await getOrCreateAssociatedTokenAccount(provider.connection, wallet.payer, mintB, maker.publicKey)).address;
    console.log("makerAtaB: ", makerAtaB);

    takerAtaA = (await getOrCreateAssociatedTokenAccount(provider.connection, wallet.payer, mintA, taker.publicKey)).address;
    console.log("takerAtaA: ", takerAtaA);


    takerAtaB = (await getOrCreateAssociatedTokenAccount(provider.connection, wallet.payer, mintB, taker.publicKey)).address;
    console.log("takerAtaB: ", takerAtaB);
 

    // mint tokens to maker ATAs
    await mintTo(provider.connection, wallet.payer, mintA, makerAtaA, provider.publicKey, 1_000_000_0);
    console.log("1");
    await mintTo(provider.connection, wallet.payer, mintB, makerAtaB, provider.publicKey, 1_000_000_0);
    console.log("2");

    // mint tokens to taker ATAs
    await mintTo(provider.connection, wallet.payer, mintA, takerAtaA, provider.publicKey, 1_000_000_0);
    console.log("3");

    await mintTo(provider.connection, wallet.payer, mintB, takerAtaB, provider.publicKey, 1_000_000_0);
    console.log("4");

  });

  it("Is initialized!", async () => {
    // Add your test here.

    // Get Vault ATA
    const vault = getAssociatedTokenAddressSync(mintA, escrow, true);

    const tx = await program.methods
      //Deposit 1 mint a and receive 1 mint b
      .make(seed, new anchor.BN(1_000_000), new anchor.BN(1_000_000))
      .accountsPartial({
        maker: maker.publicKey,
        mintA,
        mintB,
        makerAtaA: makerAtaA,
        escrow,
        vault,
        associatedTokenProgram: ASSOCIATED_PROGRAM_ID,
        tokenProgram: TOKEN_PROGRAM_ID,
        systemProgram: anchor.web3.SystemProgram.programId,
      })
      .signers([maker])//maker is signer
      .rpc();
    console.log("Your transaction signature", tx);
  });
});