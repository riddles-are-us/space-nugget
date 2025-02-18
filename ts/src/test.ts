import { Player } from "./api.js";
//import { LeHexBN, ZKWasmAppRpc} from "zkwasm-minirollup-rpc";
import { LeHexBN, query, ZKWasmAppRpc} from "zkwasm-ts-server";
import { createAsyncThunk } from '@reduxjs/toolkit';

const INSTALL_PLAYER = 1n;
const EXPLORE_NUGGET = 4n;
const SELL_NUGGET = 5n;
const BID_NUGGET = 6n;
const CREATE_NUGGET = 7n;
const WITHDRAW = 8n;
const DEPOSIT = 9n;

let account = "1234";

const rpc:any = new ZKWasmAppRpc("http://127.0.0.1:3000");
let player = new Player(account, rpc, DEPOSIT, WITHDRAW);

// Function to pause execution for a given duration
function delay(ms: number) {
  return new Promise(resolve => setTimeout(resolve, ms));
}

async function main() {
  const pubkey = new LeHexBN(query(account).pkx).toU64Array();
  console.log(pubkey);

  let r = await player.rpc.queryConfig();
  console.log("config:", r);

  console.log("Start run CREATE_PLAYER...");
  await player.runCommand(INSTALL_PLAYER, 0n, []);

  let g = await player.getState();
  console.log("state.", g);


  console.log("Start run CREATE_NUGGET ...");
  let nonce = await player.getNonce();
  await player.runCommand(CREATE_NUGGET, nonce, []);
  await player.runCommand(CREATE_NUGGET, nonce, []);
  await player.runCommand(CREATE_NUGGET, nonce, []);
  await player.runCommand(CREATE_NUGGET, nonce, []);

  console.log("Start run EXPLORE_NUGGET ...");
  nonce = await player.getNonce();
  await player.runCommand(EXPLORE_NUGGET, nonce, [0n]);

  /*
  console.log("Start run BID_NUGGET ...");
  await player.getNonce();

  console.log("Start run query nuggets ...");
  try {
    let data = await player.rpc.queryData(`nuggets`);
    console.log(data);
  } catch(e) {
    console.log(e);
  }

  console.log("Start run query nugget ...");
  try {
    let data = await player.rpc.queryData(`nugget/1`);
    console.log(data);
  } catch(e) {
    console.log(e);
  }
  */
}

main();
