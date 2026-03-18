# Benchmark Suite

Benchmarks proof generation times for Noir circuits. Measures witness generation and proof generation separately, producing timestamped JSON results.

Located in `packages/benchmark/`.

## Prerequisites

- Node.js >= 20
- `nargo` installed and on PATH (matching the project's Noir version)
- `pnpm install` run at the project root

## Usage

```
pnpm bench -- --circuit <name> --runs <N> [--leaves <count>] [--skip-compile]
```

| Flag | Required | Default | Description |
|------|----------|---------|-------------|
| `--circuit` | Yes | — | Circuit to benchmark (`membership`, `non_membership`) |
| `--runs` | Yes | — | Number of benchmark iterations |
| `--leaves` | No | 10 | Number of leaves in the test merkle tree |
| `--skip-compile` | No | — | Skip `nargo compile`, use existing artifacts |

### Examples

```bash
# Benchmark membership proof, 5 runs with 10 leaves
pnpm bench -- --circuit membership --runs 5

# Benchmark non-membership proof, 3 runs with 100 leaves
pnpm bench -- --circuit non_membership --runs 3 --leaves 100

# Skip recompilation for repeated runs
pnpm bench -- --circuit membership --runs 10 --skip-compile
```

## What It Measures

Each benchmark run captures three timings:

- **Witness generation** — executing the circuit logic (`noir.execute()`) to solve all constraints and produce a complete variable assignment
- **Proof generation** — running the Barretenberg cryptographic prover (`backend.generateProof()`) to produce an UltraHonk proof
- **Total** — wall-clock time for both phases combined

**Barretenberg init time** is recorded separately. This is the one-time cost of booting the WASM runtime, measured outside the benchmark loop.

## Output

Results are written to `benchmark-data/` at the project root (gitignored). Each run produces a timestamped JSON file:

```
benchmark-data/membership-2026-03-18T2146.json
```

### Output schema

```json
{
  "circuit": "membership",
  "timestamp": "2026-03-18T21:46:03.129Z",
  "config": { "runs": 5, "leaves": 10 },
  "system": { "platform": "linux", "arch": "x64", "nodeVersion": "v22.11.0" },
  "barretenbergInitMs": 11.4,
  "results": [
    {
      "run": 1,
      "witnessGenerationMs": 80.2,
      "proofGenerationMs": 469.8,
      "totalMs": 550.1
    }
  ],
  "aggregate": {
    "witnessGeneration": { "mean": 46.6, "min": 13.0, "max": 80.2, "stddev": 33.6 },
    "proofGeneration": { "mean": 492.1, "min": 469.8, "max": 514.4, "stddev": 22.3 },
    "total": { "mean": 538.8, "min": 527.5, "max": 550.1, "stddev": 11.3 }
  }
}
```

## Adding a New Circuit Benchmark

To benchmark a new circuit, add an entry to the circuit registry in `packages/benchmark/src/circuits.ts`.

### 1. Define the circuit config

Each circuit needs a `CircuitConfig` with five fields:

```ts
const myCircuit: CircuitConfig = {
  // Must match the Nargo package name
  name: "my_circuit",

  // Path to the Nargo project (contains Nargo.toml)
  projectDir: "circuits/my_circuit",

  // Path to the compiled artifact (produced by nargo compile)
  artifactPath: "circuits/my_circuit/target/my_circuit.json",

  // Generate test data: a set of merkle leaves and a target address
  generateTestData(leafCount: number) {
    // Create leaves and pick/generate an address for the proof
    // Return { leaves, address }
  },

  // Convert test data into the circuit's expected InputMap
  generateInputs(leaves: bigint[], address: bigint): InputMap {
    // Compute merkle proofs, format fields as hex strings
    // Return an object matching the circuit's parameter names
  },
};
```

### 2. Register it

Add the config to the `circuits` record at the bottom of `circuits.ts`:

```ts
export const circuits: Record<string, CircuitConfig> = {
  membership,
  non_membership: nonMembership,
  my_circuit: myCircuit,
};
```

The circuit is now available via `--circuit my_circuit`.

### Key conventions

- **Field values** should be formatted as `"0x"` + hex padded to 64 chars (32 bytes)
- **Array inputs** (like `hash_path`) are arrays of hex strings
- **Scalar inputs** (like `index`, `proof_type`) are decimal strings
- Use SDK utilities (`computeMerkleProof`, `computeMerkleProofForLeaf`) for merkle proof computation

## Architecture

```
packages/benchmark/src/
├── index.ts       CLI entry point, arg parsing, orchestration
├── circuits.ts    Circuit registry (test data + input generation per circuit)
├── compile.ts     Runs nargo compile on the circuit's project directory
├── bench.ts       Benchmark runner (Barretenberg init, witness/proof timing loop)
├── stats.ts       Aggregate statistics (mean, min, max, stddev)
└── output.ts      JSON result builder and file writer
```

The benchmark runner initializes Barretenberg once and reuses it across all runs. Each run creates fresh `Noir` and `UltraHonkBackend` instances to avoid state leakage between iterations.

## Future Work

### Circuits requiring on-chain data

The current circuits (membership, non-membership) only need merkle leaves and a root. Future circuits may require indexing the blockchain for specific events. The `CircuitConfig` interface can be extended with an optional data-fetching method:

```ts
interface CircuitConfig {
  // ... existing fields ...
  fetchOnChainData?: (rpcUrl: string) => Promise<unknown>;
}
```

This would pair with a `--rpc-url` CLI flag required only when the selected circuit needs on-chain data.

### Leaf count sweeps

Run benchmarks across multiple leaf counts to produce a performance curve:

```
pnpm bench -- --circuit membership --runs 5 --leaves 10,100,1000
```

### Browser-based benchmarking

The benchmark runs in Node.js. Since the end application is browser-based, a browser harness would capture real-world performance more accurately. Node.js and Chrome share V8, so proving times should be similar for Chrome users. However, differences may arise from:

- **Threading model** — Node.js `worker_threads` vs browser Web Workers + SharedArrayBuffer
- **Memory constraints** — browsers have tighter WASM memory limits and compete with the DOM
- **Cross-browser variance** — Firefox (SpiderMonkey) and Safari (JavaScriptCore) have different WASM engines and may show meaningfully different proving times

A browser harness could use a minimal Vite page (similar to the existing demos) that runs the proving loop and posts results back. This would also capture the WASM bootstrap cost (`initACVM()` / `initNoirC()`), which the Node.js benchmark skips since those modules auto-initialize in Node.
