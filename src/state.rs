use crate::config::ADMIN_PUBKEY;
use crate::player::{Owner, GamePlayer};
use crate::settlement::SettlementInfo;
use crate::Player;
use crate::nugget::Leaderboard;
use crate::nugget::LeaderboardInfo;
use serde::Serialize;
use std::cell::RefCell;
use zkwasm_rest_abi::MERKLE_MAP;
use zkwasm_rest_abi::StorageData;
use zkwasm_rust_sdk::require;
use zkwasm_rest_abi::enforce;
use crate::command::Command;
use crate::command::Activity;
use crate::command::Deposit;
use crate::command::Withdraw;
use crate::command::CommandHandler;
use crate::error::*;
use zkwasm_rest_convention::clear_events;
use zkwasm_rest_convention::WithBalance;


#[derive(Serialize)]
pub struct GlobalState {
    pub total: u64,
    pub counter: u64,
    pub txsize: u64,
    pub treasure: u64,
    pub cash: u64,
    pub leaderboard: Leaderboard,
}

#[derive(Serialize)]
pub struct QueryState {
    total: u64,
    counter: u64,
    treasure: u64,
    cash: u64,
    leaderboard: Leaderboard,
}

const TICK: u64 = 0;
const INSTALL_PLAYER: u64 = 1;
const WITHDRAW: u64 = 2;
const DEPOSIT: u64 = 3;

const EXPLORE_NUGGET: u64 = 4;
const SELL_NUGGET: u64 = 5;
const BID_NUGGET: u64 = 6;
const CREATE_NUGGET: u64 = 7;
const RECYCLE_NUGGET: u64 = 8;
const LIST_NUGGET: u64 = 9;
const CLAIM_REWARD: u64 = 10;



impl GlobalState {
    pub fn new() -> Self {
        GlobalState {
            total: 0,
            counter: 0,
            txsize: 0,
            treasure: 0,
            cash: 0,
            leaderboard: Leaderboard::default(),
        }
    }

    pub fn snapshot() -> String {
        let total = GLOBAL_STATE.0.borrow().total;
        let counter = GLOBAL_STATE.0.borrow().counter;
        let treasure = GLOBAL_STATE.0.borrow().treasure;
        let cash = GLOBAL_STATE.0.borrow().cash;
        let leaderboard = GLOBAL_STATE.0.borrow().leaderboard.clone();
        serde_json::to_string(&QueryState { counter, total, treasure, cash, leaderboard}).unwrap()
    }

    pub fn get_state(pid: Vec<u64>) -> String {
        let player = GamePlayer::get(&pid.try_into().unwrap());
        serde_json::to_string(&player).unwrap()
    }

    pub fn preempt() -> bool {
        let mut state = GLOBAL_STATE.0.borrow_mut();
        let counter = state.counter;
        let txsize = state.txsize;
        let withdraw_size = SettlementInfo::settlement_size();
        if counter % 1000 == 0 || txsize >= 200 || withdraw_size > 40 {
            state.txsize = 0;
            return true;
        } else {
            return false;
        }
    }

    pub fn flush_settlement() -> Vec<u8> {
        SettlementInfo::flush_settlement()
    }

    pub fn rand_seed() -> u64 {
        0
    }

    pub fn store_into_kvpair(&self) {
        let mut v = vec![];
        v.push(self.counter);
        v.push(self.total);
        v.push(self.treasure);
        v.push(self.cash);
        v.push(self.leaderboard.nuggets.len() as u64);
        for nugget in self.leaderboard.nuggets.iter() {
            nugget.to_data(&mut v);
        }
        let kvpair = unsafe { &mut MERKLE_MAP };
        kvpair.set(&[0, 0, 0, 0], v.as_slice());
    }

    pub fn fetch(&mut self) {
        let kvpair = unsafe { &mut MERKLE_MAP };
        let mut data = kvpair.get(&[0, 0, 0, 0]);
        if !data.is_empty() {
            let mut u64data = data.iter_mut();
            let counter = *u64data.next().unwrap();
            let total = *u64data.next().unwrap();
            let treasure = *u64data.next().unwrap();
            let cash = *u64data.next().unwrap();
            self.counter = counter;
            self.total = total;
            self.treasure = treasure;
            self.cash = cash;
            if let Some(l) = u64data.next() {
                for _ in 0..(*l) {
                    self.leaderboard.nuggets.push(LeaderboardInfo::from_data(&mut u64data));
                }
            } else {
                ()
            }
        }
    }

    pub fn store() {
        GLOBAL_STATE.0.borrow_mut().store_into_kvpair();
    }

