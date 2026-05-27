#!/usr/bin/env node
// Pin demo IPFS artifacts (compiled circuits + leaves files) to Pinata.
//
// Requires:
//   PINATA_JWT  -- scoped key with pinFileToIPFS permission
//
// Usage:
//   PINATA_JWT=eyJ... node scripts/pin-to-pinata.mjs
//
// Produces a JSON blob on stdout mapping artifact -> CID, suitable for piping
// into the demo .env or feeding into regulator-cli commands.

import { readFileSync, existsSync } from "node:fs";
import { basename, resolve } from "node:path";

const PINATA_API = "https://api.pinata.cloud/pinning/pinFileToIPFS";

const repoRoot = resolve(new URL("..", import.meta.url).pathname);

const ARTIFACTS = [
  {
    key: "non_membership_circuit",
    path: `${repoRoot}/circuits/non_membership/target/non_membership.json`,
    fileName: "non_membership.json",
    wrap: false,
    metadataName: "ppc-demo non_membership circuit",
  },
  {
    key: "membership_circuit",
    path: `${repoRoot}/circuits/membership/target/membership.json`,
    fileName: "membership.json",
    wrap: false,
    metadataName: "ppc-demo membership circuit",
  },
  {
    key: "sanction_leaves",
    path: `${repoRoot}/sanction_leaves.json`,
    fileName: "sanction_leaves.json",
    wrap: false,
    metadataName: "ppc-demo sanction leaves",
  },
  {
    key: "whitelist_leaves",
    path: `${repoRoot}/whitelist_leaves.json`,
    fileName: "whitelist_leaves.json",
    wrap: false,
    metadataName: "ppc-demo whitelist leaves",
  },
];

function die(msg) {
  console.error(`error: ${msg}`);
  process.exit(1);
}

const jwt = process.env.PINATA_JWT;
if (!jwt) die("PINATA_JWT env var not set");

for (const a of ARTIFACTS) {
  if (!existsSync(a.path)) {
    die(`missing artifact: ${a.path} (run \`nargo compile\` in the circuit dir first)`);
  }
}

async function pin(artifact) {
  const buf = readFileSync(artifact.path);
  const form = new FormData();
  form.append("file", new Blob([buf]), artifact.fileName);
  form.append(
    "pinataMetadata",
    JSON.stringify({ name: artifact.metadataName }),
  );
  form.append(
    "pinataOptions",
    JSON.stringify({ cidVersion: 0, wrapWithDirectory: artifact.wrap }),
  );

  const res = await fetch(PINATA_API, {
    method: "POST",
    headers: { Authorization: `Bearer ${jwt}` },
    body: form,
  });
  const body = await res.text();
  if (!res.ok) {
    die(`Pinata returned ${res.status} for ${artifact.key}: ${body}`);
  }
  const json = JSON.parse(body);
  return json.IpfsHash;
}

const out = {};
for (const a of ARTIFACTS) {
  process.stderr.write(`pinning ${a.key} (${basename(a.path)})... `);
  const cid = await pin(a);
  process.stderr.write(`${cid}\n`);
  out[a.key] = cid;
}

console.log(JSON.stringify(out, null, 2));
