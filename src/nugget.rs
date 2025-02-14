use std::{ops::BitXor, slice::IterMut};
use serde::Serialize;
use zkwasm_rest_abi::{StorageData, MERKLE_MAP};
use zkwasm_rest_convention::{IndexedObject, Position};

use crate::error::ERROR_NUGGET_ATTRIBUTES_ALL_EXPLORED;

#[derive(Clone, Serialize, Default, Copy)]
pub struct BidInfo {
    pub bidprice: u64,
    pub bidder: [u64; 2],
}

#[derive(Clone, Serialize, Default, Copy)]
pub struct NuggetInfo {
    pub id: u64,
    pub attributes: [u8; 8],
    pub cycle: u64,
    pub sysprice: u64,
    pub askprice: u64,
    pub bid: Option<BidInfo>,
}

impl StorageData for NuggetInfo {
    fn from_data(u64data: &mut IterMut<u64>) -> Self {
        let id = *u64data.next().unwrap();
        let attributes = (*u64data.next().unwrap()).to_le_bytes();
        let cycle = *u64data.next().unwrap();
        let sysprice = *u64data.next().unwrap();
        let askprice = *u64data.next().unwrap();
        let bid = *u64data.next().unwrap();
        let mut bidder = None;
        if bid != 0 {
            bidder =  Some(BidInfo {
                bidprice: bid,
                bidder: [*u64data.next().unwrap(), *u64data.next().unwrap()]
            })
        }
        NuggetInfo {
            id,
            attributes,
            cycle,
            sysprice,
            askprice,
            bid: bidder,
        }
    }
    fn to_data(&self, data: &mut Vec<u64>) {
        data.push(self.id);
        data.push(u64::from_le_bytes(self.attributes));
        data.push(self.cycle);
        data.push(self.sysprice);
        data.push(self.askprice);
        match self.bid {
            None => data.push(0),
            Some(b) => {
                data.push(b.bidprice);
                data.push(b.bidder[0]);
                data.push(b.bidder[1]);
            },
        }
    }
}

impl NuggetInfo {
    pub fn new(id: u64, rand: u64) -> Self {
       let c = rand.to_le_bytes();
       NuggetInfo {
           id,
           cycle: 0,
           attributes: [c[0].bitxor(c[1]), 0, 0, 0, 0, 0, 0, 0],
           sysprice: 0,
           askprice: 0,
           bid: None
       }
    }

    pub fn explore(&mut self, rand: u64) -> Result<(), u32> {
        let mut p: u64 = 1;
        let r = rand.to_le_bytes();
        for c in self.attributes.iter_mut() {
            if *c == 0 {
                *c = r[0].bitxor(r[1]);
                return Ok(())
            }
        }
        Err(ERROR_NUGGET_ATTRIBUTES_ALL_EXPLORED)
    }

    pub fn compute_sysprice(&mut self) {
        let mut p: u64 = 1;
        for c in self.attributes {
            if c == 0 {
                p = p + 2;
            } else {
                p = p + ((c as u64) - 1)
            }
        }
        self.sysprice = p;
    }
}

impl IndexedObject<NuggetInfo> for NuggetInfo {
    const PREFIX: u64 = 0x1ee1;
    const POSTFIX: u64 = 0xfee1;
    const EVENT_NAME: u64 = 0x02;
}
