# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this directory.

## Crate-specific CLAUDE.md files
Always consult CLAUDE.md files in sub-crates. Instructions in local CLAUDE.md files override instructions
in this file when they are in conflict.

# Individual Preferences
Individual preferences supercede and extend project preferences:
- @CLAUDE.local.md

## Essential Development Commands

### Building and Installation

```bash
# Build a specific crate. Generally don't need to do release build.
cargo build -p rtd-core

# Check code without building (preferred)
cargo check
```

### Testing

```bash
# Run e2e tests. simtests must be run with `cargo simtest` to avoid false negatives
cargo simtest -p rtd-e2e-tests

# Run Rust unittests. skip simulation tests as they may cause false negatives with `cargo nextest`
RTD_SKIP_SIMTESTS=1 cargo nextest run
```

**Important Notes for Testing:**
- When compiling or running tests in this repository, set timeout limits to at least 10 minutes due to the large codebase size
- For faster iteration, use -p to select only the most relevant packages for testing. Use multiple `-p` flags if necessary, e.g. `cargo nextest run -p rtd-types -p rtd-core`
- Use `cargo nextest --lib` to run only library tests and skip integration tests for faster feedback
- Consult crate-specific CLAUDE.md files for instructions on which tests to run, when changing files in those crates

### Linting and Formatting

```bash
# Formats & lints all Rust & Move, run before commit:
./scripts/lint.sh

# Alternatively, run individual lints:
cargo fmt --all -- --check
cargo xclippy
```

`cargo xclippy does not recognize -p option` - This is a known issue with some clippy command variations

## High-Level Architecture

### Core Components Structure

```
rtd/
├── crates/                   # Main Rust crates
│   ├── rtd-core/             # Core blockchain logic
│   ├── rtd-node/             # Validator node implementation
│   ├── rtd-framework/        # Move system packages & stdlib
│   ├── rtd-types/            # Core type definitions
│   ├── rtd-json-rpc/         # JSON-RPC API server
│   ├── rtd-graphql-rpc/      # GraphQL API server
│   └── rtd-indexer-alt/      # Blockchain data indexer
├── consensus/                # Consensus mechanism (Mysticeti)
├── rtd-execution/            # Move execution layer with versions (v0, v1, v2 and latest)
├── apps/                     # Frontend applications
└── external-crates/          # Move compiler and VM
```

### Key Architectural Patterns

1. **Authority System**: Rtd uses a set of validators (authorities) that process transactions in parallel. Each authority maintains its own state and participates in Byzantine consensus.

2. **Object Model**: Unlike account-based blockchains, Rtd uses an object-centric model where:
   - Each object has a unique ID and version
   - Objects can be owned, shared, or immutable

3. **Transaction Flow**:
   - Client → Transaction Driver → Authority Client → Validator
   - Transactions affecting only owned objects can start execution before consensus
   - Shared object transactions require consensus ordering before execution

4. **Storage Layer**: 
   - Uses RocksDB for persistent storage
   - Separate stores for objects, transactions, and effects
   - Checkpointing system for state synchronization

5. **Execution Pipeline**:
   - Transaction validation → Certificate creation → Execution → Effects commitment
   - Move VM executes smart contracts with gas metering
   - Parallel execution for non-conflicting transactions

### Critical Development Notes
1. **Testing Requirements**:
   - Always run tests before submitting changes
   - Framework changes require snapshot updates
2. **CRITICAL - Final Development Steps**:
   - **ALWAYS run `cargo xclippy` after finishing development** to ensure code passes all linting checks
   - **NEVER disable or ignore tests** - all tests must pass and be enabled
   - **NEVER use `#[allow(dead_code)]`, `#[allow(unused)]`, or any other linting suppressions** - fix the underlying issues instead
   - **All unit tests must work properly** - use `#[tokio::test]` for async tests, not `#[test]`

### **Comment Writing Guidelines**

**Do NOT comment the obvious** - comments should not simply repeat what the code does.
**When to comment**:
- Non-obvious algorithms or business logic
- Temporary exclusions, timeouts, or thresholds and their reasoning  
- Complex calculations where the "why" isn't immediately clear
- Subtle race conditions or threading considerations
- Assumptions about external state or preconditions

**When NOT to comment**:
- Simple variable assignments
- Standard library usage
- Self-descriptive function calls
- Basic control flow (if/for/while)

## Brand Rename Reference (RTD Fork from Sui)

This project is a fork of Sui blockchain, rebranded as RTD. When fixing errors or making modifications related to the brand rename, **ALWAYS** reference the original Sui source code located at:

```
/Users/changzechuan/WenchuanProjects/SuiTestProjects/Sui-Origin/sui
```

### Brand Rename Rules

| Original | Replacement | Description |
|----------|-------------|-------------|
| MystenLabs | LinkUVerse | Organization name |
| Mysten | LinkU | Brand name (capitalized) |
| mysten | linku | Brand name (lowercase) |
| SUI | RTD | All uppercase |
| Sui | Rtd | Mixed case |
| sui | rtd | All lowercase |

### Important Guidelines

1. **Always verify against original code**: Before fixing any rename-related errors, check the corresponding file in the original Sui codebase to understand the correct code structure.

2. **Path mapping**: When looking up original files, remember to use the pre-rename paths:
   - `crates/rtd-*` → `crates/sui-*`
   - `crates/linku-*` → `crates/mysten-*`
   - `rtd-execution/` → `sui-execution/`

3. **Common pitfalls**:
   - Constants (e.g., `SUI` → `RTD`) vs enum variants (e.g., `Sui` → `Rtd`) have different casing rules
   - String literals containing "sui" should become "rtd" (lowercase)
   - Be careful with compound words (e.g., `SuiAddress` → `RtdAddress`)

4. **Rename scripts location**: `fork-instruct/v2/` contains brand rename scripts and patches
