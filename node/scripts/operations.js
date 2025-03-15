import { fileURLToPath } from "url";
import { MsgExecuteContract } from "secretjs";
import { createClient, CONTRACT_INFO } from "../utils/client.js";

/**
 * Updates the contract's configuration
 */
async function updateContractConfig(contractAddress, codeHash) {
  try {
    console.log("Starting update contract configuration...");
    const client = await createClient();

    // Define contract configuration
    const contractConfig = {
      registration_address: "secret1ktpxcznqcls64t8tjyv3atwhndscgw08yp2jas",
      registration_wallet: client.address,
      contract_manager: client.address,
      max_registrations: 50,
      anml_token_contract: CONTRACT_INFO.ANML.ADDRESS,
      anml_token_hash: CONTRACT_INFO.ANML.HASH,
      erth_token_contract: CONTRACT_INFO.ERTH.ADDRESS,
      erth_token_hash: CONTRACT_INFO.ERTH.HASH,
      anml_pool_contract: CONTRACT_INFO.ANML_POOL.ADDRESS,
      anml_pool_hash: CONTRACT_INFO.ANML_POOL.HASH,
    };

    // Create the message
    const executeMsg = new MsgExecuteContract({
      sender: client.address,
      contract_address: contractAddress || CONTRACT_INFO.REGISTRATION.ADDRESS,
      code_hash: codeHash || CONTRACT_INFO.REGISTRATION.HASH,
      msg: {
        update_config: {
          config: contractConfig,
        },
      },
    });

    // Execute the transaction
    console.log("Executing update config transaction...");
    const resp = await client.tx.broadcast([executeMsg], {
      gasLimit: 1_000_000,
      gasPriceInFeeDenom: 0.1,
      feeDenom: "uscrt",
    });

    if (resp.code !== 0) {
      throw new Error(`Transaction failed: ${resp.rawLog}`);
    }

    console.log("Config updated successfully:", resp.transactionHash);
    return resp;
  } catch (error) {
    console.error("Error updating contract configuration:", error);
    process.exit(1);
  }
}

/**
 * Adds an allocation to the contract
 */
async function addAllocation(
  contractAddress,
  codeHash,
  receiverAddr,
  receiverHash,
  managerAddr,
  claimerAddr,
  useSend
) {
  try {
    console.log("Starting add allocation...");
    const client = await createClient();

    // Create the message
    const msg = {
      add_allocation: {
        recieve_addr:
          receiverAddr || contractAddress || CONTRACT_INFO.REGISTRATION.ADDRESS,
        recieve_hash:
          receiverHash || codeHash || CONTRACT_INFO.REGISTRATION.HASH,
        manager_addr: managerAddr || client.address,
        claimer_addr: claimerAddr || null,
        use_send: useSend !== undefined ? useSend : true,
      },
    };

    const executeMsg = new MsgExecuteContract({
      sender: client.address,
      contract_address: contractAddress || CONTRACT_INFO.REGISTRATION.ADDRESS,
      code_hash: codeHash || CONTRACT_INFO.REGISTRATION.HASH,
      msg: msg,
    });

    // Execute the transaction
    console.log("Executing add allocation transaction...");
    const resp = await client.tx.broadcast([executeMsg], {
      gasLimit: 1_000_000,
      gasPriceInFeeDenom: 0.1,
      feeDenom: "uscrt",
    });

    console.log("Allocation added successfully:", resp.transactionHash);
    return resp;
  } catch (error) {
    console.error("Error adding allocation:", error);
    process.exit(1);
  }
}

/**
 * Claims an allocation
 */
async function claimAllocation(contractAddress, codeHash, allocationId) {
  try {
    console.log(`Starting claim allocation for ID: ${allocationId}...`);
    const client = await createClient();

    // Create the message
    const msg = {
      claim_allocation: {
        allocation_id: allocationId,
      },
    };

    const executeMsg = new MsgExecuteContract({
      sender: client.address,
      contract_address: contractAddress || CONTRACT_INFO.REGISTRATION.ADDRESS,
      code_hash: codeHash || CONTRACT_INFO.REGISTRATION.HASH,
      msg: msg,
    });

    // Execute the transaction
    console.log("Executing claim allocation transaction...");
    const resp = await client.tx.broadcast([executeMsg], {
      gasLimit: 1_000_000,
      gasPriceInFeeDenom: 0.1,
      feeDenom: "uscrt",
    });

    console.log("Allocation claimed successfully:", resp.transactionHash);
    return resp;
  } catch (error) {
    console.error("Error claiming allocation:", error);
    process.exit(1);
  }
}

/**
 * Sets allocation percentages
 */
async function setAllocationPercentages(
  contractAddress,
  codeHash,
  allocations
) {
  try {
    console.log("Starting set allocation percentages...");
    const client = await createClient();

    // Create the message
    const msg = {
      set_allocation: {
        percentages: allocations || [
          {
            allocation_id: 1,
            percentage: "100",
          },
        ],
      },
    };

    const executeMsg = new MsgExecuteContract({
      sender: client.address,
      contract_address: contractAddress || CONTRACT_INFO.REGISTRATION.ADDRESS,
      code_hash: codeHash || CONTRACT_INFO.REGISTRATION.HASH,
      msg: msg,
    });

    // Execute the transaction
    console.log("Executing set allocation percentages transaction...");
    const resp = await client.tx.broadcast([executeMsg], {
      gasLimit: 1_000_000,
      gasPriceInFeeDenom: 0.1,
      feeDenom: "uscrt",
    });

    console.log(
      "Allocation percentages set successfully:",
      resp.transactionHash
    );
    return resp;
  } catch (error) {
    console.error("Error setting allocation percentages:", error);
    process.exit(1);
  }
}

