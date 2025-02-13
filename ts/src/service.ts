import { Service } from "zkwasm-ts-server";
import {TxWitness} from "zkwasm-ts-server/src/prover";
import {Event, EventModel} from "./event.js";
import { Position, IndexedObjectModel, IndexedObject, PositionModel, parseNuggetInfo} from "./info.js";
import { Player} from "./api.js";
import { get_server_admin_key } from "zkwasm-ts-server/src/config.js";
import { Express } from "express";
//import {clearTxFromCommit, CommitModel, getTxFromCommit, insertTxIntoCommit} from "./commits.js";
import {merkleRootToBeHexString} from "zkwasm-ts-server/src/lib.js";

const uncommittedTxs: TxWitness[] = [];


const service = new Service(eventCallback, batchedCallback, extra, bootstrap);
await service.initialize();


let currentUncommitMerkleRoot: string = merkleRootToBeHexString(service.merkleRoot);

function extra (app: Express) {
  app.get('/data/nugget/:nid', async(req:any, res) => {
    try {
      let nid = req.params.nid;
      const doc = await IndexedObjectModel.find(
        //{index: Number(nid)},
        {index: nid},
      );
      let data = doc.map((d) => {
        return parseNuggetInfo(IndexedObject.fromMongooseDoc(d))
      })
      res.status(201).send({
        success: true,
        data: data,
      });
    } catch (e) {
      console.log(e);
      res.status(500).send()
    }
  });

	app.get('/data/nuggets', async(req:any, res) => {
		const doc = await IndexedObjectModel.find();
    try {
      const jdoc = doc.map((d) => {
        const jdoc = IndexedObject.fromMongooseDoc(d);
        return parseNuggetInfo(jdoc);
      });
      res.status(201).send({
        success: true,
        data: jdoc,
      });
    } catch (e) {
      console.log(e);
      res.status(500).send()
    }
	});
}


service.serve();

const EVENT_POSITION_UPDATE = 1;
const EVENT_NUGGET_UPDATE = 2;

let preemptcounter = 0;

async function bootstrap(merkleRoot: string): Promise<TxWitness[]> {
	/*
	const txs = await getTxFromCommit(merkleRoot);
	console.log("tsx in bootstrap:", txs);
	return txs;
	*/
	return [];
}

async function batchedCallback(arg: TxWitness[], preMerkle: string, postMerkle: string) {
	/*
	currentUncommitMerkleRoot = postMerkle;
	await clearTxFromCommit(currentUncommitMerkleRoot);
	preemptcounter = 0;
	*/
	return;
}

async function eventCallback(arg: TxWitness, data: BigUint64Array) {
	//insertTxIntoCommit(currentUncommitMerkleRoot, arg, preemptcounter);
	//preemptcounter ++;
	if(data.length == 0) {
		return;
	}

	//console.log("eventCallback", arg, data);
	if(data[0] != 0n) {
		console.log("non-zero return, tx failed", data[0]);
		return;
	}
	if(data.length <= 2) {
		console.log("no event data");
		return;
	}

	let event = new Event(data[1], data);
	let doc = new EventModel({
		id: event.id.toString(),
		data: Buffer.from(event.data.buffer)
	});

	try {
		let result = await doc.save();
		if (!result) {
			console.log("failed to save event");
			throw new Error("save event to db failed");
		}
	} catch(e) {
		console.log(e);
		console.log("event ignored");
	}
	let i = 2; // start pos
	while(i < data.length) {
		let eventType = Number(data[i]>>32n);
		let eventLength = data[i]&((1n<<32n)-1n);
		let eventData = data.slice(i+1, i+1+Number(eventLength));
		console.log("event", eventType, eventLength, eventData);
		switch(eventType) {
			case EVENT_POSITION_UPDATE:
				{
					console.log("position event");
					let position = Position.fromEvent(eventData);
					let doc = await PositionModel.findOneAndUpdate(
							{pid_1: position.pid_1, pid_2: position.pid_2, object_index: position.object_index},
							position.toObject(),
							{upsert: true}
					);
					console.log("save position", position.pid_1, position.pid_2, position.object_index, doc);
				}
				break;
			case EVENT_NUGGET_UPDATE:
				{
					console.log("indexed object event:");
					let obj = IndexedObject.fromEvent(eventData);
					let doc = await IndexedObjectModel.findOneAndUpdate({index: obj.index}, obj.toObject(), {upsert: true});
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



