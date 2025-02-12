use std::slice::IterMut;
use serde::Serialize;
use zkwasm_rest_abi::{StorageData, MERKLE_MAP};
use zkwasm_rest_convention::{IndexedObject, Position};

#[derive(Clone, Serialize, Default, Copy)]
pub struct NuggetInfo {
    pub id: u64,
    pub attributes: [u8; 8],
    pub sysprice: u64,
    pub askprice: u64,
    pub bidprice: u64,
}

impl StorageData for NuggetInfo {
    fn from_data(u64data: &mut IterMut<u64>) -> Self {
        let id = *u64data.next().unwrap();
        let sysprice = *u64data.next().unwrap();
        let askprice = *u64data.next().unwrap();
        let bidprice = *u64data.next().unwrap();
        let attributes = (*u64data.next().unwrap()).to_le_bytes();
        NuggetInfo {
            id,
            sysprice,
            askprice,
            bidprice,
            attributes,
        }
    }
    fn to_data(&self, data: &mut Vec<u64>) {
        data.push(self.id);
        data.push(self.sysprice);
        data.push(self.askprice);
        data.push(self.bidprice);
        data.push(u64::from_le_bytes(self.attributes));
    }
}

impl IndexedObject<NuggetInfo> for NuggetInfo {
    const PREFIX: u64 = 0x1ee1;
    const POSTFIX: u64 = 0xfee1;
    const EVENT_NAME: u64 = 0x01;
}
