use anchor_lang::prelude::*;
use anchor_spl::token::{self, Token, TokenAccount, Transfer, Mint};
use solana_program::program::invoke;

declare_id!("Ei8nkgALF3Yvyn1aT2gqerk1nhjY4Wm8driK9fZJ3uZq");

#[program]
pub mod derivatives_program {
    use super::*;

    pub fn initialize_option(
        ctx: Context<InitializeOption>,
        strike_price: u64,
        expiry: i64,
        is_call: bool,
        premium: u64,
    ) -> Result<()> {
        let option_contract = &mut ctx.accounts.option_contract;
        option_contract.buyer = *ctx.accounts.buyer.key;
        option_contract.seller = *ctx.accounts.seller.key;
        option_contract.strike_price = strike_price;
        option_contract.expiry = expiry;
        option_contract.is_call = is_call;
        option_contract.premium = premium;
        option_contract.exercised = false;

        let cpi_accounts = Transfer {
            from: ctx.accounts.buyer_token_account.to_account_info(),
            to: ctx.accounts.escrow_account.to_account_info(),
            authority: ctx.accounts.buyer.to_account_info(),
        };
        let cpi_program = ctx.accounts.token_program.to_account_info();
        let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);
        token::transfer(cpi_ctx, premium)?;

        emit!(OptionCreated {
            buyer: *ctx.accounts.buyer.key,
            seller: *ctx.accounts.seller.key,
            strike_price,
            expiry,
        });
        Ok(())
    }

    pub fn deposit_margin(ctx: Context<DepositMargin>, amount: u64) -> Result<()> {
        let margin_account = &mut ctx.accounts.margin_account;
        margin_account.amount += amount;

        let cpi_accounts = Transfer {
            from: ctx.accounts.user_token_account.to_account_info(),
            to: ctx.accounts.escrow_account.to_account_info(),
            authority: ctx.accounts.user.to_account_info(),
        };
        let cpi_program = ctx.accounts.token_program.to_account_info();
        let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);
        token::transfer(cpi_ctx, amount)?;

        emit!(MarginDeposited {
            user: *ctx.accounts.user.key,
            amount,
        });
        Ok(())
    }

    pub fn settle_option(ctx: Context<SettleOption>) -> Result<()> {
        let buyer_token_account_info = ctx.accounts.buyer_token_account.to_account_info();
        let seller_token_account_info = ctx.accounts.seller_token_account.to_account_info();
        let oracle_price_account_info = ctx.accounts.oracle_price_account.to_account_info();

        let latest_price = fetch_price_from_oracle(oracle_price_account_info)?;

        if ctx.accounts.option_contract.is_call {
            if latest_price > ctx.accounts.option_contract.strike_price {
                transfer_from_escrow(
                    &ctx.accounts.escrow_account,
                    &ctx.accounts.escrow_authority,
                    &ctx.accounts.token_program,
                    &buyer_token_account_info,
                    ctx.accounts.option_contract.premium,
                    &ctx.accounts.option_contract,
                )?;
            } else {
                transfer_from_escrow(
                    &ctx.accounts.escrow_account,
                    &ctx.accounts.escrow_authority,
                    &ctx.accounts.token_program,
                    &seller_token_account_info,
                    ctx.accounts.option_contract.premium,
                    &ctx.accounts.option_contract,
                )?;
            }
        } else {
            if latest_price < ctx.accounts.option_contract.strike_price {
                transfer_from_escrow(
                    &ctx.accounts.escrow_account,
                    &ctx.accounts.escrow_authority,
                    &ctx.accounts.token_program,
                    &buyer_token_account_info,
                    ctx.accounts.option_contract.premium,
                    &ctx.accounts.option_contract,
                )?;
            } else {
                transfer_from_escrow(
                    &ctx.accounts.escrow_account,
                    &ctx.accounts.escrow_authority,
                    &ctx.accounts.token_program,
                    &seller_token_account_info,
                    ctx.accounts.option_contract.premium,
                    &ctx.accounts.option_contract,
                )?;
            }
        }

        let option_contract = &mut ctx.accounts.option_contract;

        let current_timestamp = Clock::get()?.unix_timestamp;
        require!(
            current_timestamp >= option_contract.expiry,
            CustomError::OptionNotExpired
        );

        option_contract.exercised = true;

        emit!(OptionSettled {
            option_contract: ctx.accounts.option_contract.key(),
            buyer_profit: latest_price,
            seller_profit: latest_price,
        });

        Ok(())
    }
}

