# ANML Registration Contract

A Secret Network smart contract for verifying identity, managing registrations, and distributing ANML tokens.

## Overview

This project is a Secret Network implementation of identity verification that:

1. Securely matches people to their IDs using identity verification
2. Privately stores registration information on-chain
3. Verifies uniqueness against previous registrations
4. Manages token distribution to registered users (one ANML token per day)
5. Supports flexible allocation of rewards

## Key Features

- **Privacy-Preserving Identity Verification**: Uses Secret Network's privacy features to securely handle sensitive identity information
- **Unique Registration Enforcement**: Prevents duplicate registrations by comparing against existing data
- **Daily Token Rewards**: Registered users can claim ANML tokens on a daily basis
- **Configurable Reward Allocation**: Flexible system for distributing rewards
- **Permissioned Management**: Role-based access control for contract operations

## Contract Components

The system consists of two main parts:

1. **Rust Smart Contract**: Handles on-chain logic, verification, and token distribution
2. **Node.js Client**: Provides tools for deploying and interacting with the contract

## Building the Contract

```bash
# Compile the contract
make build

# Run tests
cargo test

# Generate schema
cargo schema
```

## Contract Deployment

### Prerequisites

- Secret Network CLI installed
- Funded account on Secret Network

### Step-by-Step Deployment Process

For maximum control and security, follow these individual steps:

#### 1. Setup and Configuration

```bash
# Navigate to the node directory
cd node

# Install dependencies
npm install

# Create and configure your .env file
cp .env.example .env

# Edit the .env file with your mnemonic and network settings
nano .env  # or use your preferred text editor
```

#### 2. Verify Connection Before Proceeding

```bash
# Run the diagnostics tool to verify network connectivity
node diagnose.js

# Make sure the network connection is successful before proceeding
```

#### 3. Upload the Contract Code (Step 1 of deployment)

```bash
# Upload the contract to the blockchain
node index.js deploy upload

# IMPORTANT: Note the code ID and code hash that are returned
# You'll need these values for the next step
```

#### 4. Verify the Uploaded Contract

```bash
# You can verify the uploaded code exists with:
node index.js query hash <address_of_another_contract_you_own>

# This is a general verification that your connectivity works
```

#### 5. Instantiate a New Contract (Step 2 of deployment)

```bash
# Use the code ID from step 3
node index.js deploy instantiate <code_id> <code_hash>

# IMPORTANT: Note the contract address that is returned
# Add this address to your .env file for future reference
```

#### 6. Verify the Contract Instantiation

```bash
# Query the contract state to ensure it was instantiated correctly
node index.js query state <contract_address> <contract_hash>

# This should return the initial state of your contract
```

### Contract Migration (When needed)

Migration should be performed with extreme caution as it upgrades the contract code:

#### 1. Upload New Version of the Contract

```bash
# Upload the new contract version
node index.js deploy upload

# IMPORTANT: Note the NEW code ID and code hash
```

#### 2. Verify the Upload Was Successful

```bash
# Take time to verify the upload succeeded before proceeding
```

#### 3. Review the Migration Impact (Critical Step)

```bash
# Understand what changes the new contract version will introduce
# Review the code diff between versions
```

#### 4. Migrate the Contract to the New Version

```bash
# Use the contract address and NEW code ID from previous steps
node index.js deploy migrate <contract_address> <new_code_id> <new_code_hash>
```

#### 5. Verify the Migration Was Successful

```bash
# Query the contract state after migration
node index.js query state <contract_address> <contract_hash>

# Verify that the contract behaves as expected with a few test operations
```

## Contract Interaction

### Managing Registrations

```bash
# Query current contract state
node index.js query state

# Check registration status
node index.js query info <address>
```

### Managing Allocations

```bash
# Add a new allocation
node index.js ops add-allocation

# Set allocation percentages
node index.js ops set-allocation

# Edit existing allocation
node index.js ops edit-allocation <id>
```

## Contract Initialization

When deploying the contract, you must provide the following parameters:

```json
{
  "registration_address": "<address>",
  "registration_wallet": "<address>",
  "contract_manager": "<address>",
  "anml_token_contract": "<address>",
  "anml_token_hash": "<hash>",
  "erth_token_contract": "<address>",
  "erth_token_hash": "<hash>",
  "anml_pool_contract": "<address>",
  "anml_pool_hash": "<hash>"
}
```

Where:

- `registration_address`: Address for the API wallet that sends ID info
- `anml_token_contract`: Address of the ANML token contract
- `anml_token_hash`: Code hash of the ANML token contract

## Development

### Project Structure

```
/
├── src/                # Contract source code
│   ├── contract.rs     # Main contract implementation
│   ├── msg.rs          # Message definitions
│   └── state.rs        # Contract state management
├── node/               # JavaScript client tools
│   ├── scripts/        # Operation scripts
│   ├── utils/          # Shared utilities
│   └── index.js        # CLI entry point
└── tests/              # Contract tests
```

## License

This project is licensed under the GNU General Public License v3.0 - see the [LICENSE](LICENSE) file for details.

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

1. Fork the repository
2. Create your feature branch (`git checkout -b feature/amazing-feature`)
3. Commit your changes (`git commit -m 'Add some amazing feature'`)
4. Push to the branch (`git push origin feature/amazing-feature`)
5. Open a Pull Request
