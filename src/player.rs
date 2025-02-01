use crate::Player;
use crate::StorageData;
use core::slice::IterMut;
use serde::Serialize;
use crate::error::*;
use crate::meme::IndexedObject;
use crate::meme::Position;
use crate::meme::MemeInfo;
use crate::meme::StakeInfo;
use crate::meme::Wrapped;

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

impl PlayerData {
    pub fn check_and_update_action_timestamp(&mut self, counter: u64, duration: u64) -> Result<(), u32> {
        if self.last_action_timestamp != 0
            && counter < self.last_action_timestamp + duration
            {
                Err(PLAYER_ACTION_NOT_FINISHED)
            } else {
                self.last_action_timestamp = counter;
                Ok(())
            }
    }
    pub fn increase_progress(&mut self, counter:u64, progress: u32) {
        self.progress += progress;
        if self.progress >= 1000 {
            self.progress = 1000;
        }
        if self.progress == 1000 {
            self.last_lottery_timestamp = counter;
        }

    }
    pub fn cost_ticket(&mut self, amount: u32) -> Result<(), u32> {
        if self.ticket < amount {
            Err(PLAYER_NOT_ENOUGH_TICKET)
        } else {
            self.ticket -= amount;
            Ok(())
        }
    }
}

pub trait PositionHolder: Sized {
    fn stake(&mut self, meme_index: u64, amount: u32) -> Result<(Wrapped<StakeInfo>, Wrapped<MemeInfo>), u32>;
}



impl PositionHolder for Player<PlayerData> {
    fn stake(&mut self, meme_index: u64, amount: u32) -> Result<(Wrapped<StakeInfo>, Wrapped<MemeInfo>), u32> {
        self.data.cost_ticket(amount)?;
        let mut pos = StakeInfo::get_or_new_position(&self.player_id, meme_index, StakeInfo { stake: 0 });
        let mut meme = MemeInfo::get_object(meme_index);
        match meme {
            Some (mut m) => {
                pos.data.stake += amount as u64;
                if m.data.stake < pos.data.stake {
                    m.data.stake = pos.data.stake;
                    m.data.owner = self.player_id.clone();
                }
                Ok((pos, m))
            }
            None => Err(INVALID_MEME_INDEX)
        }
    }
}
