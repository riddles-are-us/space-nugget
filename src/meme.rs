use std::slice::IterMut;
use serde::Serialize;
use zkwasm_rest_abi::{StorageData, MERKLE_MAP};

pub static mut EVENTS: Vec<u64> = vec![];

pub fn clear_events(a: Vec<u64>) -> Vec<u64> {
    let mut c = a;
    unsafe {
        c.append(&mut EVENTS);
    }
    return c;
}

pub fn insert_event(typ: u64, data: &mut Vec<u64>) {
    unsafe {
        EVENTS.push((typ << 32) + data.len() as u64);
        EVENTS.append(data);
    }
}

#[derive(Clone, Serialize, Default, Copy)]
pub struct MemeInfo {
    pub id: u64,
    pub rank: u64,
    pub stake: u64,
    pub owner: [u64; 2],
}

impl StorageData for MemeInfo {
    fn from_data(u64data: &mut IterMut<u64>) -> Self {
        MemeInfo {
            id: *u64data.next().unwrap(),
            rank: *u64data.next().unwrap(),
            stake: *u64data.next().unwrap(),
            owner: [*u64data.next().unwrap(),*u64data.next().unwrap()],
        }
    }
    fn to_data(&self, data: &mut Vec<u64>) {
        data.push(self.id);
        data.push(self.rank);
        data.push(self.stake);
        data.push(self.owner[0]);
        data.push(self.owner[1]);
    }
}

#[derive(Clone, Serialize, Default, Copy, Debug)]
pub struct StakeInfo {
    pub stake: u64,
}

impl StorageData for StakeInfo {
    fn from_data(u64data: &mut IterMut<u64>) -> Self {
        StakeInfo {
            stake: *u64data.next().unwrap(),
        }
    }
    fn to_data(&self, data: &mut Vec<u64>) {
        data.push(self.stake);
    }
}

pub struct Wrapped<P: StorageData> {
    key: [u64; 4],
    pub data: P,
}

impl<P: StorageData> Wrapped<P> {
    pub fn store(&self) {
        let mut data = Vec::new();
        self.data.to_data(&mut data);
        let kvpair = unsafe { &mut MERKLE_MAP };
        kvpair.set(&self.key, data.as_slice());
    }
}

pub trait IndexedObject<P: StorageData> {
    const PREFIX: u64;
    const POSTFIX: u64;
    const EVENT_NAME:u64;
    fn new_object(p: P, index: u64) -> Wrapped<P> {
        let key = [Self::PREFIX + (index << 16), Self::POSTFIX, Self::POSTFIX, Self::POSTFIX];
        Wrapped {
            key,
            data: p
        }
    }

    fn get_object(index: u64) -> Option<Wrapped<P>> {
        let kvpair = unsafe { &mut MERKLE_MAP };
        let key = [Self::PREFIX + (index << 16), Self::POSTFIX, Self::POSTFIX, Self::POSTFIX];
        let mut data = kvpair.get(&key);
        if data.is_empty() {
            None
        } else {
            let mut dataslice = data.iter_mut();
            Some(Wrapped {
                key,
                data: P::from_data(&mut dataslice),
            })
        }
    }
    fn emit_event(index: u64, p: &P) {
        let mut data = vec![index];
        p.to_data(&mut data);
        insert_event(Self::EVENT_NAME, &mut data);
    }
}

pub trait Position<P: StorageData> {
    const PREFIX: u64;
    const POSTFIX: u64;
    const EVENT_NAME: u64;
    fn new_position(pid: &[u64; 2], p: P, index: u64) -> Wrapped<P> {
        let key = [Self::PREFIX + (pid[0]<<16) + (index << 32), (pid[0] >> 48) + (pid[1] << 16), (pid[1] >> 48) + (Self::POSTFIX << 16), 0];
        Wrapped {
            key,
            data: p
        }
    }

    fn get_position(pid: &[u64; 2], index: u64) -> Option<Wrapped<P>> {
        let kvpair = unsafe { &mut MERKLE_MAP };
        let key = [Self::PREFIX + (pid[0]<<16) + (index << 32), (pid[0] >> 48) + (pid[1] << 16), (pid[1] >> 48) + (Self::POSTFIX << 16), 0];
        let mut data = kvpair.get(&key);
        if data.is_empty() {
            None
        } else {
            let mut dataslice = data.iter_mut();
            Some(Wrapped {
                key,
                data: P::from_data(&mut dataslice),
            })
        }
    }

    fn get_or_new_position(pid: &[u64; 2], index: u64, default: P) -> Wrapped<P> {
        let kvpair = unsafe { &mut MERKLE_MAP };
        let key = [Self::PREFIX + (pid[0]<<16) + (index << 32), (pid[0] >> 48) + (pid[1] << 16), (pid[1] >> 48) + (Self::POSTFIX << 16), 0];
        let mut data = kvpair.get(&key);
        if data.is_empty() {
            Wrapped {
                key,
                data: default
            }
        } else {
            let mut dataslice = data.iter_mut();
            Wrapped {
                key,
                data: P::from_data(&mut dataslice),
            }
        }
    }
    fn emit_event(pid: &[u64; 2], index: u64, p: &P) {
        let mut data = vec![pid[0], pid[1], index];
        p.to_data(&mut data);
        insert_event(Self::EVENT_NAME, &mut data);
    }
}

impl IndexedObject<MemeInfo> for MemeInfo {
    const PREFIX: u64 = 0x1ee1;
    const POSTFIX: u64 = 0xfee1;
    const EVENT_NAME: u64 = 0x02;
}

impl Position<StakeInfo> for StakeInfo {
    const PREFIX: u64 = 0x1ff1;
    const POSTFIX: u64 = 0xf1f1;
    const EVENT_NAME: u64 = 0x01;
}
