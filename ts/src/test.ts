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
const RECYCLE_NUGGET = 10n;
const LIST_NUGGET = 11n;

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

  console.log("Start run EXPLORE_NUGGET ...");
  nonce = await player.getNonce();
  await player.runCommand(EXPLORE_NUGGET, nonce, [0n]);

  console.log("Start run LIST_NUGGET ...");
  nonce = await player.getNonce();
  await player.runCommand(LIST_NUGGET, nonce, [0n, 10n]);

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

  let marketid = 0;

  console.log("Start run query markets...");
  try {
    let data:any = await player.rpc.queryData(`markets`);
    console.log(data);
    marketid = data.data[0].marketid;
    console.log("Start run BID_NUGGET ...");
    console.log(JSON.stringify(data.data[0]));
    nonce = await player.getNonce();
    await player.runCommand(BID_NUGGET, nonce, [BigInt(marketid), 8n]);
  } catch(e) {
    console.log(e);
  }

  console.log("Start run query my bids...", pubkey[0], pubkey[1]);
  try {
    let data:any = await player.rpc.queryData(`bid/${pubkey[1].toString()}/${pubkey[2].toString()}`);
    console.log(data);
  } catch(e) {
    console.log(e);
  }

  try {
    console.log("SELL NUGGET ...");
    nonce = await player.getNonce();
    await player.runCommand(SELL_NUGGET, nonce, [BigInt(marketid)]);
  } catch(e) {
    console.log(e);
  }

  console.log("Start run query markets...");
  try {
    let data:any = await player.rpc.queryData(`markets`);
    console.log(data);
  } catch(e) {
    console.log(e);
  }
}

main();