/**
 * Edits an allocation
 */
async function editAllocation(contractAddress, codeHash, allocationId, config) {
  try {
    console.log(`Starting edit allocation for ID: ${allocationId}...`);
    const client = await createClient();

    // Create config for the allocation
    const allocationConfig = config || {
      receive_addr: contractAddress || CONTRACT_INFO.REGISTRATION.ADDRESS,
      receive_hash: codeHash || CONTRACT_INFO.REGISTRATION.HASH,
      manager_addr: client.address,
      claimer_addr: null,
      use_send: true,
    };

    // Create the message
    const msg = {
      edit_allocation: {
        allocation_id: allocationId,
        config: allocationConfig,
      },
    };

    const executeMsg = new MsgExecuteContract({
      sender: client.address,
      contract_address: contractAddress || CONTRACT_INFO.REGISTRATION.ADDRESS,
      code_hash: codeHash || CONTRACT_INFO.REGISTRATION.HASH,
      msg: msg,
    });

    // Execute the transaction
    console.log("Executing edit allocation transaction...");
    const resp = await client.tx.broadcast([executeMsg], {
      gasLimit: 1_000_000,
      gasPriceInFeeDenom: 0.1,
      feeDenom: "uscrt",
    });

    console.log("Allocation edited successfully:", resp.transactionHash);
    return resp;
  } catch (error) {
    console.error("Error editing allocation:", error);
    process.exit(1);
  }
}

/**
 * Adds a minter to the ANML token contract
 */
async function addMinter(minterAddress) {
  try {
    console.log(`Starting add minter: ${minterAddress}...`);
    const client = await createClient();

    // Create the message
    const msg = new MsgExecuteContract({
      sender: client.address,
      contract_address: CONTRACT_INFO.ANML.ADDRESS,
      code_hash: CONTRACT_INFO.ANML.HASH,
      msg: {
        set_minters: {
          minters: [minterAddress || CONTRACT_INFO.REGISTRATION.ADDRESS],
        },
      },
    });

    // Execute the transaction
    console.log("Executing add minter transaction...");
    const resp = await client.tx.broadcast([msg], {
      gasLimit: 1_000_000,
      gasPriceInFeeDenom: 0.1,
      feeDenom: "uscrt",
    });

    console.log("Minter added successfully:", resp.transactionHash);
    return resp;
  } catch (error) {
    console.error("Error adding minter:", error);
    process.exit(1);
  }
}

// Main execution function
async function main() {
  const args = process.argv.slice(2);
  const command = args[0];

  if (!command) {
    console.log(`
      Usage: node operations.js <command> [args]
      
      Commands:
        update-config [address] [hash]         - Update contract configuration
        add-allocation [address] [hash]        - Add allocation
        claim-allocation <id> [address] [hash] - Claim allocation
        set-allocation [address] [hash]        - Set allocation percentages
        edit-allocation <id> [address] [hash]  - Edit allocation
        add-minter [address]                   - Add a minter to ANML token
    `);
    process.exit(0);
  }

  try {
    if (command === "update-config") {
      const address = args[1] || CONTRACT_INFO.REGISTRATION.ADDRESS;
      const hash = args[2] || CONTRACT_INFO.REGISTRATION.HASH;
      await updateContractConfig(address, hash);
    } else if (command === "add-allocation") {
      const address = args[1] || CONTRACT_INFO.REGISTRATION.ADDRESS;
      const hash = args[2] || CONTRACT_INFO.REGISTRATION.HASH;
      await addAllocation(address, hash);
    } else if (command === "claim-allocation") {
      if (!args[1]) {
        console.error(
          "Error: Allocation ID is required for claim-allocation command"
        );
        process.exit(1);
      }
      const id = parseInt(args[1]);
      const address = args[2] || CONTRACT_INFO.REGISTRATION.ADDRESS;
      const hash = args[3] || CONTRACT_INFO.REGISTRATION.HASH;
      await claimAllocation(address, hash, id);
    } else if (command === "set-allocation") {
      const address = args[1] || CONTRACT_INFO.REGISTRATION.ADDRESS;
      const hash = args[2] || CONTRACT_INFO.REGISTRATION.HASH;
      await setAllocationPercentages(address, hash);
    } else if (command === "edit-allocation") {
      if (!args[1]) {
        console.error(
          "Error: Allocation ID is required for edit-allocation command"
        );
        process.exit(1);
      }
      const id = parseInt(args[1]);
      const address = args[2] || CONTRACT_INFO.REGISTRATION.ADDRESS;
      const hash = args[3] || CONTRACT_INFO.REGISTRATION.HASH;
      await editAllocation(address, hash, id);
    } else if (command === "add-minter") {
      const address = args[1] || CONTRACT_INFO.REGISTRATION.ADDRESS;
      await addMinter(address);
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
export {
  updateContractConfig,
  addAllocation,
  claimAllocation,
  setAllocationPercentages,
  editAllocation,
  addMinter,
};
