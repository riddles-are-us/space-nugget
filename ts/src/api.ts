import { ZKWasmAppRpc, PlayerConvention } from "zkwasm-minirollup-rpc";
import BN from "bn.js";

const SWAY = 0n;
const CREATE_PLAYER = 1n;
const LOTTERY = 6n;
const WITHDRAW = 8n;

// for withdraw
const address = "c177d1d314C8FFe1Ea93Ca1e147ea3BE0ee3E470";
const amount = 1n;

function bytesToHex(bytes: Array<number>): string  {
  return Array.from(bytes, byte => byte.toString(16).padStart(2, '0')).join('');
}

const rpc = new ZKWasmAppRpc("http://localhost:3000");

export class Player extends PlayerConvention {
  constructor(key: string, rpc: ZKWasmAppRpc, deposit: bigint, withdraw: bigint) {
    super(key, rpc, deposit, withdraw);
  }

  async runCommand(command: bigint, nonce: bigint) {
    try {
      let processStamp = await rpc.sendTransaction(new BigUint64Array([super.createCommand(nonce, command, 0n), 0n, 0n, 0n]), this.processingKey);
      console.log("command processed at:", processStamp);
    } catch(e) {
      let reason = "";
      if (e instanceof Error) {
        reason = e.message;
      }
      console.log("command error:", reason);
    }
  }

  // Check whether the current state is as expected
  async checkState(nonce: bigint, action: bigint, balance: bigint) {
    try {
      let data= await this.getState();
      let nonce_after_command = data.player.nonce;
      let balance_after_command = data.player.data.balance;
      let action_after_command = data.player.data.action;

      if(nonce == BigInt(nonce_after_command) && action == BigInt(action_after_command) && balance == BigInt(balance_after_command)) {
          console.log("command works");
      } else {
          console.log("command failed. current state's nonce:", nonce_after_command, ", balance:", balance_after_command, ", action:", action_after_command);
      }
    } catch(e) {
      console.log("query state error:", e);
    }
  }

  async runCommandAndCheckState(command: bigint) {
    if(command == CREATE_PLAYER) {
      await this.runCommand(command, 0n);
      await this.checkState(0n, 0n, 0n);
    } else if(command == WITHDRAW){
      let balance = 9n // previous balance(10n) - amount(1n)
      let player:any = await super.withdrawRewards(address, amount);
      await this.checkState(player.nonce + 1n, SWAY, balance);
    } else {
      let data = await this.getState();
      let nonce_before_command = BigInt(data.player.nonce);
      await this.runCommand(command, BigInt(nonce_before_command));
      let balance = 0n;

      // Run lottery once on test.ts, so balance is 10, action become SWAY
      if(command == LOTTERY) {
        balance = 10n;
        await this.checkState(nonce_before_command + 1n, SWAY, balance);
      } else {
        await this.checkState(nonce_before_command + 1n, command, balance);
      }
    }
  }
}
