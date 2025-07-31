# Native Escrow - Solana Program Development

This repository contains both native and Anchor implementations of an escrow program, along with various testing approaches.

## Project Structure

- `/` - Native escrow program using solana-program 2.0
- `/anchor/anchor-escrow` - Anchor framework implementation
- `/native/litesvm-tests` - LiteSVM tests for the native program
- `/litesvm` - LiteSVM source code (reference)
- `/mollusk` - Mollusk testing framework (reference)

## Developer Experience Issues & Improvements

### 1. **Dependency Version Conflicts**

**Issue**: Major version conflicts when using litesvm 0.3.0 with Solana crates
- litesvm 0.3.0 requires solana-program ~2.0.5
- Trying to specify exact versions leads to dependency resolution failures
- Cargo resolves `solana-account = "2.0.5"` to 2.1.0, causing conflicts

**Current Workaround**:
```toml
# Only specify minimal dependencies and let cargo resolve
[dependencies]
litesvm = "0.3.0"
spl-token = { version = "6.0.0", features = ["no-entrypoint"] }
spl-associated-token-account = "5.0.1"
bytemuck = "1.18.0"
```

**Suggested Improvements**:
- LiteSVM should use more flexible version requirements (e.g., "2.0" instead of "~2.0.5")
- Create a compatibility matrix for Solana ecosystem crates
- Consider using workspace dependencies to ensure version consistency

### 2. **Build Tool Naming Changes**

**Issue**: Confusing transition from `cargo build-bpf` to `cargo build-sbf`
- Error message suggests `build-sbf` when `build-bpf` is used
- Documentation across the ecosystem still references the old command

**Suggested Improvements**:
- Add alias from `build-bpf` to `build-sbf` for backward compatibility
- Update all documentation and examples
- Provide clear migration guide

### 3. **Anchor Framework Setup Issues**

**Issue**: `anchor init` fails to properly install node modules
- Warning about deprecated dependencies (glob, inflight)
- Manual npm/yarn install often required after init

**Suggested Improvements**:
- Update Anchor's package.json template to use latest dependencies
- Consider using pnpm or bun for faster, more reliable installs
- Add retry logic or better error messages for npm failures

### 4. **Missing Workspace Configuration**

**Issue**: `cargo build-sbf` fails without a workspace Cargo.toml
- Error: "could not find `Cargo.toml` in ... or any parent directory"
- Not obvious that a workspace file is needed

**Suggested Improvements**:
- Better error message suggesting workspace creation
- Template command to generate minimal workspace file
- Document workspace requirements clearly

### 5. **Program ID Management**

**Issue**: Hardcoded program IDs across different files
- Need to keep IDs synchronized between lib.rs, tests, and deployment
- Easy to have mismatches

**Suggested Improvements**:
- Generate program ID from keypair at build time
- Single source of truth for program IDs
- Build script to update IDs automatically

### 6. **Test Data Setup Complexity**

**Issue**: Significant boilerplate for setting up test environments
- Creating mints, token accounts, airdrops
- Lots of repetitive code across tests

**Suggested Improvements**:
- Test fixture library for common setups
- Builder pattern for test environments
- Macro for common test patterns

### 7. **Payer Keypair Access in LiteSVM**

**Issue**: LiteSVM doesn't expose the internal payer keypair directly
- Need workaround to get funded keypair for tests
- Inconsistent with other testing frameworks

**Suggested Improvements**:
- Add `payer()` or `payer_keypair()` method to LiteSVM
- Document the airdrop keypair pattern clearly

### 8. **SPL Token Version Confusion**

**Issue**: Multiple versions of SPL Token in the ecosystem
- spl-token vs spl-token-2022
- Different feature flags needed
- Version compatibility with Solana SDK versions

**Suggested Improvements**:
- Clear compatibility matrix
- Unified token interface
- Migration guide between versions

### 9. **Error Messages and Debugging**

**Issue**: Cryptic error messages from various tools
- Transaction errors often lack context
- Build errors don't suggest solutions

**Suggested Improvements**:
- Enhanced error types with suggestions
- Better transaction simulation error details
- Debug mode with verbose logging

### 10. **Test Dependencies on Built Artifacts**

**Issue**: Tests depend on compiled .so files that must be built first
- Tests fail with "No such file or directory" if program not built
- No automatic build dependency management
- Path resolution issues between test and build directories

**Suggested Improvements**:
- Build script that compiles programs before running tests
- Better error messages indicating build requirements
- Cargo alias for "build then test" workflow
- Consider embedding test programs as bytes

### 11. **Opaque Error Messages in Tests**

**Issue**: Custom program errors show only error codes without context
- "custom program error: 0x0" doesn't indicate what went wrong
- No stack traces or line numbers from within the program
- Hard to debug without adding extensive logging

**Suggested Improvements**:
- Better error type system with descriptive messages
- Program-side logging that's captured in tests
- Source maps for BPF programs
- Test framework that can decode custom errors

### 12. **Documentation Fragmentation**

**Issue**: Documentation spread across many sources
- Solana docs, Anchor book, crate docs
- Often outdated or conflicting information
- Examples use different patterns

**Suggested Improvements**:
- Centralized, versioned documentation
- Automated testing of documentation examples
- Clear "blessed path" for common tasks

## Recommendations for New Developers

1. **Start with Anchor** if you're new to Solana development
2. **Use published crate versions** rather than git dependencies when possible
3. **Pin exact versions** in production to avoid surprises
4. **Set up a proper workspace** from the beginning
5. **Use LiteSVM for fast iteration** during development
6. **Keep reference implementations** (like this repo) for common patterns

## Future Improvements

- Create a Solana program development CLI that handles all setup
- Standardize testing patterns across the ecosystem  
- Improve error messages and debugging tools
- Create official project templates with best practices
- Build compatibility checking tools for dependencies