import { fileURLToPath } from "url";
import {
  // Import from utils/client.js
  createClient,
  CONTRACT_INFO,

  // Import from scripts/deploy.js
  uploadContract,
  instantiateContract,
  migrateContract,

  // Import from scripts/query.js
  queryContractState,
  queryNodeStatus,
  queryContractInfo,
  getCodeHash,

  // Import from scripts/operations.js
  updateContractConfig,
  addAllocation,
  claimAllocation,
  setAllocationPercentages,
  editAllocation,
  addMinter,
} from "./index.js";

/**
 * This file demonstrates examples of using all the different operations
 * in the refactored codebase. Run individual examples by passing the example name:
 *
 * Usage: node examples.js <example-name>
 *
 * Example names:
 * - deploy-flow: Demonstrates the full contract deployment flow
 * - query-flow: Demonstrates various query operations
 * - allocation-flow: Demonstrates allocation management
 * - config-flow: Demonstrates configuration and admin operations
 * - all: Run all examples (will take a long time)
 */

// Helper function to print a section header
const printSection = (title) => {
  console.log("\n" + "=".repeat(50));
  console.log(`🚀 ${title.toUpperCase()}`);
  console.log("=".repeat(50) + "\n");
};

// ==================== DEPLOYMENT EXAMPLES ====================

/**
 * Demonstrates the full contract deployment flow
 */
async function deploymentFlowExample() {
  printSection("Deployment Flow Example");

  try {
    // Step 1: Upload the contract
    console.log("Step 1: Uploading contract...");
    const { codeId, codeHash } = await uploadContract();
    console.log(
      `Contract uploaded successfully with code ID: ${codeId} and hash: ${codeHash}`
    );

    // Step 2: Instantiate the contract
    console.log("\nStep 2: Instantiating contract...");
    const { contractAddress } = await instantiateContract(codeId, codeHash);
    console.log(
      `Contract instantiated successfully at address: ${contractAddress}`
    );

    // Step 3: Migrate the contract (usually done later when code needs to be updated)
    console.log("\nStep 3: Demonstrating contract migration (not executed)...");
    console.log(
      "To migrate: await migrateContract(contractAddress, newCodeId, newCodeHash);"
    );

    return { codeId, codeHash, contractAddress };
  } catch (error) {
    console.error("Deployment flow example failed:", error);
  }
}

// ==================== QUERY EXAMPLES ====================

/**
 * Demonstrates various query operations
 */
async function queryFlowExample() {
  printSection("Query Flow Example");

  try {
    // Step 1: Query node status
    console.log("Step 1: Querying node status...");
    const nodeStatus = await queryNodeStatus();
    console.log(`Node is at block height: ${nodeStatus.block.header.height}`);

    // Step 2: Query contract info
    console.log("\nStep 2: Querying contract info...");
    const address = CONTRACT_INFO.REGISTRATION.ADDRESS;
    const contractInfo = await queryContractInfo(address);
    console.log(`Contract code ID: ${contractInfo.code_id}`);

    // Step 3: Get contract code hash
    console.log("\nStep 3: Getting contract code hash...");
    const codeHash = await getCodeHash(address);
    console.log(`Contract code hash: ${codeHash}`);

    // Step 4: Query contract state
    console.log("\nStep 4: Querying contract state...");
    const contractState = await queryContractState(address);
    console.log("Contract state retrieved successfully");

    return { nodeStatus, contractInfo, codeHash, contractState };
  } catch (error) {
    console.error("Query flow example failed:", error);
  }
}

// ==================== ALLOCATION EXAMPLES ====================

/**
 * Demonstrates allocation management
 */
