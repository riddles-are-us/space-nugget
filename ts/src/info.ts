import mongoose from 'mongoose';

export class IndexedObject {
    // token idx
    index: number;
    // 40-character hexadecimal Ethereum address
    data: bigint[];

    constructor(index: number, data: bigint[]) {
        this.index = index ;
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
        return new IndexedObjectModel({
            index: this.index,
            data: this.data,
        });
    }

    toObject(): { index: number, data: bigint[]} {
        return {
            index: this.index,
            data: this.data,
        };
    }

    static fromObject(obj: { index: number, data: bigint[]}): IndexedObject {
        return new IndexedObject(obj.index, obj.data);
    }

    toJSON() {
        return this.toObject();
    }

    static fromEvent(data: BigUint64Array): IndexedObject {
        return new IndexedObject(Number(data[0]),  Array.from(data.slice(1)))
    }
}

export function parseMemeInfo(iobj: IndexedObject) {
  console.log(iobj);
  return  {
    id: Number(iobj.index),
    rank: Number(iobj.data[1]),
    stake: Number(iobj.data[2]),
    pid: [Number(iobj.data[3]), Number(iobj.data[4])]
  }
}



// Define the schema for the Token model
const indexedObjectSchema = new mongoose.Schema({
    index: { type: Number, required: true, unique: true},
    data: {
        type: [BigInt],
        required: true,
    },
});

// Create the Token model
export const IndexedObjectModel = mongoose.model('IndexedObject', indexedObjectSchema);

export class Position {
    pid_1: bigint;
    pid_2: bigint;
    object_index: bigint;
    data: bigint[];
    constructor(pid_1: bigint, pid_2: bigint, object_index: bigint, data: bigint[]) {
        this.pid_1 = pid_1;
        this.pid_2 = pid_2;
        this.object_index = object_index;
        this.data = data;
    }

    static fromMongooseDoc(doc: mongoose.Document): Position {
        const obj = doc.toObject({
            transform: (doc, ret) => {
                delete ret._id;
                return ret;
            }
        });
        return new Position(obj.pid_1, obj.pid_2, obj.object_index, obj.data);
    }

    toMongooseDoc(): mongoose.Document {
        return new PositionModel({
            pid_1: this.pid_1,
            pid_2: this.pid_2,
            object_index: this.object_index,
            data: this.data,
        });
    }

    toObject(): { pid_1: bigint, pid_2: bigint, object_index: bigint, data: bigint[] } {
        return {
            pid_1: this.pid_1,
            pid_2: this.pid_2,
            object_index: this.object_index,
            data: this.data,
        };
    }

    static fromObject(obj: { pid_1: bigint, pid_2: bigint, object_index: bigint, data: bigint[]}): Position {
        return new Position(obj.pid_1, obj.pid_2, obj.object_index, obj.data);
    }

    toJSON() {
        return {
            pid_1: this.pid_1.toString(),
            pid_2: this.pid_2.toString(),
            object_index: this.object_index.toString(),
            data: this.data.toString()
        };
    }

    static fromJSON(obj: { pid_1: string, pid_2: string, object_index: string, data: string[]}): Position {
        return new Position(
            BigInt(obj.pid_1),
            BigInt(obj.pid_2),
            BigInt(obj.object_index),
            obj.data.map((x) => BigInt(x)),
        );
    }
    static fromEvent(data: BigUint64Array): Position {
        let bigintarray:bigint[] = Array.from(data);
        return new Position(data[0], data[1], data[2], bigintarray.slice(3));
    }

}

// 创建 Schema
const PositionSchema = new mongoose.Schema({
    pid_1: {
        type: BigInt,
        required: true
    },
    pid_2: {
        type: BigInt,
        required: true
    },
    object_index: {
        type: BigInt,
        required: true
    },
    data : {
        type: [BigInt],
        required: true
    },
});

// add composition index
PositionSchema.index(
    { pid_1: 1, pid_2: 1, object_index: 1 },
    { unique: true }
);

export const PositionModel = mongoose.model('Position', PositionSchema);