    pub fn initialize() {
        GLOBAL_STATE.0.borrow_mut().fetch();
    }

    pub fn get_counter() -> u64 {
        GLOBAL_STATE.0.borrow().counter
    }
}

pub struct SafeState(pub RefCell<GlobalState>);
unsafe impl Sync for SafeState {}

lazy_static::lazy_static! {
    pub static ref GLOBAL_STATE: SafeState = SafeState(RefCell::new(GlobalState::new()));
}

pub struct Transaction {
    command: Command,
    nonce: u64,
}

impl Transaction {
    pub fn decode_error(e: u32) -> &'static str {
        crate::command::decode_error(e)
    }

    pub fn decode(params: &[u64]) -> Self {
        let command = params[0] & 0xff;
        let nonce = params[0] >> 16;
        let command = if command == WITHDRAW {
            Command::Withdraw (Withdraw {
                data: [params[2], params[3], params[4]]
            })
        } else if command == DEPOSIT {
            enforce(params[3] == 0, "check deposit index"); // only token index 0 is supported
            Command::Deposit (Deposit {
                data: [params[1], params[2], params[4]]
            })
        } else if command == INSTALL_PLAYER {
            Command::InstallPlayer
        } else if command == EXPLORE_NUGGET {
            Command::Activity (Activity::Explore(params[1]))
        } else if command == SELL_NUGGET {
            Command::Activity (Activity::Sell(params[1]))
        } else if command == RECYCLE_NUGGET {
            Command::Activity (Activity::Recycle(params[1]))
        } else if command == LIST_NUGGET {
            Command::Activity (Activity::List(params[1], params[2]))
        } else if command == BID_NUGGET {
            Command::Activity (Activity::Bid(params[1], params[2]))
        } else if command == CREATE_NUGGET {
            Command::Activity (Activity::Create)
        } else if command == CLAIM_REWARD {
            Command::Activity (Activity::Claim(params[1]))
        } else {
            unsafe {zkwasm_rust_sdk::require(command == TICK)};
            Command::Tick
        };
        Transaction {
            command,
            nonce,
        }
    }

    pub fn create_player(&self, pkey: &[u64; 4]) -> Result<(), u32> {
        let player = GamePlayer::get(pkey);
        match player {
            Some(_) => Err(ERROR_PLAYER_ALREADY_EXIST),
            None => {
                let mut player = Player::new(pkey);
                player.data.balance = 0;
                player.store();
                Ok(())
            }
        }
    }

    pub fn tick(&self) {
        GLOBAL_STATE.0.borrow_mut().counter += 1;
        let c = GLOBAL_STATE.0.borrow().counter;
        let nuggets = GLOBAL_STATE.0.borrow().leaderboard.nuggets.clone();
        for n in nuggets {
            if (c - n.start) > 12 * 60 * 24 * 7 {
                let mut player = GamePlayer::get_from_pid(&n.owner).unwrap();
                player.data.inc_balance(400000);
                player.store();
            }
        }
        GLOBAL_STATE.0.borrow_mut().leaderboard
            .nuggets.retain(|nugget|
                (c - nugget.start) < 12 * 60 * 24 * 7
            );
    }

    pub fn inc_tx_number(&self) {
        GLOBAL_STATE.0.borrow_mut().txsize += 1;
    }

    pub fn process(&self, pkey: &[u64; 4], rand: &[u64; 4]) -> Vec<u64> {
        let pid = GamePlayer::pkey_to_pid(&pkey);
        let counter = GLOBAL_STATE.0.borrow().counter;
        let e = match &self.command {
            Command::Tick => {
                unsafe { require(*pkey == *ADMIN_PUBKEY) };
                self.tick();
                0
            },
            Command::InstallPlayer => self.create_player(pkey)
                .map_or_else(|e| e, |_| 0),
            Command::Withdraw(cmd) => cmd.handle(&pid, self.nonce, rand, counter)
                .map_or_else(|e| e, |_| 0),
            Command::Activity(cmd) => cmd.handle(&pid, self.nonce, rand, counter)
                .map_or_else(|e| e, |_| 0),
            Command::Deposit(cmd) => {
                unsafe { require(*pkey == *ADMIN_PUBKEY) };
                cmd.handle(&pid, self.nonce, rand, counter)
                    .map_or_else(|e| e, |_| 0)
            },
        };
        if e == 0 {
            match self.command {
                Command::Tick => (),
                _ => {
                    self.inc_tx_number();
                }
            }
        }
        let txsize = GLOBAL_STATE.0.borrow().txsize;
        clear_events(vec![e as u64, txsize])
    }
}
