import type { CompiledCircuit } from "@noir-lang/noir_js";

/**
 * Fetch a compiled Noir circuit artifact from an IPFS gateway.
 *
 * Expects the CID to point directly at the compiled circuit JSON file
 * (output of `nargo compile`), not a wrapping directory.
 */
export async function fetchCircuit(
  gatewayUrl: string,
  metadataHash: string,
): Promise<CompiledCircuit> {
  const baseUrl = gatewayUrl.replace(/\/+$/, "");
  const cid = metadataHash.startsWith("/ipfs/")
    ? metadataHash.slice(6)
    : metadataHash;

  let circuit: CompiledCircuit;
  try {
    const res = await fetch(`${baseUrl}/ipfs/${cid}`);
    if (!res.ok) {
      throw new Error(`HTTP ${res.status}`);
    }
    circuit = await res.json();
  } catch (err) {
    throw new Error(
      `Failed to fetch circuit from IPFS (${cid}) at ${baseUrl}: ${err instanceof Error ? err.message : err}`,
    );
  }

  if (!circuit.bytecode || !circuit.abi) {
    throw new Error(
      `CID ${cid} is not a compiled circuit artifact (missing bytecode or abi). ` +
        `Ensure the regulator uploaded the compiled JSON from nargo compile.`,
    );
  }

  return circuit;
}

/**
 * Fetch a leaves JSON file from IPFS (a single file, not a directory).
 *
 * Expects the CID to point to a JSON array of hex strings: `["0x...", ...]`.
 * Returns the parsed array as `bigint[]`.
 */
export async function fetchLeaves(
  gatewayUrl: string,
  leavesCid: string,
): Promise<bigint[]> {
  const baseUrl = gatewayUrl.replace(/\/+$/, "");
  const cid = leavesCid.startsWith("/ipfs/") ? leavesCid.slice(6) : leavesCid;

  const res = await fetch(`${baseUrl}/ipfs/${cid}`);
  if (!res.ok) {
    throw new Error(
      `Failed to fetch leaves from IPFS (${cid}): HTTP ${res.status}`,
    );
  }

  const arr: string[] = await res.json();
  if (!Array.isArray(arr)) {
    throw new Error(`Leaves CID ${cid} did not contain a JSON array`);
  }

  return arr.map((v) => BigInt(v));
}
