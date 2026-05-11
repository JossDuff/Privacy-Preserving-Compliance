export const ComplianceDefinitionABI = [
  {
    "type": "constructor",
    "inputs": [
      {
        "name": "_regulator",
        "type": "address",
        "internalType": "address"
      },
      {
        "name": "_name",
        "type": "string",
        "internalType": "string"
      }
    ],
    "stateMutability": "nonpayable"
  },
  {
    "type": "function",
    "name": "getActiveVersion",
    "inputs": [],
    "outputs": [
      {
        "name": "",
        "type": "tuple",
        "internalType": "struct ComplianceDefinition.ComplianceVersion",
        "components": [
          {
            "name": "verifier",
            "type": "address",
            "internalType": "address"
          },
          {
            "name": "merkleRoot",
            "type": "bytes32",
            "internalType": "bytes32"
          },
          {
            "name": "merkleRoot2",
            "type": "bytes32",
            "internalType": "bytes32"
          },
          {
            "name": "tStart",
            "type": "uint256",
            "internalType": "uint256"
          },
          {
            "name": "tEnd",
            "type": "uint256",
            "internalType": "uint256"
          },
          {
            "name": "metadataHash",
            "type": "string",
            "internalType": "string"
          },
          {
            "name": "leavesHash",
            "type": "string",
            "internalType": "string"
          },
          {
            "name": "leavesHash2",
            "type": "string",
            "internalType": "string"
          }
        ]
      }
    ],
    "stateMutability": "view"
  },
  {
    "type": "function",
    "name": "getVersionAt",
    "inputs": [
      {
        "name": "blockHeight",
        "type": "uint256",
        "internalType": "uint256"
      }
    ],
    "outputs": [
      {
        "name": "",
        "type": "tuple",
        "internalType": "struct ComplianceDefinition.ComplianceVersion",
        "components": [
          {
            "name": "verifier",
            "type": "address",
            "internalType": "address"
          },
          {
            "name": "merkleRoot",
            "type": "bytes32",
            "internalType": "bytes32"
          },
          {
            "name": "merkleRoot2",
            "type": "bytes32",
            "internalType": "bytes32"
          },
          {
            "name": "tStart",
            "type": "uint256",
            "internalType": "uint256"
          },
          {
            "name": "tEnd",
            "type": "uint256",
            "internalType": "uint256"
          },
          {
            "name": "metadataHash",
            "type": "string",
            "internalType": "string"
          },
          {
            "name": "leavesHash",
            "type": "string",
            "internalType": "string"
          },
          {
            "name": "leavesHash2",
            "type": "string",
            "internalType": "string"
          }
        ]
      }
    ],
    "stateMutability": "view"
  },
  {
    "type": "function",
    "name": "getVersionCount",
    "inputs": [],
    "outputs": [
      {
        "name": "",
        "type": "uint256",
        "internalType": "uint256"
      }
    ],
    "stateMutability": "view"
  },
  {
    "type": "function",
    "name": "name",
    "inputs": [],
    "outputs": [
      {
        "name": "",
        "type": "string",
        "internalType": "string"
      }
    ],
    "stateMutability": "view"
  },
  {
    "type": "function",
    "name": "regulator",
    "inputs": [],
    "outputs": [
      {
        "name": "",
        "type": "address",
        "internalType": "address"
      }
    ],
    "stateMutability": "view"
  },
  {
    "type": "function",
    "name": "updateCircuit",
    "inputs": [
      {
        "name": "newVerifier",
        "type": "address",
        "internalType": "address"
      },
      {
        "name": "newMerkleRoot",
        "type": "bytes32",
        "internalType": "bytes32"
      },
      {
        "name": "newMerkleRoot2",
        "type": "bytes32",
        "internalType": "bytes32"
      },
      {
        "name": "tStart",
        "type": "uint256",
        "internalType": "uint256"
      },
      {
        "name": "tEnd",
        "type": "uint256",
        "internalType": "uint256"
      },
      {
        "name": "metadataHash",
        "type": "string",
        "internalType": "string"
      },
      {
        "name": "leavesHash",
        "type": "string",
        "internalType": "string"
      },
      {
        "name": "leavesHash2",
        "type": "string",
        "internalType": "string"
      }
    ],
    "outputs": [],
    "stateMutability": "nonpayable"
  },
  {
    "type": "function",
    "name": "updateParams",
    "inputs": [
      {
        "name": "newMerkleRoot",
        "type": "bytes32",
        "internalType": "bytes32"
      },
      {
        "name": "newMerkleRoot2",
        "type": "bytes32",
        "internalType": "bytes32"
      },
      {
        "name": "newLeavesHash",
        "type": "string",
        "internalType": "string"
      },
      {
        "name": "newLeavesHash2",
        "type": "string",
        "internalType": "string"
      }
    ],
    "outputs": [],
    "stateMutability": "nonpayable"
  },
  {
    "type": "function",
    "name": "verify",
    "inputs": [
      {
        "name": "proof",
        "type": "bytes",
        "internalType": "bytes"
      }
    ],
    "outputs": [
      {
        "name": "",
        "type": "bool",
        "internalType": "bool"
      }
    ],
    "stateMutability": "nonpayable"
  },
  {
    "type": "function",
    "name": "versions",
    "inputs": [
      {
        "name": "",
        "type": "uint256",
        "internalType": "uint256"
      }
    ],
    "outputs": [
      {
        "name": "verifier",
        "type": "address",
        "internalType": "address"
      },
      {
        "name": "merkleRoot",
        "type": "bytes32",
        "internalType": "bytes32"
      },
      {
        "name": "merkleRoot2",
        "type": "bytes32",
        "internalType": "bytes32"
      },
      {
        "name": "tStart",
        "type": "uint256",
        "internalType": "uint256"
      },
      {
        "name": "tEnd",
        "type": "uint256",
        "internalType": "uint256"
      },
      {
        "name": "metadataHash",
        "type": "string",
        "internalType": "string"
      },
      {
        "name": "leavesHash",
        "type": "string",
        "internalType": "string"
      },
      {
        "name": "leavesHash2",
        "type": "string",
        "internalType": "string"
      }
    ],
    "stateMutability": "view"
  },
  {
    "type": "error",
    "name": "NoActiveVersion",
    "inputs": []
  },
  {
    "type": "error",
    "name": "NoVersionAtBlock",
    "inputs": [
      {
        "name": "blockHeight",
        "type": "uint256",
        "internalType": "uint256"
      }
    ]
  },
  {
    "type": "error",
    "name": "NotRegulator",
    "inputs": []
  }
] as const;
