import { Wallet, SecretNetworkClient, MsgExecuteContract} from "secretjs";
import * as fs from "fs";

const wallet = new Wallet(
  "",
);

const url = "https://api.pulsar.scrttestnet.com";
//const url = "https://lcd.testnet.secretsaturn.net";

const secretjs = new SecretNetworkClient({
  url,
  chainId: "pulsar-3",
  wallet: wallet,
  walletAddress: wallet.address,
});

const ERTH_CONTRACT = "secret12wcgts3trvzccyns4s632neqguqsfzv4p0jgxn";
const ERTH_HASH = "55bac6db7ea861e9c59c2d4429623a7b445838fed0b8fd5b4d8de10fa4fb6fe7";
const ANML_CONTRACT = "secret1hsn3045l5eztd8xdeqly67wfver5gh7c7267pk";
const ANML_HASH = "55bac6db7ea861e9c59c2d4429623a7b445838fed0b8fd5b4d8de10fa4fb6fe7";
const PROTOCOL_CONTRACT =  "secret1fh2038x3p0tdz85vdvkl4lk7pkggl0zxndt03v";
const PROTOCOL_HASH =  "06a809264fcb5867effd2f8337c073376dd2349b9f1d969b57f66d8dcac8bffb";
const sscrt_contract = 'secret1p6r5zc8898c9h3zfssfxu2x75nz3t4z8q68w8t';
const sscrt_hash = 'c74bc4b0406507257ed033caa922272023ab013b0c74330efc16569528fa34fe';
const contract_wasm = fs.readFileSync("../contract.wasm.gz");
const codeId = 2330;


let upload_contract = async () => {
	try {
	  let tx = await secretjs.tx.compute.storeCode(
		{
		  sender: wallet.address,
		  wasm_byte_code: contract_wasm,
		  source: "",
		  builder: "",
		},
		{
		  gasLimit: 4000000, // Using the direct number for better compatibility
		}
	  );
  
	  const logEntry = tx.arrayLog.find((log) => log.type === "message" && log.key === "code_id");
  
	  if (!logEntry) {
		throw new Error("Unable to find code_id in the transaction logs.");
	  }
  
	  const codeId = Number(logEntry.value);
	  console.log("codeId: ", codeId);
  
	  const contractCodeHash = (
		await secretjs.query.compute.codeHashByCodeId({ code_id: codeId })
	  ).code_hash;
	  console.log(`Contract hash: ${contractCodeHash}`);
	} catch (error) {
	  console.error("Error uploading contract:", error.message);
	}
  };
  

  // upload_contract();




let instantiate_contract = async () => {
  const initMsg = {
      registration_address: wallet.address,
      anml_contract: ANML_CONTRACT,
      anml_hash: ANML_HASH,
      erth_contract: ERTH_CONTRACT,
      erth_hash: ERTH_HASH,
  };

  try {
      let tx = await secretjs.tx.compute.instantiateContract(
          {
              code_id: codeId,
              sender: wallet.address,
              code_hash: PROTOCOL_HASH,
              init_msg: initMsg,
              label: "secret raffle" + Math.ceil(Math.random() * 100000),
              admin: wallet.address,
          },
          {
              gasLimit: 1000000,
          }
      );
      //console.log(tx);

      if (!tx.arrayLog) {
          throw new Error("Transaction log is missing.");
      }

      // Find the contract_address in the logs
      const logEntry = tx.arrayLog.find(
          (log) => log.type === "message" && log.key === "contract_address"
      );

      if (!logEntry || !logEntry.value) {
          throw new Error("Contract address not found in the logs.");
      }

      const contractAddress = logEntry.value;
      console.log("contract address: ", contractAddress);

  } catch (err) {
      console.error("An error occurred while instantiating the contract:", err);
  }
};


  // instantiate_contract();




async function snip(){
	let hookmsg = {
    sell: {}
	};
	let hookmsg64 = btoa(JSON.stringify(hookmsg));
	let msg = new MsgExecuteContract({
		sender: secretjs.address,
		contract_address: ERTH_CONTRACT,
    	code_hash: ERTH_HASH,
		msg: {
			send: {
				recipient: PROTOCOL_CONTRACT,
        		code_hash: PROTOCOL_HASH,
				amount: "1000000000",
				msg: hookmsg64,
			}
		}
	});
	let resp = await secretjs.tx.broadcast([msg], {
		gasLimit: 1_000_000,
		gasPriceInFeeDenom: 0.1,
		feeDenom: "uscrt",
	});
	console.log(resp);
};

 // snip();

 async function initiate_pool(){
	let anml_msg = new MsgExecuteContract({
		sender: secretjs.address,
		contract_address: ANML_CONTRACT,
    code_hash: ANML_HASH,
		msg: {
      increase_allowance: {
        spender: PROTOCOL_CONTRACT,
        amount: "100000000",
      }
		}
	});
  let other_msg = new MsgExecuteContract({
		sender: secretjs.address,
		contract_address: ERTH_CONTRACT,
    code_hash: ERTH_HASH,
		msg: {
			increase_allowance: {
        spender: PROTOCOL_CONTRACT,
        amount: "100000000",
      }
		}
	});
  let pool_msg = new MsgExecuteContract({
		sender: secretjs.address,
		contract_address: PROTOCOL_CONTRACT,
    code_hash: PROTOCOL_HASH,
		msg: {
			initialize_pool: {
        other_contract: ERTH_CONTRACT,
        other_hash: ERTH_HASH,
        initial_anml: "100000000",
        initial_other: "100000000",
      }
		}
	});
  let msg_array = [anml_msg, other_msg, pool_msg];
	let resp = await secretjs.tx.broadcast(msg_array, {
		gasLimit: 1_000_000,
		gasPriceInFeeDenom: 0.1,
		feeDenom: "uscrt",
	});
	console.log(resp);
};

  // initiate_pool();

async function contract(){
	let msg = new MsgExecuteContract({
		sender: secretjs.address,
		contract_address: PROTOCOL_CONTRACT,
    	code_hash: PROTOCOL_HASH,
		msg: {
			withdraw_liquidity: {
        pool_id: ERTH_CONTRACT,
        //amount: "1000000"
      },
		}
	});
	let resp = await secretjs.tx.broadcast([msg], {
		gasLimit: 1_000_000,
		gasPriceInFeeDenom: 0.1,
		feeDenom: "uscrt",
	});
	console.log(resp);
};

 //contract();


async function query(){
	let tx = await secretjs.query.compute.queryContract({
	  contract_address: PROTOCOL_CONTRACT,
	  code_hash: PROTOCOL_HASH,
	  query: {
		  query_pool_info: {
			  pool_id: ERTH_CONTRACT,
		  },
	  }
	});
	console.log(tx);
};

//query();

async function querySscrt(){
  let sscrt_info = await secretjs.query.compute.queryContract({
    contract_address: sscrt_contract,
    code_hash: sscrt_hash,
    query: {
      balance: {
        address: lottery_contract,
        key: viewing_key,
        time : Date.now()
      }
    }
  });
  console.log(sscrt_info);
};

//querySscrt();


async function getCodeHash(){
	let tx = await secretjs.query.compute.codeHashByContractAddress({
		contract_address: sscrt_contract,
	});
	console.log(tx);
}

//getCodeHash();