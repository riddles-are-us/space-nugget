//import initHostBind, * as hostbind from "./wasmbind/hostbind.js";
import { query, LeHexBN, ZKWasmAppRpc } from "zkwasm-ts-server";

let account = "1234";

const rpc = new ZKWasmAppRpc("http://localhost:3000");
//const rpc = new ZKWasmAppRpc("http://114.119.187.224:8085");

async function main() {
  let state:any = await rpc.queryState(account);
  let data = JSON.parse(state.data);
  console.log("player info:", data);

  //let config = await rpc.query_config();
  //console.log("config", config);
}

main();


