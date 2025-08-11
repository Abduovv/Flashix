import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { BlueshiftFlashLoan } from "../target/types/blueshift_flash_loan";
import {
  Keypair,
  PublicKey,
  SystemProgram,
  Transaction,
  sendAndConfirmTransaction
} from "@solana/web3.js";
import {
  TOKEN_PROGRAM_ID,
  ASSOCIATED_TOKEN_PROGRAM_ID,
  createMint,
  createAssociatedTokenAccount,
  mintTo,
  getAccount,
  getAssociatedTokenAddress
} from "@solana/spl-token";
import { expect } from "chai";

describe("flashix", () => {
  // Configure the client to use the devnet cluster.
  anchor.setProvider(anchor.AnchorProvider.env());
  const provider = anchor.getProvider();
  const program = anchor.workspace.BlueshiftFlashLoan as Program<BlueshiftFlashLoan>;

  // Test accounts
  let protocol: Keypair;
  let user: Keypair;
  let borrower: Keypair;

  // Token accounts
  let usdtMint: PublicKey;
  let lpMint: PublicKey;
  let configPda: PublicKey;
  let configBump: number;

  // Associated token accounts
  let protocolUsdtAta: PublicKey;
  let userUsdtAta: PublicKey;
  let userLpAta: PublicKey;
  let borrowerAta: PublicKey;

  before(async () => {
    // Generate test accounts
    protocol = Keypair.generate();
    user = Keypair.generate();
    borrower = Keypair.generate();

    // Airdrop SOL to accounts
    const signature1 = await provider.connection.requestAirdrop(protocol.publicKey, 10 * anchor.web3.LAMPORTS_PER_SOL);
    await provider.connection.confirmTransaction(signature1);

    const signature2 = await provider.connection.requestAirdrop(user.publicKey, 10 * anchor.web3.LAMPORTS_PER_SOL);
    await provider.connection.confirmTransaction(signature2);

    const signature3 = await provider.connection.requestAirdrop(borrower.publicKey, 10 * anchor.web3.LAMPORTS_PER_SOL);
    await provider.connection.confirmTransaction(signature3);

    // Create USDT mint
    usdtMint = await createMint(
      provider.connection,
      protocol,
      protocol.publicKey,
      null,
      6
    );

    // Find config PDA
    [configPda, configBump] = PublicKey.findProgramAddressSync(
      [Buffer.from("config")],
      program.programId
    );

    // Create LP mint
    lpMint = await createMint(
      provider.connection,
      protocol,
      configPda,
      null,
      6
    );

    // Create associated token accounts
    protocolUsdtAta = await getAssociatedTokenAddress(usdtMint, configPda);
    userUsdtAta = await getAssociatedTokenAddress(usdtMint, user.publicKey);
    userLpAta = await getAssociatedTokenAddress(lpMint, user.publicKey);
    borrowerAta = await getAssociatedTokenAddress(usdtMint, borrower.publicKey);

    // Create ATAs
    await createAssociatedTokenAccount(
      provider.connection,
      protocol,
      usdtMint,
      configPda
    );

    await createAssociatedTokenAccount(
      provider.connection,
      user,
      usdtMint,
      user.publicKey
    );

    await createAssociatedTokenAccount(
      provider.connection,
      user,
      lpMint,
      user.publicKey
    );

    // Mint USDT to user for testing
    await mintTo(
      provider.connection,
      protocol,
      usdtMint,
      userUsdtAta,
      protocol,
      1000000000 // 1000 USDT with 6 decimals
    );
  });

  it("Initialize the protocol", async () => {
    const basisPoints = 500; // 5% fee

    try {
      const tx = await program.methods
        .initialize(basisPoints)
        .accounts({
          protocol: protocol.publicKey,
          config: configPda,
          lpMint: lpMint,
          usdtMint: usdtMint,
          protocolAta: protocolUsdtAta,
          tokenProgram: TOKEN_PROGRAM_ID,
          associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
          systemProgram: SystemProgram.programId,
        })
        .signers([protocol])
        .rpc();

      console.log("Initialize transaction signature:", tx);

      // Verify config was created
      const configAccount = await program.account.config.fetch(configPda);
      expect(configAccount.basisPoints).to.equal(basisPoints);
      expect(configAccount.netDeposits).to.equal(0);
      expect(configAccount.collectedFees).to.equal(0);
      expect(configAccount.lpMint.toString()).to.equal(lpMint.toString());
      expect(configAccount.usdtMint.toString()).to.equal(usdtMint.toString());
      expect(configAccount.bumps).to.equal(configBump);

    } catch (error: any) {
      console.error("Initialize error:", error);
      throw error;
    }
  });

  it("Deposit USDT to the protocol", async () => {
    const depositAmount = 100000000; // 100 USDT

    try {
      const tx = await program.methods
        .deposit(depositAmount)
        .accounts({
          liquidator: user.publicKey,
          lpMint: lpMint,
          usdtMint: usdtMint,
          config: configPda,
          lpAta: userLpAta,
          liquidatorUsdtAta: userUsdtAta,
          usdtProtocolAta: protocolUsdtAta,
          tokenProgram: TOKEN_PROGRAM_ID,
          associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
          systemProgram: SystemProgram.programId,
        })
        .signers([user])
        .rpc();

      console.log("Deposit transaction signature:", tx);

      // Verify deposit
      const configAccount = await program.account.config.fetch(configPda);
      expect(configAccount.netDeposits).to.equal(depositAmount);

      const userLpBalance = await getAccount(provider.connection, userLpAta);
      expect(Number(userLpBalance.amount)).to.be.greaterThan(0);

    } catch (error: any) {
      console.error("Deposit error:", error);
      throw error;
    }
  });

  it("Borrow USDT via flash loan", async () => {
    const borrowAmount = 50000000; // 50 USDT

    try {
      // Create borrower ATA if it doesn't exist
      try {
        await createAssociatedTokenAccount(
          provider.connection,
          borrower,
          usdtMint,
          borrower.publicKey
        );
      } catch (e) {
        // ATA might already exist
      }

      // Create a transaction with both borrow and repay instructions
      const borrowIx = await program.methods
        .borrow(borrowAmount)
        .accounts({
          borrower: borrower.publicKey,
          config: configPda,
          mint: usdtMint,
          borrowerAta: borrowerAta,
          protocolAta: protocolUsdtAta,
          instructions: anchor.web3.SYSVAR_INSTRUCTIONS_PUBKEY,
          tokenProgram: TOKEN_PROGRAM_ID,
          associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
          systemProgram: SystemProgram.programId,
        })
        .instruction();

      const repayIx = await program.methods
        .repay()
        .accounts({
          borrower: borrower.publicKey,
          config: configPda,
          usdtMint: usdtMint,
          borrowerAta: borrowerAta,
          protocolAta: protocolUsdtAta,
          instructions: anchor.web3.SYSVAR_INSTRUCTIONS_PUBKEY,
          tokenProgram: TOKEN_PROGRAM_ID,
          associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
          systemProgram: SystemProgram.programId,
        })
        .instruction();

      const transaction = new Transaction().add(borrowIx, repayIx);

      const tx = await sendAndConfirmTransaction(
        provider.connection,
        transaction,
        [borrower]
      );

      console.log("Flash loan transaction signature:", tx);

      // Verify the flash loan was successful
      const configAccount = await program.account.config.fetch(configPda);
      expect(configAccount.netDeposits).to.equal(100000000); // Should remain the same
      expect(configAccount.collectedFees).to.be.greaterThan(0); // Should have collected fees

    } catch (error: any) {
      console.error("Flash loan error:", error);
      throw error;
    }
  });

  it("Withdraw LP tokens", async () => {
    const userLpBalance = await getAccount(provider.connection, userLpAta);
    const withdrawAmount = Number(userLpBalance.amount) / 2; // Withdraw half

    try {
      const tx = await program.methods
        .withdraw(withdrawAmount)
        .accounts({
          lp: user.publicKey,
          lpMint: lpMint,
          usdtMint: usdtMint,
          config: configPda,
          lpAta: userLpAta,
          lpUsdtAta: userUsdtAta,
          usdtProtocolAta: protocolUsdtAta,
          tokenProgram: TOKEN_PROGRAM_ID,
        })
        .signers([user])
        .rpc();

      console.log("Withdraw transaction signature:", tx);

      // Verify withdrawal
      const newUserLpBalance = await getAccount(provider.connection, userLpAta);
      expect(Number(newUserLpBalance.amount)).to.be.lessThan(Number(userLpBalance.amount));

    } catch (error: any) {
      console.error("Withdraw error:", error);
      throw error;
    }
  });

  it("Should fail when trying to borrow more than available", async () => {
    const borrowAmount = 200000000; // 200 USDT (more than available)

    try {
      const borrowIx = await program.methods
        .borrow(borrowAmount)
        .accounts({
          borrower: borrower.publicKey,
          config: configPda,
          mint: usdtMint,
          borrowerAta: borrowerAta,
          protocolAta: protocolUsdtAta,
          instructions: anchor.web3.SYSVAR_INSTRUCTIONS_PUBKEY,
          tokenProgram: TOKEN_PROGRAM_ID,
          associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
          systemProgram: SystemProgram.programId,
        })
        .instruction();

      const repayIx = await program.methods
        .repay()
        .accounts({
          borrower: borrower.publicKey,
          config: configPda,
          usdtMint: usdtMint,
          borrowerAta: borrowerAta,
          protocolAta: protocolUsdtAta,
          instructions: anchor.web3.SYSVAR_INSTRUCTIONS_PUBKEY,
          tokenProgram: TOKEN_PROGRAM_ID,
          associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
          systemProgram: SystemProgram.programId,
        })
        .instruction();

      const transaction = new Transaction().add(borrowIx, repayIx);

      await expect(
        sendAndConfirmTransaction(provider.connection, transaction, [borrower])
      ).to.be.rejected;

    } catch (error: any) {
      console.log("Expected error for borrowing too much:", error.message);
      expect(error.message).to.include("NotEnoughFunds");
    }
  });

  it("Should fail when trying to borrow without repay instruction", async () => {
    const borrowAmount = 10000000; // 10 USDT

    try {
      const tx = await program.methods
        .borrow(borrowAmount)
        .accounts({
          borrower: borrower.publicKey,
          config: configPda,
          mint: usdtMint,
          borrowerAta: borrowerAta,
          protocolAta: protocolUsdtAta,
          instructions: anchor.web3.SYSVAR_INSTRUCTIONS_PUBKEY,
          tokenProgram: TOKEN_PROGRAM_ID,
          associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
          systemProgram: SystemProgram.programId,
        })
        .signers([borrower])
        .rpc();

      // This should fail
      expect.fail("Should have thrown an error");

    } catch (error: any) {
      console.log("Expected error for missing repay instruction:", error.message);
      expect(error.message).to.include("MissingRepayIx");
    }
  });
});
