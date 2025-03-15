# ANML Registration Contract Client

This is a CLI tool and JavaScript library for interacting with the ANML Registration smart contract on the Secret Network blockchain.

## Project Structure

```
node/
├── .env                     # Environment variables configuration
├── index.js                 # Main entry point and re-exports
├── examples.js              # Example usage of all operations
├── scripts/                 # Individual command scripts
│   ├── deploy.js            # Contract deployment, instantiation, and migration
│   ├── query.js             # Read-only contract and network queries
│   └── operations.js        # Contract state-changing operations
└── utils/
    └── client.js            # Common utilities and client setup
```

## Setup

1. Install dependencies:

```bash
npm install
```

2. Configure your environment variables in `.env`:

```
MNEMONIC="your wallet mnemonic here"
ERTH_URL="https://lcd.erth.network"
CHAIN_ID="secret-4"
```

## Usage

### Using the CLI

The package provides a simple CLI for common operations:

```bash
# Show available commands
node index.js

# Show deployment commands
node index.js deploy

# Show query commands
node index.js query

# Show operation commands
node index.js ops

# Examples of specific commands:
node index.js query state                        # Query contract state
node index.js query hash <contract-address>      # Get contract's code hash
node index.js deploy instantiate                 # Instantiate a contract
node index.js ops add-allocation                 # Add an allocation
```

### Running Examples

To see examples of how to use the library, run:

```bash
# Show available examples
node examples.js

# Run the query flow example
node examples.js query-flow

# Run the allocation management example
node examples.js allocation-flow
```

### Using as a Library

You can also import the functions to use in your own scripts:

```javascript
import { createClient, queryContractState, addAllocation } from "./index.js";

// Example: Create a custom script
async function myCustomScript() {
  // Connect to the network
  const client = await createClient();

  // Query contract state
  const state = await queryContractState();
  console.log(state);

  // Add an allocation
  const result = await addAllocation();
  console.log(result);
}
```

## Available Functions

### Utility Functions

- `createClient()` - Creates an authenticated client connected to the network
- `executeQueryWithTimeout(fn, timeout)` - Executes a query with a timeout
- `readContractWasm()` - Reads the contract WASM file

### Deployment Functions

- `uploadContract()` - Uploads the contract to the blockchain
- `instantiateContract(codeId, codeHash)` - Instantiates a contract
- `migrateContract(contractAddress, codeId, codeHash)` - Migrates a contract to a new code ID

### Query Functions

- `queryContractState(contractAddress, codeHash)` - Queries contract state
- `queryNodeStatus()` - Queries node/network status
- `queryContractInfo(contractAddress)` - Queries contract information
- `getCodeHash(contractAddress)` - Gets contract's code hash

### Operation Functions

- `updateContractConfig(contractAddress, codeHash)` - Updates contract configuration
- `addAllocation(contractAddress, codeHash, ...)` - Adds an allocation
- `claimAllocation(contractAddress, codeHash, allocationId)` - Claims an allocation
- `setAllocationPercentages(contractAddress, codeHash, allocations)` - Sets allocation percentages
- `editAllocation(contractAddress, codeHash, allocationId, config)` - Edits an allocation
- `addMinter(minterAddress)` - Adds a minter to the ANML token contract

## Troubleshooting

If you experience hanging or timeouts:

1. Check your internet connection
2. Verify the node URL in the `.env` file
3. Ensure you have proper funds in your wallet
4. The functions include timeouts to prevent indefinite hanging

## Security Notes

1. Your mnemonic is stored in the `.env` file - keep this secure and never commit it to version control
2. Only execute transactions on networks you trust
3. Always verify transaction parameters before broadcasting
