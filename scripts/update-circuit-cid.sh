#!/usr/bin/env bash
# Update the on-chain `metadataHash` (circuit CID) on a ComplianceDefinition,
# reusing its current verifier, merkle root, tStart, tEnd, and leavesHash.
#
# Useful when only the IPFS pin location changes (e.g. re-pinning the same
# compiled circuit under a different CID format) without changing the circuit.
#
# Requires `cast` (from Foundry).
#
# Usage:
#   RPC_URL=...  PRIVATE_KEY=... \
#     scripts/update-circuit-cid.sh <CD_ADDRESS> <NEW_METADATA_CID>

set -euo pipefail

if [[ $# -ne 2 ]]; then
  echo "usage: $0 <CD_ADDRESS> <NEW_METADATA_CID>" >&2
  exit 2
fi

CD_ADDRESS="$1"
NEW_METADATA_CID="$2"

: "${RPC_URL:?RPC_URL not set}"
: "${PRIVATE_KEY:?PRIVATE_KEY not set}"

echo "Reading active version from $CD_ADDRESS..."
# getActiveVersion() returns a ComplianceVersion struct -- the extra paren
# layer around the tuple tells cast to treat it as a struct.
ACTIVE=$(cast call "$CD_ADDRESS" \
  "getActiveVersion()((address,bytes32,uint256,uint256,string,string))" \
  --rpc-url "$RPC_URL")

# cast prints the struct as one line: (verifier, merkleRoot, tStart, tEnd, "metadataHash", "leavesHash")
# Strip the outer parens, then split on ', '.
TRIMMED="${ACTIVE#(}"
TRIMMED="${TRIMMED%)}"
IFS=',' read -r VERIFIER MERKLE T_START T_END OLD_META LEAVES <<< "$TRIMMED"
VERIFIER=$(echo "$VERIFIER" | xargs)
MERKLE=$(echo "$MERKLE" | xargs)
T_START=$(echo "$T_START" | xargs | awk '{print $1}')
T_END=$(echo "$T_END" | xargs | awk '{print $1}')
OLD_META=$(echo "$OLD_META" | xargs | tr -d '"')
LEAVES=$(echo "$LEAVES" | xargs | tr -d '"')

echo "  verifier:     $VERIFIER"
echo "  merkleRoot:   $MERKLE"
echo "  tStart:       $T_START"
echo "  tEnd:         $T_END"
echo "  old metadata: $OLD_META"
echo "  new metadata: $NEW_METADATA_CID"
echo "  leaves:       $LEAVES"
echo

echo "Calling updateCircuit..."
cast send "$CD_ADDRESS" \
  "updateCircuit(address,bytes32,uint256,uint256,string,string)" \
  "$VERIFIER" "$MERKLE" "$T_START" "$T_END" "$NEW_METADATA_CID" "$LEAVES" \
  --rpc-url "$RPC_URL" \
  --private-key "$PRIVATE_KEY"
