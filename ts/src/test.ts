import { Player } from "./api.js";
//import { LeHexBN, ZKWasmAppRpc} from "zkwasm-minirollup-rpc";
import { LeHexBN, ZKWasmAppRpc} from "zkwasm-ts-server";
import { createAsyncThunk } from '@reduxjs/toolkit';

const CREATE_PLAYER = 1n;
const SHAKE_FEET = 2n;
const JUMP = 3n;
const SHAKE_HEADS = 4n;
const POST_COMMENTS = 5n;
const LOTTERY = 6n;
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
    let r = await player.rpc.query_config();
    console.log(r);
    console.log("Start run CREATE_PLAYER...");
    await player.runCommand(CREATE_PLAYER, 0n, []);

    /*
    console.log("Start run SHAKE_FEET...");
    await player.runCommandAndCheckState(SHAKE_FEET);
    await delay(10000); // Wait for 10 seconds/2 ticks

    console.log("Start run JUMP...");
    await player.runCommandAndCheckState(JUMP);
    await delay(10000); // Wait for 10 seconds/2 ticks

    console.log("Start run SHAKE_HEADS...");
    await player.runCommandAndCheckState(SHAKE_HEADS);
    await delay(10000); // Wait for 10 seconds/2 ticks

    console.log("Start run POST_COMMENTS...");
    await player.runCommandAndCheckState(POST_COMMENTS);
    await delay(10000); // Wait for 10 seconds/2 ticks

    // Run extra 16 actions to test lottery
    for(let i = 0; i < 16; i++) {
      await player.runCommandAndCheckState(JUMP);
      await delay(10000); // Wait for 10 seconds/2 ticks
    }
    console.log("Run extra actions done!");
    console.log("Start run LOTTERY...");
    await player.runCommandAndCheckState(LOTTERY);

    console.log("Start run WITHDRAW...");
    await player.runCommandAndCheckState(WITHDRAW);
    */
}

main();
