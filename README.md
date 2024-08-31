# derivatives_program

This project enables the creation and management of derivatives options contracts on the Solana blockchain. It allows users to:

- **Initialize Options Contracts**: Users can create options contracts with specific terms, such as strike price, expiry date, and whether the option is a call or a put.
- **Deposit Margin**: Participants can deposit margin into an escrow account to secure their positions in the options contract.
- **Settle Contracts**: Options contracts can be settled based on the current price provided by an oracle, following predefined conditions.

The contract utilizes **Cross-Program Invocation (CPI)** to interact with the Solana Token Program for transferring tokens and relies on oracles for fetching real-time price data to determine the settlement of options contracts.

## Features

- **Options Initialization**: Create customizable options contracts on-chain.
- **Escrow Management**: Securely deposit and hold margin in an escrow account.
- **Settlement Mechanism**: Automatically settle contracts based on market data and contract conditions.
