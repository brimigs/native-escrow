import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { AnchorEscrow } from "../target/types/anchor_escrow";
import { 
  Keypair, 
  PublicKey, 
  SystemProgram, 
  Transaction,
  sendAndConfirmTransaction,
  LAMPORTS_PER_SOL
} from "@solana/web3.js";
import {
  TOKEN_PROGRAM_ID,
  TOKEN_2022_PROGRAM_ID,
  createMint,
  getAssociatedTokenAddress,
  mintTo,
  getAccount,
  createAssociatedTokenAccount,
} from "@solana/spl-token";
import { assert } from "chai";

describe("anchor-escrow", () => {
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);

  const program = anchor.workspace.AnchorEscrow as Program<AnchorEscrow>;
  
  const maker = Keypair.generate();
  const taker = Keypair.generate();
  const seed = new anchor.BN(Date.now());
  
  let mintA: PublicKey;
  let mintB: PublicKey;
  let makerAtaA: PublicKey;
  let makerAtaB: PublicKey;
  let takerAtaA: PublicKey;
  let takerAtaB: PublicKey;
  let escrow: PublicKey;
  let vault: PublicKey;

  const deposit = new anchor.BN(1000);
  const receive = new anchor.BN(2000);

  before(async () => {
    // Airdrop SOL to maker and taker
    await provider.connection.confirmTransaction(
      await provider.connection.requestAirdrop(maker.publicKey, 10 * LAMPORTS_PER_SOL)
    );
    await provider.connection.confirmTransaction(
      await provider.connection.requestAirdrop(taker.publicKey, 10 * LAMPORTS_PER_SOL)
    );

    // Create mints
    mintA = await createMint(
      provider.connection,
      maker,
      maker.publicKey,
      null,
      6,
      undefined,
      undefined,
      TOKEN_PROGRAM_ID
    );

    mintB = await createMint(
      provider.connection,
      taker,
      taker.publicKey,
      null,
      6,
      undefined,
      undefined,
      TOKEN_PROGRAM_ID
    );

    // Create ATAs
    makerAtaA = await createAssociatedTokenAccount(
      provider.connection,
      maker,
      mintA,
      maker.publicKey,
      undefined,
      TOKEN_PROGRAM_ID
    );

    takerAtaB = await createAssociatedTokenAccount(
      provider.connection,
      taker,
      mintB,
      taker.publicKey,
      undefined,
      TOKEN_PROGRAM_ID
    );

    // Mint tokens
    await mintTo(
      provider.connection,
      maker,
      mintA,
      makerAtaA,
      maker,
      10000,
      [],
      undefined,
      TOKEN_PROGRAM_ID
    );

    await mintTo(
      provider.connection,
      taker,
      mintB,
      takerAtaB,
      taker,
      10000,
      [],
      undefined,
      TOKEN_PROGRAM_ID
    );

    // Derive PDAs
    [escrow] = PublicKey.findProgramAddressSync(
      [Buffer.from("escrow"), maker.publicKey.toBuffer(), seed.toArrayLike(Buffer, "le", 8)],
      program.programId
    );

    [vault] = PublicKey.findProgramAddressSync(
      [Buffer.from("vault"), escrow.toBuffer()],
      program.programId
    );
  });

  it("Make escrow", async () => {
    await program.methods
      .make(seed, deposit, receive)
      .accounts({
        maker: maker.publicKey,
        mintA,
        mintB,
        makerAtaA,
        escrow,
        vault,
        tokenProgram: TOKEN_PROGRAM_ID,
      })
      .signers([maker])
      .rpc();

    // Verify escrow account
    const escrowAccount = await program.account.escrow.fetch(escrow);
    assert.equal(escrowAccount.maker.toString(), maker.publicKey.toString());
    assert.equal(escrowAccount.mintA.toString(), mintA.toString());
    assert.equal(escrowAccount.mintB.toString(), mintB.toString());
    assert.equal(escrowAccount.receive.toString(), receive.toString());
    assert.equal(escrowAccount.seed.toString(), seed.toString());

    // Verify vault has tokens
    const vaultAccount = await getAccount(provider.connection, vault, undefined, TOKEN_PROGRAM_ID);
    assert.equal(vaultAccount.amount.toString(), deposit.toString());
  });

  it("Take escrow", async () => {
    takerAtaA = await getAssociatedTokenAddress(mintA, taker.publicKey, false, TOKEN_PROGRAM_ID);
    makerAtaB = await getAssociatedTokenAddress(mintB, maker.publicKey, false, TOKEN_PROGRAM_ID);

    await program.methods
      .take()
      .accounts({
        taker: taker.publicKey,
        maker: maker.publicKey,
        mintA,
        mintB,
        takerAtaA,
        takerAtaB,
        makerAtaB,
        escrow,
        vault,
        tokenProgram: TOKEN_PROGRAM_ID,
      })
      .signers([taker])
      .rpc();

    // Verify taker received tokens
    const takerAtaAAccount = await getAccount(provider.connection, takerAtaA, undefined, TOKEN_PROGRAM_ID);
    assert.equal(takerAtaAAccount.amount.toString(), deposit.toString());

    // Verify maker received tokens
    const makerAtaBAccount = await getAccount(provider.connection, makerAtaB, undefined, TOKEN_PROGRAM_ID);
    assert.equal(makerAtaBAccount.amount.toString(), receive.toString());

    // Verify escrow closed
    try {
      await program.account.escrow.fetch(escrow);
      assert.fail("Escrow should be closed");
    } catch (e) {
      assert.ok(e.toString().includes("Account does not exist"));
    }
  });

  it("Refund escrow", async () => {
    // Create a new escrow for refund test
    const refundSeed = new anchor.BN(Date.now() + 1000);
    const [refundEscrow] = PublicKey.findProgramAddressSync(
      [Buffer.from("escrow"), maker.publicKey.toBuffer(), refundSeed.toArrayLike(Buffer, "le", 8)],
      program.programId
    );
    const [refundVault] = PublicKey.findProgramAddressSync(
      [Buffer.from("vault"), refundEscrow.toBuffer()],
      program.programId
    );

    // Make escrow
    await program.methods
      .make(refundSeed, deposit, receive)
      .accounts({
        maker: maker.publicKey,
        mintA,
        mintB,
        makerAtaA,
        escrow: refundEscrow,
        vault: refundVault,
        tokenProgram: TOKEN_PROGRAM_ID,
      })
      .signers([maker])
      .rpc();

    // Get initial balance
    const initialBalance = (await getAccount(provider.connection, makerAtaA, undefined, TOKEN_PROGRAM_ID)).amount;

    // Refund escrow
    await program.methods
      .refund()
      .accounts({
        maker: maker.publicKey,
        mintA,
        makerAtaA,
        escrow: refundEscrow,
        vault: refundVault,
        tokenProgram: TOKEN_PROGRAM_ID,
      })
      .signers([maker])
      .rpc();

    // Verify maker got tokens back
    const finalBalance = (await getAccount(provider.connection, makerAtaA, undefined, TOKEN_PROGRAM_ID)).amount;
    assert.equal((finalBalance - initialBalance).toString(), deposit.toString());

    // Verify escrow closed
    try {
      await program.account.escrow.fetch(refundEscrow);
      assert.fail("Escrow should be closed");
    } catch (e) {
      assert.ok(e.toString().includes("Account does not exist"));
    }
  });
});