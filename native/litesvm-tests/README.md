# LiteSVM Tests for Native Escrow Program

This crate contains integration tests for the native escrow program using LiteSVM, a fast and lightweight library for testing Solana programs.

## Overview

LiteSVM creates an in-process Solana VM optimized for program developers, making it much faster to run tests compared to alternatives like `solana-program-test` or `solana-test-validator`.

## Tests

The test suite includes three main test cases:

### 1. `test_make_escrow`
Tests the creation of a new escrow:
- Creates mints and token accounts
- Deposits tokens into the escrow vault
- Verifies the escrow account is properly initialized
- Checks that tokens are transferred to the vault

### 2. `test_take_escrow`
Tests the completion of an escrow exchange:
- Sets up an escrow with the make instruction
- Executes the take instruction as a different user
- Verifies token transfers between parties
- Confirms the escrow and vault accounts are closed

### 3. `test_refund_escrow`
Tests the cancellation of an escrow:
- Creates an escrow
- Executes the refund instruction
- Verifies tokens are returned to the maker
- Confirms the escrow and vault accounts are closed

## Running the Tests

First, build the native escrow program:
```bash
cd ../../  # Go to native-escrow root
cargo build-sbf
```

Then run the tests:
```bash
cargo test
```

## Test Structure

Each test follows this pattern:
1. Initialize LiteSVM instance
2. Load the escrow program
3. Create necessary accounts (mints, token accounts)
4. Execute transactions
5. Verify the results

The tests use helper functions for common operations:
- `create_mint()` - Creates a new SPL token mint
- `create_token_account()` - Creates an associated token account
- `mint_tokens()` - Mints tokens to an account
- `payer_keypair()` - Gets a funded keypair for paying transaction fees