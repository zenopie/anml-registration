// Re-export utility functions
export * from "./utils/client.js";

// Re-export deployment functions
export * from "./scripts/deploy.js";

// Re-export query functions
export * from "./scripts/query.js";

// Re-export operation functions
export * from "./scripts/operations.js";

// Import all functions directly for use in the CLI
import { CONTRACT_INFO } from "./utils/client.js";
import {
  uploadContract,
  instantiateContract,
  migrateContract,
} from "./scripts/deploy.js";
import {
  queryContractState,
  queryNodeStatus,
  queryContractInfo,
  getCodeHash,
} from "./scripts/query.js";
import {
  updateContractConfig,
  addAllocation,
  claimAllocation,
  setAllocationPercentages,
  editAllocation,
  addMinter,
} from "./scripts/operations.js";

// Main CLI handler
import { fileURLToPath } from "url";

// Display help for deployment commands
function showDeployHelp() {
  console.log(`
    Deployment Commands:
      upload              - Upload the contract
      instantiate         - Instantiate a contract (run after upload)
      migrate [address]   - Migrate a contract to new code ID
  `);
}

// Display help for query commands
function showQueryHelp() {
  console.log(`
    Query Commands:
      state [address] [hash]    - Query contract state
      info [address]            - Query contract info
      status                    - Query node status
      hash <address>            - Get contract code hash
  `);
}

// Display help for operation commands
function showOperationsHelp() {
  console.log(`
    Operation Commands:
      update-config [address] [hash]         - Update contract configuration
      add-allocation [address] [hash]        - Add allocation
      claim-allocation <id> [address] [hash] - Claim allocation
      set-allocation [address] [hash]        - Set allocation percentages
      edit-allocation <id> [address] [hash]  - Edit allocation
      add-minter [address]                   - Add a minter to ANML token
  `);
}

async function main() {
  const args = process.argv.slice(2);
  const category = args[0];
  const command = args[1];

  if (!category) {
    console.log(`
      ANML Registration Contract CLI
      
      Usage: node index.js <category> <command> [args]
      
      Categories:
        deploy     - Contract deployment and migration commands
        query      - Read-only commands to query contracts or network
        ops        - Contract operations and management
        
      Run 'node index.js <category>' for specific commands in each category
    `);
    return;
  }

  try {
    // Handle deploy category
    if (category === "deploy") {
      if (!command) {
        showDeployHelp();
        return;
      }

      if (command === "upload") {
        await uploadContract();
      } else if (command === "instantiate") {
        const codeId = args[2] ? parseInt(args[2]) : undefined;
        const hash = args[3];
        await instantiateContract(codeId, hash);
      } else if (command === "migrate") {
        const address = args[2];
        const codeId = args[3] ? parseInt(args[3]) : undefined;
        const hash = args[4];
        await migrateContract(address, codeId, hash);
      } else {
        console.log(`Unknown deploy command: ${command}`);
        showDeployHelp();
      }
    }
    // Handle query category
    else if (category === "query") {
      if (!command) {
        showQueryHelp();
        return;
      }

      if (command === "state") {
        const address = args[2] || CONTRACT_INFO.REGISTRATION.ADDRESS;
        const hash = args[3] || CONTRACT_INFO.REGISTRATION.HASH;
        await queryContractState(address, hash);
      } else if (command === "info") {
        const address = args[2] || CONTRACT_INFO.REGISTRATION.ADDRESS;
        await queryContractInfo(address);
      } else if (command === "status") {
        await queryNodeStatus();
      } else if (command === "hash") {
        if (!args[2]) {
          console.error("Error: Contract address is required for hash command");
          showQueryHelp();
          return;
        }
        await getCodeHash(args[2]);
      } else {
        console.log(`Unknown query command: ${command}`);
        showQueryHelp();
      }
    }
    // Handle operations category
    else if (category === "ops" || category === "operations") {
      if (!command) {
        showOperationsHelp();
        return;
      }

      if (command === "update-config") {
        const address = args[2] || CONTRACT_INFO.REGISTRATION.ADDRESS;
        const hash = args[3] || CONTRACT_INFO.REGISTRATION.HASH;
        await updateContractConfig(address, hash);
      } else if (command === "add-allocation") {
        const address = args[2] || CONTRACT_INFO.REGISTRATION.ADDRESS;
        const hash = args[3] || CONTRACT_INFO.REGISTRATION.HASH;
        await addAllocation(address, hash);
      } else if (command === "claim-allocation") {
        if (!args[2]) {
          console.error(
            "Error: Allocation ID is required for claim-allocation command"
          );
          showOperationsHelp();
          return;
        }
        const id = parseInt(args[2]);
        const address = args[3] || CONTRACT_INFO.REGISTRATION.ADDRESS;
        const hash = args[4] || CONTRACT_INFO.REGISTRATION.HASH;
        await claimAllocation(address, hash, id);
      } else if (command === "set-allocation") {
        const address = args[2] || CONTRACT_INFO.REGISTRATION.ADDRESS;
        const hash = args[3] || CONTRACT_INFO.REGISTRATION.HASH;
        await setAllocationPercentages(address, hash);
      } else if (command === "edit-allocation") {
        if (!args[2]) {
          console.error(
            "Error: Allocation ID is required for edit-allocation command"
          );
          showOperationsHelp();
          return;
        }
        const id = parseInt(args[2]);
        const address = args[3] || CONTRACT_INFO.REGISTRATION.ADDRESS;
        const hash = args[4] || CONTRACT_INFO.REGISTRATION.HASH;
        await editAllocation(address, hash, id);
      } else if (command === "add-minter") {
        const address = args[2] || CONTRACT_INFO.REGISTRATION.ADDRESS;
        await addMinter(address);
      } else {
        console.log(`Unknown operations command: ${command}`);
        showOperationsHelp();
      }
    } else {
      console.log(`Unknown category: ${category}`);
      console.log(`
        Available categories:
          deploy     - Contract deployment and migration commands
          query      - Read-only commands to query contracts or network
          ops        - Contract operations and management
      `);
    }
  } catch (error) {
    console.error("Error executing command:", error);
    process.exit(1);
  }
}

// Execute main function if script is run directly
if (process.argv[1] === fileURLToPath(import.meta.url)) {
  main()
    .then(() => console.log("CLI execution completed successfully"))
    .catch((error) => {
      console.error("CLI execution failed:", error);
      process.exit(1);
    });
}
