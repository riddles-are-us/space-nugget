import mongoose from 'mongoose';

export class IndexedObject {
    // token idx
    index: number;
    // 40-character hexadecimal Ethereum address
    data: bigint[];

    constructor(index: number, data: bigint[]) {
        this.index = index;
        this.data = data;
    }

    static fromMongooseDoc(doc: mongoose.Document): IndexedObject {
        const obj = doc.toObject({
            transform: (doc, ret) => {
                delete ret._id;
                return ret;
            }
        });
        return new IndexedObject(obj.index, obj.data);
    }

    toMongooseDoc(): mongoose.Document {
        return new IndexedObjectModel(this.toObject());
    }

    toObject(): { index: number, data: string[], bidder: string[] | null} {
        let bidder = null;
        if (this.data[6] != 0n) {
          bidder = [this.data[7].toString(), this.data[8].toString()];
        }
        return {
            bidder: bidder,
            index: this.index,
            data: this.data.map((x) => x.toString()),
        };
    }

    toJSON() {
      const iobj = this.toObject();
      let bidder = null;
      if (iobj.bidder != null) {
        bidder = {
          bidder: [iobj.bidder[0], iobj.bidder[1]],
          bidprice: Number(iobj.data[6]),
        }
      }

      return  {
        id: Number(iobj.index),
        attributes: iobj.data[1].toString(),
        cycle: Number(iobj.data[2]),
        feature: Number(iobj.data[3]),
        sysprice: Number(iobj.data[4]),
        askprice: Number(iobj.data[5]),
        bid: bidder,
      }
    }

    static fromEvent(data: BigUint64Array): IndexedObject {
        return new IndexedObject(Number(data[0]),  Array.from(data.slice(1)))
    }
}

// Define the schema for the Token model
const indexedObjectSchema = new mongoose.Schema({
    index: { type: Number, required: true, unique: true},
    bidder:  {
      type: [String],
      required: false,
    },
    data: {
        type: [String],
        required: true,
    },
});

// Create the Token model
export const IndexedObjectModel = mongoose.model('IndexedObject', indexedObjectSchema);
