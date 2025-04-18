use crate::nugget::NuggetInfo;
use crate::Player;
use crate::StorageData;
use core::slice::IterMut;
use serde::Serialize;
use zkwasm_rest_convention::IndexedObject;
use zkwasm_rest_convention::Position;
use zkwasm_rest_convention::Wrapped;
use crate::error::*;

#[derive(Clone, Serialize, Debug)]
pub struct PlayerData {
    pub balance: u64,
    pub inventory_size: u64,
    pub inventory: Vec<u64>,
}

impl Default for PlayerData {
    fn default() -> Self {
        Self {
            balance: 0,
            inventory_size: 4,
            inventory: vec![],
        }
    }
}

impl StorageData for PlayerData {
    fn from_data(u64data: &mut IterMut<u64>) -> Self {
        let balance = *u64data.next().unwrap();
        let inventory_size = *u64data.next().unwrap();
        let length = *u64data.next().unwrap();
        let mut inventory = Vec::with_capacity(length as usize);
        for _ in 0..length {
            inventory.push(*u64data.next().unwrap());
        }
        PlayerData {
            balance,
            inventory_size,
            inventory,
        }
    }
    fn to_data(&self, data: &mut Vec<u64>) {
        data.push(self.balance);
        data.push(self.inventory_size);
        data.push(self.inventory.len() as u64);
        for i in 0..self.inventory.len() {
            data.push(self.inventory[i])
        }
    }
}

pub type GamePlayer = Player<PlayerData>;

pub trait Owner: Sized {
    fn new(pkey: &[u64; 4]) -> Self;
    fn get(pkey: &[u64; 4]) -> Option<Self>;
}

impl Owner for GamePlayer {
    fn new(pkey: &[u64; 4]) -> Self {
        Self::new_from_pid(Self::pkey_to_pid(pkey))
    }

    fn get(pkey: &[u64; 4]) -> Option<Self> {
        Self::get_from_pid(&Self::pkey_to_pid(pkey))
    }
}

impl PlayerData {
    pub fn cost_balance(&mut self, amount: u64) -> Result<(), u32> {
        if self.balance < amount {
            Err(PLAYER_NOT_ENOUGH_BALANCE)
        } else {
            self.balance -= amount;
            Ok(())
        }
    }
    pub fn inc_balance(&mut self, amount: u64) {
        self.balance += amount;
    }
}
