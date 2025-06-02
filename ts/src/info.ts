import mongoose from 'mongoose';
import { Market, ObjectEvent } from 'zkwasm-ts-server';

(BigInt.prototype as any).toJSON = function () {
    return BigInt.asUintN(64, this).toString();
};

interface Nugget {
  id: bigint;
  attributes: bigint;
  cycle: bigint;
  feature: bigint;
  sysprice: bigint;
  marketid: bigint;
}

class NuggetDecoder implements ObjectEvent.Decodable<Nugget> {
  constructor() {
  }
  fromData(u64data: bigint[]): Nugget {
    const id: bigint = u64data.shift()!;
    const attributes: bigint = u64data.shift()!;
    const cycle: bigint = u64data.shift()!;
    const feature: bigint = u64data.shift()!;
    const sysprice: bigint = u64data.shift()!;
    const marketid: bigint = u64data.shift()!;
    return {
        id,
        attributes,
        cycle,
        feature,
        sysprice,
        marketid,
    }
  }
}


const NUGGET_INFO = 1;
const MARKET_INFO = 2;

export function docToJSON(doc: mongoose.Document) {
    console.log("doc...", doc);
    const obj = doc.toObject({
        transform: (_, ret:any) => {
            delete ret._id;
            return ret;
        }
    });
    return obj;
}

export class IndexedObject {
    // token idx
    index: number;
    // 40-character hexadecimal Ethereum address
    data: bigint[];

    constructor(index: number, data: bigint[]) {
        this.index = index;
        this.data = data;
    }

    toObject() {
        let decoder = new NuggetDecoder();
        if (this.index == NUGGET_INFO) {
            return decoder.fromData(this.data);
        } else if (this.index == MARKET_INFO) {
            return Market.fromData(this.data, decoder);
        } else {
            console.log("fatal, unexpected object index");
            process.exit();
        }
    }

    toJSON() {
      return JSON.stringify(this.toObject());
    }

    static fromEvent(data: BigUint64Array): IndexedObject {
        return new IndexedObject(Number(data[0]),  Array.from(data.slice(1)))
    }

    async storeRelatedObject() {
        let obj = this.toObject() as any;
        if (this.index == NUGGET_INFO) {
            let doc = await NuggetObjectModel.findOneAndUpdate({id: obj.id}, obj, {upsert: true});
            return doc;
        } else if (this.index == MARKET_INFO) {
            let doc = await MarketObjectModel.findOneAndUpdate({marketid: obj.marketid}, obj, {upsert: true});
            return doc;
        }

    }
}

// Define the schema for the Token model
const NuggetObjectSchema = new mongoose.Schema({
    id: { type: BigInt, required: true, unique: true},
    attributes: {type: BigInt, required: true},
    cycle: {type: BigInt, required: true},
    feature: {type: BigInt, required: true},
    sysprice: {type: BigInt, required: true},
    marketid: {type: BigInt, required: true},
});

const MarketObjectSchema = Market.createMarketSchema(NuggetObjectSchema);

NuggetObjectSchema.pre('init', ObjectEvent.uint64FetchPlugin);

// Create the Token model
export const MarketObjectModel = mongoose.model('MarketObject', MarketObjectSchema);
export const NuggetObjectModel = mongoose.model('NuggetObject', NuggetObjectSchema);
