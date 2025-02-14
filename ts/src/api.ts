import { ZKWasmAppRpc, PlayerConvention, createCommand } from "zkwasm-minirollup-rpc";
import BN from "bn.js";



// for withdraw
const address = "c177d1d314C8FFe1Ea93Ca1e147ea3BE0ee3E470";
const amount = 1n;

const rpc = new ZKWasmAppRpc("http://localhost:3000");

export class Player extends PlayerConvention {
  constructor(key: string, rpc: ZKWasmAppRpc, deposit: bigint, withdraw: bigint) {
    super(key, rpc, deposit, withdraw);
  }

  async runCommand(command: bigint, nonce: bigint, params: bigint[]) {
    try {
      let result = await rpc.sendTransaction(createCommand(nonce, command, params), this.processingKey);
      console.log(JSON.stringify(result, null, 2));
      return result;
    } catch(e) {
      let reason = "";
      if (e instanceof Error) {
        reason = e.message;
      }
      console.log("command error:", reason);
    }
  }
}
