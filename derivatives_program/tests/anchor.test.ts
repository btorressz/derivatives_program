import * as web3 from "@solana/web3.js";
import * as anchor from "@coral-xyz/anchor";
import * as spl from "@solana/spl-token";
import { BN } from "@coral-xyz/anchor";

describe("Derivatives Program Tests", () => {
  let mint: web3.PublicKey;

  before(async () => {
    // Ensure that the provider is set correctly
    const provider = anchor.AnchorProvider.env();
    anchor.setProvider(provider);

    // Define wallet and connection
    const wallet = provider.wallet;
    const connection = provider.connection;

    if (!wallet || !wallet.publicKey) {
      throw new Error("Wallet is not defined or public key is not available.");
    }

    // Create a mint
    mint = await spl.createMint(
      connection,
      wallet.payer,
      wallet.publicKey,
      null,
      9
    );
  });

  it("Initialize Option Contract", async () => {
    const provider = anchor.AnchorProvider.env();
    const wallet = provider.wallet;
    const connection = provider.connection;

    const buyer = new web3.Keypair();
    const seller = new web3.Keypair();
    const optionContract = new web3.Keypair();

    const buyerTokenAccount = await spl.createAccount(connection, wallet.payer, mint, buyer.publicKey);
    const sellerTokenAccount = await spl.createAccount(connection, wallet.payer, mint, seller.publicKey);

    const strikePrice = new BN(1000);
    const expiry = new BN(Math.floor(Date.now() / 1000) + 86400); // 1 day from now
    const isCall = true;
    const premium = new BN(100);

    const txHash = await provider.program.methods
      .initializeOption(strikePrice, expiry, isCall, premium)
      .accounts({
        optionContract: optionContract.publicKey,
        buyer: buyer.publicKey,
        seller: seller.publicKey,
        buyerTokenAccount: buyerTokenAccount,
        escrowAccount: wallet.publicKey, // Assuming the wallet itself is the escrow account
        tokenProgram: spl.TOKEN_PROGRAM_ID,
        systemProgram: web3.SystemProgram.programId,
      })
      .signers([optionContract, buyer, seller])
      .rpc();

    console.log(`Use 'solana confirm -v ${txHash}' to see the logs`);

    await connection.confirmTransaction(txHash);

    const optionContractData = await provider.program.account.optionContract.fetch(optionContract.publicKey);
    console.log("Option Contract:", optionContractData);
    assert(optionContractData.strikePrice.eq(strikePrice));
    assert(optionContractData.expiry.eq(expiry));
    assert(optionContractData.isCall === isCall);
    assert(optionContractData.premium.eq(premium));
  });
});
