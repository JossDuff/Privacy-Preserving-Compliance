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

## Architecture

### Actor Model

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

---

# Contributing
All contributions must be made by opening a PR to main and requires a review to be merged.

This is a master's thesis project and feedback and suggestions are welcome. Please open issues for bugs or feature requests.


# To put back in 

- [ ] Multi-constraint aggregated circuits
  - [ ] Sanctions + account age
  - [ ] Protocol interaction requirements (INTERACT/AVOID)
  - [ ] Anti-structuring constraint (STRUCTURE)

- [ ] Inference engine
  - [ ] Identify previously proven constraints
  - [ ] Determine minimal proof set needed
  - [ ] Handle constraint subset relationships

  - [ ] Add aggregation proof support (later)