// Cross-Program Invocation (CPI) for fetching price from oracle
fn fetch_price_from_oracle(oracle_account: AccountInfo) -> Result<u64> {
    let price_data = oracle_account.try_borrow_data()?;
    let price = u64::from_le_bytes(price_data[..8].try_into().unwrap());
    Ok(price)
}

// Transfer funds from the escrow to the designated account
fn transfer_from_escrow<'info>(
    escrow_account: &Account<'info, TokenAccount>,
    escrow_authority: &AccountInfo<'info>,
    token_program: &Program<'info, Token>,
    recipient: &AccountInfo<'info>,
    premium: u64,
    option_contract: &Account<'info, OptionContract>,
) -> Result<()> {
    let cpi_accounts = Transfer {
        from: escrow_account.to_account_info(),
        to: recipient.clone(),
        authority: escrow_authority.clone(),
    };
    let cpi_program = token_program.to_account_info();

    let option_contract_key = option_contract.key();
    let seeds = &[b"escrow", option_contract_key.as_ref()];
    let signer_seeds = &[&seeds[..]];

    let cpi_ctx = CpiContext::new_with_signer(cpi_program, cpi_accounts, signer_seeds);
    token::transfer(cpi_ctx, premium)?;
    Ok(())
}

// Account structs

#[derive(Accounts)]
pub struct InitializeOption<'info> {
    #[account(init, payer = buyer, space = 8 + OptionContract::MAX_SIZE)]
    pub option_contract: Account<'info, OptionContract>,
    #[account(mut)]
    pub buyer: Signer<'info>,
    #[account(mut)]
    pub seller: AccountInfo<'info>,
    #[account(mut)]
    pub buyer_token_account: Account<'info, TokenAccount>,
    #[account(mut)]
    pub escrow_account: Account<'info, TokenAccount>, // PDA for holding escrow funds
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct DepositMargin<'info> {
    #[account(mut)]
    pub margin_account: Account<'info, MarginAccount>,
    #[account(mut)]
    pub user: Signer<'info>,
    #[account(mut)]
    pub user_token_account: Account<'info, TokenAccount>,
    #[account(mut)]
    pub escrow_account: Account<'info, TokenAccount>, // Escrow account
    pub token_program: Program<'info, Token>,
}

#[derive(Accounts)]
pub struct SettleOption<'info> {
    #[account(mut, has_one = buyer, has_one = seller)]
    pub option_contract: Account<'info, OptionContract>,
    #[account(mut)]
    pub buyer: Signer<'info>,
    #[account(mut)]
    pub seller: Signer<'info>,
    #[account(mut)]
    pub escrow_account: Account<'info, TokenAccount>,
    #[account(mut)]
    pub escrow_authority: AccountInfo<'info>, // PDA for escrow account signer
    #[account(mut)]
    pub buyer_token_account: Account<'info, TokenAccount>,
    #[account(mut)]
    pub seller_token_account: Account<'info, TokenAccount>,
    #[account()]
    pub oracle_price_account: AccountInfo<'info>, // Price feed account from oracle
    pub token_program: Program<'info, Token>,
}

// Data structures

#[account]
#[derive(Default)]
pub struct OptionContract {
    pub buyer: Pubkey,      // Buyer of the option
    pub seller: Pubkey,     // Seller of the option
    pub strike_price: u64,  // Strike price of the option
    pub expiry: i64,        // Expiry date in Unix timestamp
    pub is_call: bool,      // True for call, false for put
    pub premium: u64,       // Premium paid by the buyer
    pub exercised: bool,    // Whether the option has been exercised
}

impl OptionContract {
    pub const MAX_SIZE: usize = 32 + 32 + 8 + 8 + 1 + 8 + 1; // Calculate based on field sizes
}

#[account]
#[derive(Default)]
pub struct MarginAccount {
    pub owner: Pubkey,      // Owner of the margin account
    pub amount: u64,        // Margin amount
}

// Events
#[event]
pub struct OptionCreated {
    pub buyer: Pubkey,
    pub seller: Pubkey,
    pub strike_price: u64,
    pub expiry: i64,
}

#[event]
pub struct OptionSettled {
    pub option_contract: Pubkey,
    pub buyer_profit: u64,
    pub seller_profit: u64,
}

#[event]
pub struct MarginDeposited {
    pub user: Pubkey,
    pub amount: u64,
}

// Error handling
#[error_code]
pub enum CustomError {
    #[msg("The option has not expired yet.")]
    OptionNotExpired,
    #[msg("Unauthorized access to settle this option.")]
    UnauthorizedAccess,
}
