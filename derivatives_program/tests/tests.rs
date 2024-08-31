use anchor_lang::prelude::*;
use anchor_lang::system_program;
use anchor_spl::token::{self, Mint, TokenAccount, Token};
use solana_program::system_instruction;
use solana_program_test::*;
use solana_sdk::{signature::Keypair, transaction::Transaction};
use derivatives_program::program::DerivativesProgram;
use derivatives_program::accounts::*;
use derivatives_program::*;

#[tokio::test]
async fn test_initialize_option() {
    let mut program_test = ProgramTest::new(
        "derivatives_program",     // Program name
        DerivativesProgram::id(),  // Program ID
        processor!(derivatives_program::DerivativesProgram::process_instruction), // Entry point
    );

    // Add necessary programs like Token, System, etc.
    program_test.add_program("spl_token", spl_token::id(), None);

    let (mut banks_client, payer, recent_blockhash) = program_test.start().await;

    // Create accounts for buyer, seller, and other necessary accounts
    let buyer = Keypair::new();
    let seller = Keypair::new();

    // Create a mint and a token account for the buyer
    let mint = Keypair::new();
    let buyer_token_account = Keypair::new();

    let token_program = spl_token::id();
    
    // Fund accounts
    let tx = Transaction::new_signed_with_payer(
        &[system_instruction::create_account(
            &payer.pubkey(),
            &mint.pubkey(),
            1_000_000,  // Allocate enough lamports for rent-exemption
            Mint::LEN as u64,
            &token_program,
        )],
        Some(&payer.pubkey()),
        &[&payer, &mint],
        recent_blockhash,
    );
    banks_client.process_transaction(tx).await.unwrap();

    // Initialize an option contract
    let strike_price = 1000;
    let expiry = 1650000000; // Some arbitrary future timestamp
    let is_call = true;
    let premium = 100;

    let option_contract = Keypair::new();

    let tx = Transaction::new_signed_with_payer(
        &[instruction::initialize_option(
            &DerivativesProgram::id(),
            &option_contract.pubkey(),
            &buyer.pubkey(),
            &seller.pubkey(),
            &mint.pubkey(),
            strike_price,
            expiry,
            is_call,
            premium,
        )],
        Some(&payer.pubkey()),
        &[&payer, &option_contract],
        recent_blockhash,
    );
    banks_client.process_transaction(tx).await.unwrap();

    // Verify that the option contract was initialized
    let option_account = banks_client.get_account(option_contract.pubkey()).await.unwrap();
    assert!(option_account.is_some(), "Option contract not initialized");
}

#[tokio::test]
async fn test_deposit_margin() {
    let mut program_test = ProgramTest::new(
        "derivatives_program",
        DerivativesProgram::id(),
        processor!(derivatives_program::DerivativesProgram::process_instruction),
    );

    // Set up program test client
    let (mut banks_client, payer, recent_blockhash) = program_test.start().await;

    let user = Keypair::new();
    let mint = Keypair::new();
    let user_token_account = Keypair::new();
    let margin_account = Keypair::new();

    let token_program = spl_token::id();

    // Fund the user with lamports
    let tx = Transaction::new_signed_with_payer(
        &[system_instruction::create_account(
            &payer.pubkey(),
            &mint.pubkey(),
            1_000_000,  // Rent exemption
            Mint::LEN as u64,
            &token_program,
        )],
        Some(&payer.pubkey()),
        &[&payer, &mint],
        recent_blockhash,
    );
    banks_client.process_transaction(tx).await.unwrap();

    // Deposit margin
    let margin_amount = 500;

    let tx = Transaction::new_signed_with_payer(
        &[instruction::deposit_margin(
            &DerivativesProgram::id(),
            &margin_account.pubkey(),
            &user.pubkey(),
            &user_token_account.pubkey(),
            margin_amount,
        )],
        Some(&payer.pubkey()),
        &[&payer, &margin_account],
        recent_blockhash,
    );
    banks_client.process_transaction(tx).await.unwrap();

    // Verify that the margin was deposited
    let margin_account_data = banks_client.get_account(margin_account.pubkey()).await.unwrap();
    assert!(margin_account_data.is_some(), "Margin account not created");
    // You would also assert margin amount, owner, etc.
}

#[tokio::test]
async fn test_settle_option() {
    let mut program_test = ProgramTest::new(
        "derivatives_program",
        DerivativesProgram::id(),
        processor!(derivatives_program::DerivativesProgram::process_instruction),
    );

    let (mut banks_client, payer, recent_blockhash) = program_test.start().await;

    // Create accounts for buyer, seller, mint, and token accounts
    let buyer = Keypair::new();
    let seller = Keypair::new();
    let option_contract = Keypair::new();
    let buyer_token_account = Keypair::new();
    let seller_token_account = Keypair::new();
    let oracle_account = Keypair::new(); // Simulate an oracle account
    let escrow_account = Keypair::new();
    let escrow_authority = Keypair::new();

    let mint = Keypair::new();
    let token_program = spl_token::id();

    // Simulate fetching the price from an oracle
    let simulated_oracle_price = 1500;

    // Initialize the option contract and settle it

    let tx = Transaction::new_signed_with_payer(
        &[instruction::settle_option(
            &DerivativesProgram::id(),
            &option_contract.pubkey(),
            &buyer.pubkey(),
            &seller.pubkey(),
            &escrow_account.pubkey(),
            &escrow_authority.pubkey(),
            &buyer_token_account.pubkey(),
            &seller_token_account.pubkey(),
            &oracle_account.pubkey(),
            token_program,
        )],
        Some(&payer.pubkey()),
        &[&payer],
        recent_blockhash,
    );
    banks_client.process_transaction(tx).await.unwrap();

    // Verify that the option was settled
    let option_account_data = banks_client.get_account(option_contract.pubkey()).await.unwrap();
    assert!(option_account_data.is_some(), "Option not settled correctly");

    // You can also assert further based on contract status (like exercised state)
}
