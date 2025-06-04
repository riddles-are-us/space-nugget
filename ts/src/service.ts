import {
  TxWitness,
  Service,
  Event,
  EventModel,
  TxStateManager,
} from "zkwasm-ts-server";
import {
  NuggetObjectModel,
  MarketObjectModel,
  IndexedObject,
  docToJSON,
} from "./info.js";
import { Express } from "express";
import { merkleRootToBeHexString } from "zkwasm-ts-server/src/lib.js";
import mongoose from "mongoose";

const service = new Service(eventCallback, batchedCallback, extra);
await service.initialize();

let txStateManager = new TxStateManager(
  merkleRootToBeHexString(service.merkleRoot)
);

function extra(app: Express) {
  app.get("/data/nugget/:nid", async (req: any, res) => {
    try {
      let nid = req.params.nid;
      const doc = await NuggetObjectModel.find(
        //{index: Number(nid)},
        { id: nid }
      );
      let data = doc.map((d) => {
        return docToJSON(d);
      });
      res.status(201).send({
        success: true,
        data: data,
      });
    } catch (e) {
      console.log(e);
      res.status(500).send();
    }
  });

  app.get("/data/bid/:pid1/:pid2", async (req: any, res) => {
    try {
      let pid1 = req.params.pid1;
      let pid2 = req.params.pid2;
      const skip = parseInt(req.query.skip) || 0;
      const limit = parseInt(req.query.limit) || 30;
      const [count, doc] = await Promise.all([
        MarketObjectModel.countDocuments({
          settleinfo: { $ne: BigInt(2) },
          "bidder.bidder": [pid1, pid2],
        }),
        MarketObjectModel.find({
          settleinfo: { $ne: BigInt(2) },
          "bidder.bidder": [pid1, pid2],
        })
          .skip(skip)
          .limit(limit),
      ]);

      let data = doc.map((d: mongoose.Document) => {
        return docToJSON(d);
      });
      console.log(data);
      res.status(201).send({
        success: true,
        data: data,
        count: count,
      });
    } catch (e) {
      console.log(e);
      res.status(500).send();
    }
  });

  app.get("/data/sell/:pid1/:pid2", async (req: any, res) => {
    try {
      let pid1 = req.params.pid1;
      let pid2 = req.params.pid2;
      const skip = parseInt(req.query.skip) || 0;
      const limit = parseInt(req.query.limit) || 30;
      const [count, doc] = await Promise.all([
        MarketObjectModel.countDocuments({
          settleinfo: { $ne: BigInt(2) },
          owner: [pid1, pid2],
        }),
        MarketObjectModel.find({
          settleinfo: { $ne: BigInt(2) },
          owner: [pid1, pid2],
        })
          .skip(skip)
          .limit(limit),
      ]);

      let data = doc.map((d: mongoose.Document) => {
        return docToJSON(d);
      });
      console.log(data);
      res.status(201).send({
        success: true,
        data: data,
        count: count,
      });
    } catch (e) {
      console.log(e);
      res.status(500).send();
    }
  });

  app.get("/data/markets", async (req: any, res) => {
    try {
      const skip = parseInt(req.query.skip) || 0;
      const limit = parseInt(req.query.limit) || 30;
      const [count, doc] = await Promise.all([
        MarketObjectModel.countDocuments({ settleinfo: { $ne: BigInt(2) } }),
        MarketObjectModel.find({ settleinfo: { $ne: BigInt(2) } })
          .skip(skip)
          .limit(limit),
      ]);

      const data = doc.map((d) => {
        return docToJSON(d);
      });
      console.log(data);
      res.status(201).send({
        success: true,
        data: data,
        count: count,
      });
    } catch (e) {
      console.log(e);
      res.status(500).send();
    }
  });

  app.get("/data/nuggets", async (req: any, res) => {
    try {
      let rawIds = req.query.ids;
      if (rawIds === undefined) {
        rawIds = [];
      } else if (!Array.isArray(rawIds)) {
        rawIds = [rawIds]; // wrap single value into an array
      }
      const ids = rawIds.map((id: string) => BigInt(id));
      const doc = await (ids.length > 0
        ? NuggetObjectModel.find({ id: { $in: ids } })
        : NuggetObjectModel.find());

      const jdoc = doc.map((d) => {
        return docToJSON(d);
      });
      console.log(jdoc);
      res.status(201).send({
        success: true,
        data: jdoc,
      });
    } catch (e) {
      console.log(e);
      res.status(500).send();
    }
  });
}

service.serve();

const EVENT_POSITION_UPDATE = 1;
const EVENT_NUGGET_UPDATE = 2;

async function bootstrap(merkleRoot: string): Promise<TxWitness[]> {
  /*
       const txs = await txStateManager.getTxFromCommit(merkleRoot);
       console.log("tsx in bootstrap:", txs);
       return txs;
       */
  return [];
}

async function batchedCallback(
  arg: TxWitness[],
  preMerkle: string,
  postMerkle: string
) {
  await txStateManager.moveToCommit(postMerkle);
}

async function eventCallback(arg: TxWitness, data: BigUint64Array) {
  if (data.length == 0) {
    return;
  }

  //console.log("eventCallback", arg, data);
  if (data[0] != 0n) {
    console.log("non-zero return, tx failed", data[0]);
    return;
  }
  if (data.length <= 2) {
    console.log("no event data");
    return;
  }

  let event = new Event(data[1], data);
  let doc = new EventModel({
    id: event.id.toString(),
    data: Buffer.from(event.data.buffer),
  });

  try {
    let result = await doc.save();
    if (!result) {
      console.log("failed to save event");
      throw new Error("save event to db failed");
    }
  } catch (e) {
    console.log(e);
    console.log("event ignored");
  }
  let i = 2; // start pos
  while (i < data.length) {
    let eventType = Number(data[i] >> 32n);
    let eventLength = data[i] & ((1n << 32n) - 1n);
    let eventData = data.slice(i + 1, i + 1 + Number(eventLength));
    console.log("event", eventType, eventLength, eventData);
    switch (eventType) {
      case EVENT_POSITION_UPDATE:
        {
          console.log("position event");
        }
        break;
      case EVENT_NUGGET_UPDATE:
        {
          console.log("indexed object event:");
          let obj = IndexedObject.fromEvent(eventData);
          let doc = await obj.storeRelatedObject();
          console.log("indexed object", doc);
        }
        break;
      default:
        console.log("unknown event");
        break;
    }
    i += 1 + Number(eventLength);
  }
}
