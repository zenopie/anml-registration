import { fileURLToPath } from "url";
import { Wallet, SecretNetworkClient } from "secretjs";
import * as dotenv from "dotenv";
import * as fs from "fs";
import {
  createClient,
  executeQueryWithTimeout,
  CONTRACT_INFO,
} from "./utils/client.js";
// Import dns and https modules using ES module syntax
import * as dns from "dns";
import * as https from "https";

// Load environment variables
dotenv.config();

/**
 * Comprehensive network diagnostics tool
 *
 * This script runs a series of tests to diagnose issues with:
 * - Network connectivity
 * - Node responsiveness
 * - Contract availability
 * - Configuration correctness
 *
 * Usage: node diagnose.js
 */

// Helper function to format elapsed time
const formatTime = (startTime) => {
  const elapsed = Date.now() - startTime;
  return `${elapsed}ms`;
};

// Print a test result
const printResult = (testName, success, elapsed, details = null) => {
  console.log(`${success ? "✅" : "❌"} ${testName} [${elapsed}]`);
  if (details) {
    if (typeof details === "object") {
      console.log(
        `   ${JSON.stringify(details, null, 2).replace(/\n/g, "\n   ")}`
      );
    } else {
      console.log(`   ${details}`);
    }
  }
};

// Run a test with timeout protection
const runTest = async (testName, testFn, timeoutMs = 10000) => {
  console.log(`🔍 Running test: ${testName}...`);
  const startTime = Date.now();

  try {
    // Set up timeout protection
    const timeoutPromise = new Promise((_, reject) =>
      setTimeout(
        () => reject(new Error(`Test timed out after ${timeoutMs}ms`)),
        timeoutMs
      )
    );

    // Race the test against the timeout
    const result = await Promise.race([testFn(), timeoutPromise]);
    printResult(testName, true, formatTime(startTime), result);
    return true;
  } catch (error) {
    printResult(testName, false, formatTime(startTime), error.message);
    return false;
  }
};

// Basic test to check environment variables
const checkEnvironmentVars = async () => {
  const results = {};

  if (!process.env.MNEMONIC) {
    results.mnemonic = "Missing MNEMONIC in environment variables";
  } else {
    try {
      const wallet = new Wallet(process.env.MNEMONIC);
      results.wallet = {
        address: wallet.address,
        status: "Valid",
      };
    } catch (error) {
      results.wallet = {
        error: "Invalid mnemonic",
        details: error.message,
      };
    }
  }

  results.url =
    process.env.ERTH_URL || "Using default: https://lcd.erth.network";
  results.chainId = process.env.CHAIN_ID || "Using default: secret-4";

  return results;
};

// Test basic DNS resolution for the API endpoint
const checkDnsResolution = async () => {
  const url = process.env.ERTH_URL || "https://lcd.erth.network";
  const hostname = new URL(url).hostname;

  return new Promise((resolve, reject) => {
    dns.lookup(hostname, (err, address) => {
      if (err) {
        reject(
          new Error(`DNS resolution failed for ${hostname}: ${err.message}`)
        );
      } else {
        resolve({ hostname, ipAddress: address });
      }
    });
  });
};

// Test basic HTTP connectivity to the node
const checkHttpConnectivity = async () => {
  const url = process.env.ERTH_URL || "https://lcd.erth.network";

  return new Promise((resolve, reject) => {
    const req = https.get(
      `${url}/node_info`,
      {
        timeout: 5000,
      },
      (res) => {
        let data = "";
        res.on("data", (chunk) => {
          data += chunk;
        });

        res.on("end", () => {
          if (res.statusCode >= 200 && res.statusCode < 300) {
            try {
              const jsonData = JSON.parse(data);
              resolve({
                statusCode: res.statusCode,
                nodeInfo: jsonData,
              });
            } catch (error) {
              reject(new Error(`Invalid JSON response: ${error.message}`));
            }
          } else {
            reject(
              new Error(
                `HTTP request failed with status code: ${res.statusCode}`
              )
            );
          }
        });
      }
    );

    req.on("error", (error) => {
      reject(new Error(`HTTP request error: ${error.message}`));
    });

    req.on("timeout", () => {
      req.destroy();
      reject(new Error("HTTP request timed out"));
    });
  });
};

// Initialize SecretNetworkClient
const initializeClient = async () => {
  const client = await createClient();

  // Return basic client info to verify it's working
  return {
    address: client.address,
    url: client.url,
    chainId: client.chainId,
  };
};

// Query node info
const queryNodeInfo = async () => {
  const client = await createClient();

  const nodeInfo = await executeQueryWithTimeout(() =>
    client.query.tendermint.getNodeInfo({})
  );

  return {
    network: nodeInfo.default_node_info.network,
    version: nodeInfo.application_version.version,
    moniker: nodeInfo.default_node_info.moniker,
  };
};

// Query node status
const queryLatestBlock = async () => {
  const client = await createClient();

  const status = await executeQueryWithTimeout(() =>
    client.query.tendermint.getLatestBlock({})
  );

  return {
    height: status.block.header.height,
    time: status.block.header.time,
    numTxs: status.block.data.txs ? status.block.data.txs.length : 0,
  };
};

// Test contract query
const queryRegistrationContract = async () => {
  const client = await createClient();

  const result = await executeQueryWithTimeout(() =>
    client.query.compute.queryContract({
      contract_address: CONTRACT_INFO.REGISTRATION.ADDRESS,
      code_hash: CONTRACT_INFO.REGISTRATION.HASH,
      query: { query_state: {} },
    })
  );

  return result;
};

// Run all diagnostics
async function runAllDiagnostics() {
  console.log(
    "\n🛠️  Starting ANML Registration Contract Network Diagnostics Tool"
  );
  console.log("=".repeat(70));
  console.log(`Date: ${new Date().toISOString()}`);
  console.log(`Node.js: ${process.version}`);
  console.log("=".repeat(70));

  // Phase 1: Check environment setup
  console.log("\n📋 PHASE 1: Environment Configuration Check");
  await runTest("Environment Variables", checkEnvironmentVars);

  // Phase 2: Network connectivity
  console.log("\n📡 PHASE 2: Network Connectivity Tests");
  const dnsOk = await runTest("DNS Resolution", checkDnsResolution);
  const httpOk =
    dnsOk && (await runTest("HTTP Connectivity", checkHttpConnectivity));

  // Phase 3: Secret Network client
  console.log("\n🔐 PHASE 3: Secret Network Client Tests");
  const clientOk =
    httpOk && (await runTest("Client Initialization", initializeClient));

  // Phase 4: Node queries
  console.log("\n🔍 PHASE 4: Node Status Tests");
  if (clientOk) {
    await runTest("Node Info Query", queryNodeInfo);
    await runTest("Latest Block Query", queryLatestBlock);
  } else {
    console.log(
      "❌ Skipping node queries due to client initialization failure"
    );
  }

  // Phase 5: Contract queries
  console.log("\n📄 PHASE 5: Contract Query Tests");
  if (clientOk) {
    await runTest(
      "Registration Contract Query",
      queryRegistrationContract,
      15000
    );
  } else {
    console.log(
      "❌ Skipping contract queries due to client initialization failure"
    );
  }

  console.log("\n=".repeat(70));
  console.log("🏁 Diagnostics completed!");
  console.log("=".repeat(70));
}

// Execute diagnostics if script is run directly
if (process.argv[1] === fileURLToPath(import.meta.url)) {
  runAllDiagnostics()
    .then(() => console.log("\nDiagnostics completed successfully"))
    .catch((error) => {
      console.error("\nDiagnostics failed:", error);
      process.exit(1);
    });
}
