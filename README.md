# derivatives_program

This project enables the creation and management of derivatives options contracts on the Solana blockchain and was created in Solana Playground IDE. 

This program allows users to:

- **Initialize Options Contracts**: Users can create options contracts with specific terms, such as strike price, expiry date, and whether the option is a call or a put.
- **Deposit Margin**: Participants can deposit margin into an escrow account to secure their positions in the options contract.
- **Settle Contracts**: Options contracts can be settled based on the current price provided by an oracle, following predefined conditions.

The contract utilizes **Cross-Program Invocation (CPI)** to interact with the Solana Token Program for transferring tokens and relies on oracles for fetching real-time price data to determine the settlement of options contracts.

## Features

- **Options Initialization**: Create customizable options contracts on-chain.
- **Escrow Management**: Securely deposit and hold margin in an escrow account.
- **Settlement Mechanism**: Automatically settle contracts based on market data and contract conditions.


  ## Functionality
 ## 1. Initialize Option
- **Function**: `initialize_option`
- **Purpose**: Creates a new options contract.
- **Parameters**:
  - `strike_price`: The price at which the option can be exercised.
  - `expiry`: The expiry date of the option in Unix timestamp format.
  - `is_call`: Indicates if the option is a call (`true`) or a put (`false`).
  - `premium`: The amount paid by the buyer for the option.
- **Action**: 
  - Transfers the premium amount from the buyer's token account to an escrow account.
  - Initializes the option contract.

## 2. Deposit Margin
- **Function**: `deposit_margin`
- **Purpose**: Deposits margin into the escrow account.
- **Parameters**:
  - `amount`: The amount of margin to deposit.
- **Action**: 
  - Updates the margin account.
  - Transfers the specified amount from the user's token account to the escrow account.
 

## 3. Settle Option
- **Function**: `settle_option`
- **Purpose**: Settles the option based on the current price.
- **Action**: 
  - Fetches the latest price from an oracle.
  - Determines the outcome of the option.
  - Transfers the premium from the escrow account to the appropriate party.
  - Updates the option contract status.

 ## License
    This project is under MIT license. 
    
