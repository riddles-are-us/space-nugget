//import initHostBind, * as hostbind from "./wasmbind/hostbind.js";
import { query, ZKWasmAppRpc, LeHexBN } from "zkwasm-ts-server";

const CMD_GUESS = 1n;

function createCommand(nonce: bigint, command: bigint, feature: bigint) {
  return (nonce << 16n) + (feature << 8n) + command;
}

let account = "1234";

const rpc = new ZKWasmAppRpc("http://localhost:3000");
//const rpc = new ZKWasmAppRpc("http://114.119.187.224:8085");


async function guess(num: bigint) {
  let accountInfo = new LeHexBN(query(account).pkx).toU64Array();
  console.log("account info:", accountInfo);
  try {
    let processStamp = await rpc.sendTransaction([createCommand(0n, CMD_GUESS, 0n), num, 0n, 0n], account);
    console.log("processed at:", processStamp);
  } catch(e) {
    if (e instanceof Error) {
      console.log(e.message);
    }
    console.log("process guess error at id:", e);
  }
}


async function main() {
  //sending_transaction([0n,0n,0n,0n], "1234");
  let x = 0n;
  await guess(x);
}

main();


