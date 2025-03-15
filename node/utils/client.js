import { Wallet, SecretNetworkClient } from "secretjs";
import * as dotenv from "dotenv";
import * as fs from "fs";
import path from "path";

// Load environment variables
dotenv.config();

// Contract addresses and hashes
export const CONTRACT_INFO = {
  REGISTRATION: {
    ADDRESS: "secret12q72eas34u8fyg68k6wnerk2nd6l5gaqppld6p",
    HASH: "32cd885fe0753693d976ae405c1f098a1bacc49b6eceb7251fb31bacecfe5eb9",
  },
  ERTH: {
    ADDRESS: "secret16snu3lt8k9u0xr54j2hqyhvwnx9my7kq7ay8lp",
    HASH: "638a3e1d50175fbcb8373cf801565283e3eb23d88a9b7b7f99fcc5eb1e6b561e",
  },
  ANML: {
    ADDRESS: "secret14p6dhjznntlzw0yysl7p6z069nk0skv5e9qjut",
    HASH: "638a3e1d50175fbcb8373cf801565283e3eb23d88a9b7b7f99fcc5eb1e6b561e",
  },
  ANML_POOL: {
    ADDRESS: "secret1rj2phrf6x3v7526jrz60m2dcq58slyq2269kra",
    HASH: "3f15639c67a22ea023384d901820ddb67bb716bf4a119fa517c63e68b1432dbe",
  },
};

export const CODE_ID = 2211;

/**
 * Creates and returns a SecretNetworkClient instance with proper error handling
 * @returns {Promise<SecretNetworkClient>} The initialized SecretNetworkClient
 */
export async function createClient() {
  if (!process.env.MNEMONIC) {
    throw new Error("MNEMONIC not found in environment variables");
  }

  const url = process.env.ERTH_URL || "https://lcd.erth.network";
  const chainId = process.env.CHAIN_ID || "secret-4";

  console.log(`Connecting to ${url} (chain ID: ${chainId})`);

  try {
    // Create wallet from mnemonic
    const wallet = new Wallet(process.env.MNEMONIC);
    console.log(`Wallet initialized with address: ${wallet.address}`);

    // Initialize client
    const client = new SecretNetworkClient({
      url,
      chainId,
      wallet,
      walletAddress: wallet.address,
    });

    // Test connection
    const connectionTestPromise = client.query.tendermint.getNodeInfo({});

    // Set timeout for connection test
    const timeoutPromise = new Promise((_, reject) => {
      setTimeout(
        () => reject(new Error("Connection timed out after 15 seconds")),
        15000
      );
    });

    // Race the connection test against timeout
    const nodeInfo = await Promise.race([
      connectionTestPromise,
      timeoutPromise,
    ]);
    console.log(
      `Successfully connected to the network (Chain ID: ${nodeInfo.default_node_info.network})`
    );

    return client;
  } catch (error) {
    console.error("Failed to create SecretNetworkClient:", error.message);
    throw error;
  }
}

/**
 * Helper function to execute a query with timeout
 * @param {Function} queryFn The query function to execute
 * @param {number} timeout Timeout in milliseconds
 * @returns {Promise<any>} The query result
 */
export async function executeQueryWithTimeout(queryFn, timeout = 15000) {
  const queryPromise = queryFn();

  const timeoutPromise = new Promise((_, reject) => {
    setTimeout(
      () => reject(new Error(`Query timed out after ${timeout}ms`)),
      timeout
    );
  });

  return Promise.race([queryPromise, timeoutPromise]);
}

/**
 * Helper function to read contract wasm file
 * @returns {Buffer} The contract wasm file buffer
 */
export function readContractWasm() {
  try {
    // Adjust the path as needed for your project structure
    const wasmPath = path.resolve(__dirname, "../../contract.wasm.gz");
    return fs.readFileSync(wasmPath);
  } catch (error) {
    console.error("Error reading contract wasm file:", error.message);
    throw error;
  }
}
