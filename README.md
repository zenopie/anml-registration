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

### Using the Node.js Client

The `node` directory contains a complete client for interacting with the contract:

```bash
cd node

# Install dependencies
npm install

# Configure environment variables
cp .env.example .env
# Edit .env with your mnemonic and network settings

# Deploy the contract
node index.js deploy upload
node index.js deploy instantiate
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