async function allocationFlowExample() {
  printSection("Allocation Flow Example");

  try {
    const address = CONTRACT_INFO.REGISTRATION.ADDRESS;
    const hash = CONTRACT_INFO.REGISTRATION.HASH;

    // Step 1: Add an allocation
    console.log("Step 1: Adding allocation...");
    const addResult = await addAllocation(address, hash);
    console.log(
      `Allocation added with transaction hash: ${addResult.transactionHash}`
    );

    // Step 2: Set allocation percentages
    console.log("\nStep 2: Setting allocation percentages...");
    const percentages = [
      {
        allocation_id: 1,
        percentage: "100",
      },
    ];
    const setResult = await setAllocationPercentages(
      address,
      hash,
      percentages
    );
    console.log(
      `Allocation percentages set with transaction hash: ${setResult.transactionHash}`
    );

    // Step 3: Edit an allocation
    console.log("\nStep 3: Editing allocation...");
    const config = {
      receive_addr: address,
      receive_hash: hash,
      manager_addr: null, // Will be set to client address in function
      claimer_addr: null,
      use_send: true,
    };
    const editResult = await editAllocation(address, hash, 1, config);
    console.log(
      `Allocation edited with transaction hash: ${editResult.transactionHash}`
    );

    // Step 4: Claim an allocation
    console.log("\nStep 4: Claiming allocation...");
    const claimResult = await claimAllocation(address, hash, 1);
    console.log(
      `Allocation claimed with transaction hash: ${claimResult.transactionHash}`
    );

    return { addResult, setResult, editResult, claimResult };
  } catch (error) {
    console.error("Allocation flow example failed:", error);
  }
}

// ==================== CONFIGURATION EXAMPLES ====================

/**
 * Demonstrates configuration and admin operations
 */
async function configFlowExample() {
  printSection("Configuration Flow Example");

  try {
    // Step 1: Update contract configuration
    console.log("Step 1: Updating contract configuration...");
    const address = CONTRACT_INFO.REGISTRATION.ADDRESS;
    const hash = CONTRACT_INFO.REGISTRATION.HASH;
    const updateResult = await updateContractConfig(address, hash);
    console.log(
      `Contract configuration updated with transaction hash: ${updateResult.transactionHash}`
    );

    // Step 2: Add a minter to the ANML token
    console.log("\nStep 2: Adding a minter to ANML token...");
    const addMinterResult = await addMinter(address);
    console.log(
      `Minter added with transaction hash: ${addMinterResult.transactionHash}`
    );

    return { updateResult, addMinterResult };
  } catch (error) {
    console.error("Configuration flow example failed:", error);
  }
}

// ==================== MAIN FUNCTION ====================

async function main() {
  const args = process.argv.slice(2);
  const exampleName = args[0] || "";

  if (!exampleName) {
    console.log(`
      Usage: node examples.js <example-name>
      
      Available examples:
        deploy-flow     - Demonstrates the full contract deployment flow
        query-flow      - Demonstrates various query operations
        allocation-flow - Demonstrates allocation management
        config-flow     - Demonstrates configuration and admin operations
        all             - Run all examples (will take a long time)
    `);
    process.exit(0);
  }

  try {
    if (exampleName === "deploy-flow" || exampleName === "all") {
      await deploymentFlowExample();
    }

    if (exampleName === "query-flow" || exampleName === "all") {
      await queryFlowExample();
    }

    if (exampleName === "allocation-flow" || exampleName === "all") {
      await allocationFlowExample();
    }

    if (exampleName === "config-flow" || exampleName === "all") {
      await configFlowExample();
    }

    if (
      ![
        "deploy-flow",
        "query-flow",
        "allocation-flow",
        "config-flow",
        "all",
      ].includes(exampleName)
    ) {
      console.log(`Unknown example: ${exampleName}`);
    }
  } catch (error) {
    console.error("Error running examples:", error);
    process.exit(1);
  }
}

// Execute main function if script is run directly
if (process.argv[1] === fileURLToPath(import.meta.url)) {
  main()
    .then(() => console.log("\n✅ Examples completed successfully"))
    .catch((error) => {
      console.error("\n❌ Examples failed:", error);
      process.exit(1);
    });
}
