use std::{ops::BitXor, slice::IterMut};
use serde::Serialize;
use zkwasm_rest_abi::StorageData;
use zkwasm_rest_convention::IndexedObject;
use crate::error::*;

#[derive(Clone, Serialize, Default, Copy)]
pub struct NuggetInfo {
    pub id: u64,
    pub attributes: [u8; 8],
    pub cycle: u64,
    pub feature: u64,
    pub sysprice: u64,
    pub marketid: u64, // the associated makret id for this object. None if zero
}

impl StorageData for NuggetInfo {
    fn from_data(u64data: &mut IterMut<u64>) -> Self {
        let id = *u64data.next().unwrap();
        let attributes = (*u64data.next().unwrap()).to_le_bytes();
        let cycle = *u64data.next().unwrap();
        let feature = *u64data.next().unwrap();
        let sysprice = *u64data.next().unwrap();
        let marketid= *u64data.next().unwrap();
        NuggetInfo {
            id,
            attributes,
            cycle,
            feature,
            sysprice,
            marketid,
        }
    }
    fn to_data(&self, data: &mut Vec<u64>) {
        data.push(self.id);
        data.push(u64::from_le_bytes(self.attributes));
        data.push(self.cycle);
        data.push(self.feature);
        data.push(self.sysprice);
        data.push(self.marketid);
    }
}

const EXPLORE_WEIGHT:[u8; 64] = [
    2,2,2,1,1,1,1,0,
    3,3,3,3,3,2,1,0,
    4,4,4,4,3,2,1,0,
    5,5,5,4,3,2,1,0,
    6,6,5,4,3,2,1,0,
    7,6,5,4,3,2,1,0,
    8,7,6,5,4,3,2,0,
    9,8,7,6,5,4,3,2,
];

impl NuggetInfo {
    pub fn new(id: u64, rand: u64) -> Self {
       let c = rand.to_le_bytes();
       NuggetInfo {
           id,
           cycle: 0,
           attributes: [c[0].bitxor(c[1]) / 2 + 1, 0, 0, 0, 0, 0, 0, 0],
           feature: rand % 8,
           sysprice: 0,
           marketid: 0,
       }
    }

    pub fn explore(&mut self, rand: u64) -> Result<(), u32> {
        let r = EXPLORE_WEIGHT[(rand % 64) as usize ];
        for c in self.attributes.iter_mut() {
            if *c == 0 {
                *c = r + 1;
                return Ok(())
            }
        }
        Err(ERROR_NUGGET_ATTRIBUTES_ALL_EXPLORED)
    }

    pub fn compute_sysprice(&mut self) {
        let plus_pos = self.feature as usize;
        let mut p: u64 = self.attributes[0] as u64;
        for i in 1..(plus_pos + 1) {
            let c = self.attributes[i];
            if c == 0 {
                p = p + 2;
            } else {
                p = p + ((c as u64 - 1) % 10)
            }
        }
        for i in (plus_pos + 1) .. 8 {
            let c = self.attributes[i];
            if c == 0 {
                p = p * 2;
            } else {
                p = p * ((c as u64 - 1) % 10)
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


