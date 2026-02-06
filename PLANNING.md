# Development and planning document

---


## Tech Stack

### Smart Contracts
- **Solidity** - Verifier contracts and example applications
- **EVM** - deployments will be made to the Sepolia testnet

### Zero-Knowledge Proofs
- **Noir** - ZK circuit DSL for compliance definitions
- **PLONK** - Proving system (supports reusable trusted setup)

### Backend/Tooling
- **Rust** - CLI tools and proof manager
- **IPFS** - Decentralized storage for compliance definitions

---

## Components

### 1. Regulator Cli

**Purpose**: Enable regulators to create, publish, and update compliance definitions

**Deliverables**:
- CLI for compliance definition management
- Example compliance circuits (sanctions list, account age, etc.)
- Verifier contract deployment system
- IPFS publishing integration

**Key Functionality**:
- Sign compliance definitions with regulator private key
- Deploy verifier contracts with aggregation support
- Update existing compliance definitions (constraints or parameters)
- Upload to IPFS and manage metadata

### 2. User Proof Manager

**Purpose**: Enable users to generate compliance proofs efficiently

**Deliverables**:
- Rust binary/library for proof generation
- Transaction history indexing
- Constraint inference engine
- Proof caching system

**Key Functionality**:
- Fetch compliance definition from IPFS or direct input
- Verify regulator signatures
- Index user transaction history
- Fetch public proof inputs from verifier contract
- Generate individual constraint proofs
- Aggregate proofs into compliance definition proof
- Cache and reuse previously generated proofs

### 3. Example Applications

**Purpose**: Demonstrate framework integration

**Deliverables**:
- Compliant ERC20 Stablecoin or Compliant Transaction Mixer (Tornado Cash fork)
- Example frontend

**Integration Pattern**:
```solidity
function transfer(address recipient, uint256 amount, bytes proof) {
    require(verifierContract.verify(proof), "Not compliant");
    // ... rest of transfer logic
}
```

---

## MVP Tasks

### Milestone 1: Regulator Stack

#### Core CLI Development
- [ ] `init` command
  - [ ] Generate example config files
- [ ] `publish` command
  - [ ] Take Noir circuit input
  - [ ] Sign circuit with regulator private key
  - [ ] Upload to IPFS (integrate with ipfs-api crate)
  - [ ] Deploy verifier contract
- [ ] `update` command
  - [ ] Update circuit (deploy new verifier)
  - [ ] Update public parameters (contract call)
  - [ ] Version management

#### Verifier Contract Development
- [ ] Extend Noir-generated verifier
  - [ ] Add version tracking
  - [ ] Implement parameter update functions
  - [ ] Add metadata storage (IPFS hash)
- [ ] Access control
  - [ ] Ownable pattern for updates
  - [ ] Regulator signature verification on-chain

#### Example Compliance Circuits
- [ ] Simple single-constraint circuits
  - [ ] Sanctions list check (NON-MEM)
  - [ ] Allow-list check (MEM)
  - [ ] Account age constraint (AGE)

### Milestone 2: User Proof Manager

#### Circuit handling
- [ ] Input verifier contract address
- [ ] Fetch Noir code of compliance definition from IPFS
- [ ] Verify regulator signature 
- [ ] Generate circuit from Noir code

#### Indexing System
- [ ] Fetch on-chain data required for proof generation
  - [ ] Indexing chain data
  - [ ] Query verifier contract for public inputs

#### Proof Generation 
- [ ] Input preparation
  - [ ] Format public inputs from contract
  - [ ] Handle private user inputs
  - [ ] Prepare witness data from tx history
- [ ] Proof generation
  - [ ] Interface with Noir proving system
  - [ ] Generate individual constraint proofs
- [ ] Proof storage system

### Milestone 3: Example Application Compliant Stablecoin

- [ ] Base ERC20 implementation
- [ ] Integration with framework
  - [ ] Add proof verification to transfer()
  - [ ] Add proof verification to approve()
- [ ] Simple frontend

### Milestone 4: Benchmarking

### Performance Metrics
- [ ] Proof generation time per constraint type
- [ ] Gas costs for verification
- [ ] Proof size measurements
- [ ] Transaction history size scaling
- [ ] Aggregation overhead

### Test Scenarios
- [ ] Empty set membership (baseline)
- [ ] Large sanction lists (10k+ addresses)
- [ ] Long transaction histories (1000+ txns)
- [ ] Complex multi-constraint definitions
- [ ] Update frequency stress tests

## Post-MVP

#### Proof aggregation
- [ ] Modify verifier contract to aggregate and verify multiple proofs
- [ ] Example compliance definition that requires multiple constraints
- [ ] Modify proof manager CLI to generate proofs of multiple constraints

#### Compliant Transaction Mixer
- [ ] Fork Tornado Cash or similar
- [ ] Add compliance checks
  - [ ] Require proof on deposit
  - [ ] Verify proof on withdrawal
- [ ] Deploy demo frontend

## Extensions and future work

#### Proof Logical Inference
- [ ] In circuit logical implication detection
- [ ] Cross-definition optimization
  - [ ] Shared constraint identification
  - [ ] Proof reuse across definitions
  - [ ] Load and fetch minimal amount of on-chain state based on constraint overlap

#### Constraint DSL
- [ ] Design readable constraint syntax
  - [ ] Transpiler to Noir
- [ ] Example constraints in DSL

#### Web Applications
- [ ] Proof manager web UI
  - [ ] Browser-based proof generation
  - [ ] Wallet integration
  - [ ] Proof status dashboard
- [ ] Regulator web interface
  - [ ] Compliance definition builder
  - [ ] Visual constraint composer
  - [ ] Deployment wizard

#### Alternative Demo Applications
- **Airdrops**: Require specific on-chain activity proofs
- **Off-chain Computation**: Prove invariants instead of executing on-chain
- **Credential Systems**: Prove eligibility without revealing attributes
- **Supply Chain**: B2B privacy on public chains

---

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

---
