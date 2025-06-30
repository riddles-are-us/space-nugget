import { Player, commandSpec } from "./api.js";
//import { LeHexBN, ZKWasmAppRpc} from "zkwasm-minirollup-rpc";
import { LeHexBN, query, ZKWasmAppRpc } from "zkwasm-ts-server";
import { createAsyncThunk } from "@reduxjs/toolkit";

let account = "1234";
let account_bid = "5678";

const rpc: any = new ZKWasmAppRpc("http://127.0.0.1:3000");
let player = new Player(
  account,
  rpc,
  commandSpec.DEPOSIT.id,
  commandSpec.WITHDRAW.id
);

async function main() {
  const pubkey = new LeHexBN(query(account).pkx).toU64Array();
  console.log(pubkey);

  let r = await player.rpc.queryConfig();
  console.log("config:", r);

  console.log("Start run CREATE_PLAYER...");
  await player.runCommand(commandSpec.INSTALL_PLAYER.id, 0n, []);

  console.log("Start run CREATE_NUGGET ...");
  let nonce = await player.getNonce();
  await player.runCommand(commandSpec.CREATE_NUGGET.id, nonce, []);

  console.log("Start run LIST_NUGGET ...");
  nonce = await player.getNonce();
  await player.runCommand(commandSpec.RECYCLE_NUGGET.id, nonce, [0n]);

  console.log("Start run CLAIM_REWARD ...");
  nonce = await player.getNonce();
  await player.runCommand(commandSpec.CLAIM_REWARD.id, nonce, [0n]);

  console.log("Start run query auction...");
  try {
    let data = await player.getState();
    console.log(data.state.leaderboard.nuggets.length);
  } catch (e) {
    console.log(e);
  }
}

main();
