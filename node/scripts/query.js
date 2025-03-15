import { fileURLToPath } from "url";
import {
  createClient,
  executeQueryWithTimeout,
  CONTRACT_INFO,
} from "../utils/client.js";

/**
 * Query the contract's current state
 */
async function queryContractState(contractAddress, codeHash) {
  try {
    console.log(`Querying state for contract: ${contractAddress}`);
    const client = await createClient();

    const result = await executeQueryWithTimeout(() =>
      client.query.compute.queryContract({
        contract_address: contractAddress || CONTRACT_INFO.REGISTRATION.ADDRESS,
        code_hash: codeHash || CONTRACT_INFO.REGISTRATION.HASH,
        query: { query_state: {} },
      })
    );

    console.log("Contract state:", JSON.stringify(result, null, 2));
    return result;
  } catch (error) {
    console.error("Error querying contract state:", error);
    if (error.message.includes("timed out")) {
      console.log(
        "The query timed out. This could indicate network issues or contract problems."
      );
    }
    process.exit(1);
  }
}

/**
 * Query basic network status
 */
async function queryNodeStatus() {
  try {
    console.log("Querying node status...");
    const client = await createClient();

    const status = await executeQueryWithTimeout(() =>
      client.query.tendermint.getLatestBlock({})
    );

    console.log("Latest block height:", status.block.header.height);
    console.log("Latest block time:", status.block.header.time);
    return status;
  } catch (error) {
    console.error("Node status query failed:", error);
    process.exit(1);
  }
}

/**
 * Query contract information
 */
async function queryContractInfo(contractAddress) {
  try {
    console.log(`Querying contract info for: ${contractAddress}`);
    const client = await createClient();

    const contractInfo = await executeQueryWithTimeout(() =>
      client.query.compute.contractInfo({
        contract_address: contractAddress || CONTRACT_INFO.REGISTRATION.ADDRESS,
      })
    );

    console.log("Contract Info:", JSON.stringify(contractInfo, null, 2));
    return contractInfo;
  } catch (error) {
    console.error("Error querying contract info:", error);
    process.exit(1);
  }
}

/**
 * Get a contract's code hash
 */
async function getCodeHash(contractAddress) {
  try {
    console.log(`Getting code hash for contract: ${contractAddress}`);
    const client = await createClient();

    const result = await executeQueryWithTimeout(() =>
      client.query.compute.codeHashByContractAddress({
        contract_address: contractAddress,
      })
    );

    console.log(`Code hash for contract ${contractAddress}:`, result.code_hash);
    return result.code_hash;
  } catch (error) {
    console.error("Error getting code hash:", error);
    process.exit(1);
  }
}

// Main execution function
async function main() {
  const args = process.argv.slice(2);
  const command = args[0];

  if (!command) {
    console.log(`
      Usage: node query.js <command> [args]
      
      Commands:
        state [address] [hash]    - Query contract state
        info [address]            - Query contract info
        status                    - Query node status
        hash <address>            - Get contract code hash
    `);
    process.exit(0);
  }

  try {
    if (command === "state") {
      const address = args[1] || CONTRACT_INFO.REGISTRATION.ADDRESS;
      const hash = args[2] || CONTRACT_INFO.REGISTRATION.HASH;
      await queryContractState(address, hash);
    } else if (command === "info") {
      const address = args[1] || CONTRACT_INFO.REGISTRATION.ADDRESS;
      await queryContractInfo(address);
    } else if (command === "status") {
      await queryNodeStatus();
    } else if (command === "hash") {
      if (!args[1]) {
        console.error("Error: Contract address is required for hash command");
        process.exit(1);
      }
      await getCodeHash(args[1]);
    } else {
      console.log(`Unknown command: ${command}`);
    }
  } catch (error) {
    console.error("Error executing command:", error);
    process.exit(1);
  }
}

// Execute main function if script is run directly
if (process.argv[1] === fileURLToPath(import.meta.url)) {
  main()
    .then(() => console.log("Script completed successfully"))
    .catch((error) => {
      console.error("Script failed:", error);
      process.exit(1);
    });
}

// Export functions for use in other scripts
export { queryContractState, queryNodeStatus, queryContractInfo, getCodeHash };
