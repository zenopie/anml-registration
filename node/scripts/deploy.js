import { fileURLToPath } from "url";
import {
  createClient,
  readContractWasm,
  CONTRACT_INFO,
  CODE_ID,
  executeQueryWithTimeout,
} from "../utils/client.js";

/**
 * Uploads the contract to the blockchain and returns the code ID
 */
async function uploadContract() {
  try {
    console.log("Starting contract upload...");
    const client = await createClient();
    const contractWasm = readContractWasm();

    console.log("Uploading contract...");
    const tx = await client.tx.compute.storeCode(
      {
        sender: client.address,
        wasm_byte_code: contractWasm,
        source: "",
        builder: "",
      },
      {
        gasLimit: 4_000_000,
      }
    );

    console.log("Upload transaction completed:", tx.transactionHash);

    const codeId = Number(
      tx.arrayLog.find((log) => log.type === "message" && log.key === "code_id")
        .value
    );

    console.log("Code ID:", codeId);

    const contractCodeHash = await executeQueryWithTimeout(() =>
      client.query.compute.codeHashByCodeId({ code_id: codeId })
    );

    console.log(`Contract hash: ${contractCodeHash.code_hash}`);
    return { codeId, codeHash: contractCodeHash.code_hash };
  } catch (error) {
    console.error("Error uploading contract:", error);
    process.exit(1);
  }
}

/**
 * Instantiates a contract using the provided code ID and hash
 */
async function instantiateContract(codeId, codeHash) {
  try {
    console.log("Starting contract instantiation...");
    const client = await createClient();

    const initMsg = {
      registration_address: "secret1ktpxcznqcls64t8tjyv3atwhndscgw08yp2jas",
      registration_wallet: client.address,
      contract_manager: client.address,
      anml_token_contract: CONTRACT_INFO.ANML.ADDRESS,
      anml_token_hash: CONTRACT_INFO.ANML.HASH,
      erth_token_contract: CONTRACT_INFO.ERTH.ADDRESS,
      erth_token_hash: CONTRACT_INFO.ERTH.HASH,
      anml_pool_contract: CONTRACT_INFO.ANML_POOL.ADDRESS,
      anml_pool_hash: CONTRACT_INFO.ANML_POOL.HASH,
    };

    console.log(
      "Instantiating contract with init message:",
      JSON.stringify(initMsg, null, 2)
    );

    const tx = await client.tx.compute.instantiateContract(
      {
        code_id: codeId || CODE_ID,
        sender: client.address,
        code_hash: codeHash || CONTRACT_INFO.REGISTRATION.HASH,
        init_msg: initMsg,
        label: "animal registration " + Math.ceil(Math.random() * 10000),
        admin: client.address,
      },
      {
        gasLimit: 6_000_000,
      }
    );

    console.log("Instantiation transaction completed:", tx.transactionHash);

    const contractAddress = tx.arrayLog.find(
      (log) => log.type === "message" && log.key === "contract_address"
    ).value;

    console.log("Contract address:", contractAddress);
    return { contractAddress };
  } catch (error) {
    console.error("Error instantiating contract:", error);
    process.exit(1);
  }
}

/**
 * Migrates a contract to a new code ID
 */
async function migrateContract(contractAddress, codeId, codeHash) {
  try {
    console.log(`Starting contract migration for ${contractAddress}...`);
    const client = await createClient();

    const migrateMsg = { migrate: {} };

    console.log(`Migrating contract to code ID ${codeId}...`);

    const migrationTx = await client.tx.compute.migrateContract(
      {
        sender: client.address,
        contract_address: contractAddress || CONTRACT_INFO.REGISTRATION.ADDRESS,
        code_id: codeId || CODE_ID,
        code_hash: codeHash || CONTRACT_INFO.REGISTRATION.HASH,
        msg: migrateMsg,
        sent_funds: [],
      },
      {
        gasLimit: 2_000_000,
      }
    );

    console.log(
      "Migration transaction completed:",
      migrationTx.transactionHash
    );
    return { success: true, txHash: migrationTx.transactionHash };
  } catch (error) {
    console.error("Error migrating contract:", error);
    process.exit(1);
  }
}

// Main execution function to be called when script is run directly
async function main() {
  const args = process.argv.slice(2);
  const command = args[0];

  if (!command) {
    console.log(`
      Usage: node deploy.js <command> [args]
      
      Commands:
        upload              - Upload the contract
        instantiate         - Instantiate a contract (run after upload)
        migrate [address]   - Migrate a contract to new code ID
    `);
    process.exit(0);
  }

  try {
    if (command === "upload") {
      await uploadContract();
    } else if (command === "instantiate") {
      const codeId = args[1] ? parseInt(args[1]) : CODE_ID;
      const hash = args[2] || CONTRACT_INFO.REGISTRATION.HASH;
      await instantiateContract(codeId, hash);
    } else if (command === "migrate") {
      const address = args[1] || CONTRACT_INFO.REGISTRATION.ADDRESS;
      const codeId = args[2] ? parseInt(args[2]) : CODE_ID;
      const hash = args[3] || CONTRACT_INFO.REGISTRATION.HASH;
      await migrateContract(address, codeId, hash);
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
export { uploadContract, instantiateContract, migrateContract };
