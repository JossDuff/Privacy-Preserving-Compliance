# Onboarding Task: Whitelist Zero-Knowledge Circuit

## Overview

This task is designed to help you understand how zero-knowledge circuits interact with smart contracts and the use of on-chain vs off-chain data in proofs. If you hate Noir and Solidity then you won't have to touch it after this task.  I'm assigning this to you because **you must understand the interaction between smart contracts and proofs** — this is fundamental to my thesis. 

## Task Requirements

### Part 1: Noir Circuit Implementation

Write a whitelist Noir circuit with the following specifications:

- **Input**: An address and verify if it exists in the whitelist (fail otherwise)
- **Secret validation**: Check if the caller knows the secret number (42)
- **Secret number**: Must remain off-chain (cannot be stored in the smart contract)

Your Noir function should follow this structure:

```noir
fn main(whitelist, caller, secret_number) {
    let mut found = false;
    for addr in whitelist {
        if addr == caller {
            found = true;
        }
    }
    
    assert(found == true);
    assert(secret_number == 42);
}
```

### Part 2: Smart Contract Implementation

Create a smart contract with a `do-something` function that:

- Verifies the caller has a valid proof from the above circuit
- Uses a public whitelist (stored on-chain)
- References the verifier contract address
- **Important**: The secret number must NOT be stored on-chain

Your Solidity function should follow this structure:

```solidity
pub whitelist: address[]
pub verifier_contract: address

pub fn do_something(proof) {
    require(verifier_contract.verify(proof, msg.sender, whitelist))
}
```

**Testing**: Add your address and a few dummy addresses to the whitelist for testing purposes.

## Deliverables

Deploy both contracts to **Sepolia testnet** and provide:

1. Address of the whitelist contract
2. Address of the circuit verifier contract

**Important**: Verify both contracts on the Sepolia block explorer so the source code is publicly visible.

## Resources

- [Noir.js Tutorial](https://noir-lang.org/docs/tutorials/noirjs_app)
- [Barretenberg Solidity Verifier Guide](https://barretenberg.aztec.network/docs/how_to_guides/how-to-solidity-verifier/)
