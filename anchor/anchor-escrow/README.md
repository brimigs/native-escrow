# Anchor Escrow Program

An Anchor framework implementation of a token escrow program on Solana.

## Overview

This escrow program allows trustless token exchanges between two parties using the Anchor framework, which provides a more ergonomic and safer way to write Solana programs.

## Instructions

### Make
Creates a new escrow by:
- Creating an escrow account with trade details
- Initializing a vault token account
- Transferring tokens from maker to vault

**Parameters:**
- `seed`: Unique identifier for the escrow
- `deposit`: Amount of tokens to deposit
- `receive`: Amount of tokens expected in return

### Take
Completes the escrow exchange by:
- Transferring requested tokens from taker to maker
- Transferring deposited tokens from vault to taker
- Closing the vault and escrow accounts

### Refund
Cancels the escrow by:
- Returning deposited tokens from vault to maker
- Closing the vault and escrow accounts

## Project Structure

```
programs/anchor-escrow/
├── src/
│   ├── lib.rs           # Program entry point
│   ├── state/
│   │   ├── mod.rs       # State module exports
│   │   └── escrow.rs    # Escrow account structure
│   └── contexts/
│       ├── mod.rs       # Context module exports
│       ├── make.rs      # Make instruction context
│       ├── take.rs      # Take instruction context
│       └── refund.rs    # Refund instruction context
└── Cargo.toml           # Dependencies
```

## Building

```bash
anchor build
```

## Testing

```bash
anchor test
```

## Key Features

- Uses Anchor's account validation and constraint system
- Supports SPL Token and Token-2022
- Automatic PDA derivation and validation
- Safe account initialization and closing
- Type-safe instruction handling