use crate::Player;
use crate::StorageData;
use core::slice::IterMut;
use serde::Serialize;

#[derive(Clone, Serialize, Debug)]
pub struct PlayerData {
    pub balance: u32,
    pub ticket: u32,
    pub action: u64,
    pub last_lottery_timestamp: u64, // last timestamp when this user allowed to pick a lottery
    pub last_action_timestamp: u64,  // last timestamp when this user allowed to pick a lottery
    pub lottery_info: u32,
    pub progress: u32,
}

impl Default for PlayerData {
    fn default() -> Self {
        Self {
            action: 0,
            last_lottery_timestamp: 0,
            last_action_timestamp: 0, // last timestamp when this user allowed to pick a lottery
            balance: 0,
            ticket: 50,
            lottery_info: 0,
            progress: 0,
        }
    }
}

impl StorageData for PlayerData {
    fn from_data(u64data: &mut IterMut<u64>) -> Self {
        let bt = *u64data.next().unwrap();
        let balance = (bt >> 32) as u32;
        let ticket = (bt & 0xffffffff) as u32;
        let lot = *u64data.next().unwrap();
        let progress = (lot >> 32) as u32;
        let lottery_info = (lot & 0xffffffff) as u32;
        PlayerData {
            progress,
            lottery_info,
            balance,
            ticket,
            action: *u64data.next().unwrap(),
            last_lottery_timestamp: *u64data.next().unwrap(),
            last_action_timestamp: *u64data.next().unwrap(),
        }
    }
    fn to_data(&self, data: &mut Vec<u64>) {
        data.push(((self.balance as u64) << 32) + (self.ticket as u64));
        data.push(((self.progress as u64) << 32) + (self.lottery_info as u64));
        data.push(self.action);
        data.push(self.last_lottery_timestamp);
        data.push(self.last_action_timestamp);
    }
}

pub type PuppyPlayer = Player<PlayerData>;

pub trait Owner: Sized {
    fn new(pkey: &[u64; 4]) -> Self;
    fn get(pkey: &[u64; 4]) -> Option<Self>;
}

impl Owner for PuppyPlayer {
    fn new(pkey: &[u64; 4]) -> Self {
        Self::new_from_pid(Self::pkey_to_pid(pkey))
    }

    fn get(pkey: &[u64; 4]) -> Option<Self> {
        Self::get_from_pid(&Self::pkey_to_pid(pkey))
    }
}
