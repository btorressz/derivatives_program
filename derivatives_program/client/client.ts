import * as anchor from "@project-serum/anchor";
import * as web3 from "@solana/web3.js";
import * as spl from "@solana/spl-token";
import { BN } from "@project-serum/anchor";

// Set up the provider and program context
const provider = anchor.Provider.env();
anchor.setProvider(provider);
const program = anchor.workspace.DerivativesProgram as anchor.Program;
const connection = provider.connection;
const wallet = provider.wallet;

// Client
(async () => {
  console.log("My address:", wallet.publicKey.toString());

  // Check wallet balance
  const balance = await connection.getBalance(wallet.publicKey);
  console.log(`My balance: ${balance / web3.LAMPORTS_PER_SOL} SOL`);

  // Generate keypairs for buyer, seller, and option contract
  const buyer = new web3.Keypair();
  const seller = new web3.Keypair();
  const optionContract = new web3.Keypair();

  // Create a mint and token accounts for the buyer and seller
  const mint = await spl.createMint(
    connection,
    wallet.payer,          // The payer of the transaction
    wallet.publicKey,      // The mint authority
    null,                  // Freeze authority (set to null)
    9                      // Decimals
  );

  const buyerTokenAccount = await spl.createAccount(
    connection,
    wallet.payer,          // The payer of the transaction
    mint,                  // Mint address
    buyer.publicKey        // Token account owner
  );

  const sellerTokenAccount = await spl.createAccount(
    connection,
    wallet.payer,          // The payer of the transaction
    mint,                  // Mint address
    seller.publicKey       // Token account owner
  );

  // Define the parameters for the option contract
  const strikePrice = new BN(1000);
  const expiry = new BN(Math.floor(Date.now() / 1000) + 86400); // 1 day from now
  const isCall = true;
  const premium = new BN(100);

  // Initialize the option contract on-chain
  console.log("Initializing option contract...");
  let txHash = await program.methods
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
  console.log(`Transaction hash: ${txHash}`);

  // Fetch and display the option contract data
  let optionContractData = await program.account.optionContract.fetch(optionContract.publicKey);
  console.log("Option Contract Initialized:", optionContractData);

  // Deposit margin into the contract
  const marginAmount = new BN(500);
  console.log("Depositing margin...");

  txHash = await program.methods
    .depositMargin(marginAmount)
    .accounts({
      marginAccount: optionContract.publicKey, // Assuming option contract is the margin account
      user: buyer.publicKey,
      userTokenAccount: buyerTokenAccount,
      escrowAccount: wallet.publicKey,
      tokenProgram: spl.TOKEN_PROGRAM_ID,
    })
    .signers([buyer])
    .rpc();
  console.log(`Transaction hash: ${txHash}`);

  // Fetch and display the margin account data
  let marginAccountData = await program.account.optionContract.fetch(optionContract.publicKey);
  console.log("Margin Deposited:", marginAccountData);

  // Simulate fetching a price from an oracle (mocking the price)
  const simulatedOraclePrice = new BN(1500);
  const oracleAccount = new web3.Keypair(); // Simulated oracle account

  // Set the oracle price on-chain (if applicable in your setup)
  console.log("Simulating oracle price update...");
  // In a real-world scenario, the oracle price would be fetched from an external source

  // Settle the option contract
  console.log("Settling option contract...");

  txHash = await program.methods
    .settleOption()
    .accounts({
      optionContract: optionContract.publicKey,
      buyer: buyer.publicKey,
      seller: seller.publicKey,
      escrowAccount: wallet.publicKey,
      escrowAuthority: wallet.publicKey,
      buyerTokenAccount: buyerTokenAccount,
      sellerTokenAccount: sellerTokenAccount,
      oraclePriceAccount: oracleAccount.publicKey, // Assuming this is how oracle data is fetched
      tokenProgram: spl.TOKEN_PROGRAM_ID,
    })
    .signers([buyer])
    .rpc();
  console.log(`Transaction hash: ${txHash}`);

  // Fetch and display the option contract data after settlement
  optionContractData = await program.account.optionContract.fetch(optionContract.publicKey);
  console.log("Option Contract after Settlement:", optionContractData);
})();
