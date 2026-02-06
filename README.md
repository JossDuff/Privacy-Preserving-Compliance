# Privacy-Preserving-Compliance
Prototype implementation of masters thesis framework `Privacy Preserving Compliance`.  The current draft of my thesis can be found at [https://github.com/JossDuff/thesis](https://github.com/JossDuff/thesis).

## Overview

This repository contains the implementation of a framework for composable privacy-preserving compliance on blockchain systems. The framework enables regulatory bodies to publish compliance definitions, applications to require compliance proofs, and users to prove compliance without revealing private transaction data.

### Key Features

- **No Deanonymization**: Users prove compliance without revealing transaction histories or balances
- **Proactive Compliance**: Non-compliant actors are blocked before transactions, not detected after
- **Rich Compliance Language**: Express complex requirements using composable constraints
- **Multiple Compliance**: Support for requirements from multiple regulatory jurisdictions
- **Chain Agnostic**: Works on any blockchain with smart contracts and ZK proof verification
- **Modular Privacy**: Compatible with any privacy protocol

---

## Framework Actors

The framework supports three types of actors:

1. **Regulators**: Create and publish compliance definitions
2. **Applications**: Select relevant compliance definitions and require proofs from users
3. **Users**: Generate ZK proofs demonstrating compliance without revealing private data

### System Components
```
┌─────────────┐
│  Regulator  │
│    CLI      │
└──────┬──────┘
       │ publishes
       ▼
┌─────────────────────┐
│ ComplianceDefinition│
│   Verifier Contract │◄────────┐
└──────┬──────────────┘         │
       │                        │ requires
       │                        │
       ▼                   ┌────┴─────┐
┌─────────────┐            │   Dapp   │
│    User     │            │ Contract │
│Proof Manager│───────────►│          │
└─────────────┘  submits   └──────────┘
                  proof
```


## Repository Structure
```
privacy-preserving-compliance/
├── circuits/               # Example Noir compliance circuits
├── contracts/             # Solidity smart contracts
├── regulator-cli/         # Rust CLI for regulators
│   └── src/
├── proof-manager/         # Rust proof generation system
│   └── src/
```

# Running

```bash 
# Build 
cargo build --bin regulator-cli
cargo build --bin proof-manager

# Build and run.  For debugging, omit --release
cargo run --release --bin regulator-cli
cargo run --release --bin proof-manager
```

# Contributing
All contributions must be made by opening a PR to main and requires a review to be merged.  Include sufficient tests with any code implemented.

This is a master's thesis project and feedback and suggestions are welcome. Please open issues for bugs or feature requests.

