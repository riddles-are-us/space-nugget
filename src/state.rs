use crate::config::ADMIN_PUBKEY;
use crate::meme::MemeInfo;
use crate::player::{Owner, PuppyPlayer};
use crate::settlement::SettlementInfo;
use crate::Player;
use serde::Serialize;
use std::cell::RefCell;
use zkwasm_rest_abi::StorageData;
use zkwasm_rest_abi::MERKLE_MAP;
use zkwasm_rust_sdk::require;
use crate::command::Command;
use crate::command::Activity;
use crate::command::Deposit;
use crate::command::Withdraw;
use crate::command::WithdrawLottery;
use crate::command::CommandHandler;
use crate::error::*;



#[derive(Serialize)]
pub struct GlobalState {
    pub meme_list: Vec<MemeInfo>,
    pub counter: u64,
    pub txsize: u64,
    pub airdrop: u64,
}

#[derive(Serialize)]
pub struct QueryState {
    counter: u64,
    meme_list: Vec<MemeInfo>,
    airdrop: u64,
}

const TICK: u64 = 0;
const INSTALL_PLAYER: u64 = 1;

const VOTE: u64 = 2;
const STAKE: u64 = 3;
const BET: u64 = 4;
const COMMENT: u64 = 5;
const LOTTERY: u64 = 6;

const WITHDRAW: u64 = 8;
const DEPOSIT: u64 = 9;
const WITHDRAW_LOTTERY: u64 = 10;



impl GlobalState {
    pub fn new() -> Self {
        GlobalState {
            meme_list: [MemeInfo::default(); 12].to_vec(),
            counter: 0,
            txsize: 0,
            airdrop: 10000000
        }
    }

    pub fn update_meme_rank(index: usize) {
        let mut state = GLOBAL_STATE.0.borrow_mut();
        state.meme_list[index].rank += 1;
    }

    pub fn update_meme(index: usize, meme: MemeInfo) {
        let mut state = GLOBAL_STATE.0.borrow_mut();
        state.meme_list[index] = meme;
    }

    pub fn snapshot() -> String {
        let meme_list = GLOBAL_STATE.0.borrow().meme_list.clone();
        let counter = GLOBAL_STATE.0.borrow().counter;
        let airdrop = GLOBAL_STATE.0.borrow().airdrop;
        serde_json::to_string(&QueryState { counter, meme_list, airdrop }).unwrap()
    }

    pub fn get_state(pid: Vec<u64>) -> String {
        let player = PuppyPlayer::get(&pid.try_into().unwrap());
        serde_json::to_string(&player).unwrap()
    }

    pub fn preempt() -> bool {
        let counter = GLOBAL_STATE.0.borrow().counter;
        let txsize = GLOBAL_STATE.0.borrow().txsize;
        let withdraw_size = SettlementInfo::settlement_size();
        if counter % 600 == 0 || txsize >= 300 || withdraw_size > 40 {
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
        let n = self.meme_list.len();
        let mut v = Vec::with_capacity(n * 2 + 1);
        v.push(self.counter);
        v.push(self.airdrop);
        for e in self.meme_list.iter() {
            e.to_data(&mut v);
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
            let airdrop = *u64data.next().unwrap();
            let mut meme_list = vec![];
            while u64data.len() != 0 {
                meme_list.push(MemeInfo::from_data(&mut u64data))
            }
            self.counter = counter;
            self.airdrop = airdrop;
            self.meme_list = meme_list;
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
            Command::Deposit (Deposit {
                data: [params[2], params[3], params[4]]
            })
        } else if command == WITHDRAW_LOTTERY {
            Command::WithdrawLottery (WithdrawLottery {
                data: [params[2], params[3], params[4]]
            })
        } else if command == INSTALL_PLAYER {
            Command::InstallPlayer
        } else  if command == LOTTERY {
            Command::Activity (Activity::Lottery)
        } else if command == VOTE {
            Command::Activity (Activity::Vote(params[1] as usize))
        } else if command == STAKE {
            Command::Activity (Activity::Stake(params[1] as usize, params[2]))
        } else if command == BET {
            Command::Activity (Activity::Bet(params[1] as usize))
        } else if command == COMMENT {
            let chars = params[1..].iter().flat_map(|x| x.to_le_bytes()).collect::<Vec<u8>>();
            Command::Activity (Activity::Comment(chars))
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
        let player = PuppyPlayer::get(pkey);
        match player {
            Some(_) => Err(ERROR_PLAYER_ALREADY_EXIST),
            None => {
                let mut player = Player::new(pkey);
                if GLOBAL_STATE.0.borrow().airdrop > 50 {
                    player.data.balance = 50;
                    GLOBAL_STATE.0.borrow_mut().airdrop -= 50;
                } else {
                    player.data.balance = 0;
                }
                player.store();
                Ok(())
            }
        }
    }

    pub fn tick(&self) {
        GLOBAL_STATE.0.borrow_mut().counter += 1;
    }

    pub fn inc_tx_number(&self) {
        GLOBAL_STATE.0.borrow_mut().txsize += 1;
    }

    pub fn process(&self, pkey: &[u64; 4], rand: &[u64; 4]) -> Vec<u64> {
        let pid = PuppyPlayer::pkey_to_pid(&pkey);
        let e = match &self.command {
            Command::Tick => {
                unsafe { require(*pkey == *ADMIN_PUBKEY) };
                self.tick();
                0
            },
            Command::InstallPlayer => self.create_player(pkey)
                .map_or_else(|e| e, |_| 0),
            Command::Withdraw(cmd) => cmd.handle(&pid, self.nonce, rand)
                .map_or_else(|e| e, |_| 0),
            Command::WithdrawLottery(cmd) => cmd.handle(&pid, self.nonce, rand)
                .map_or_else(|e| e, |_| 0),
            Command::Activity(cmd) => cmd.handle(&pid, self.nonce, rand)
                .map_or_else(|e| e, |_| 0),
            Command::Deposit(cmd) => {
                unsafe { require(*pkey == *ADMIN_PUBKEY) };
                cmd.handle(&pid, self.nonce, rand)
                    .map_or_else(|e| e, |_| 0)
            },
        };
        match self.command {
            Command::Tick => (),
            _ => {
                self.inc_tx_number();
                self.tick();
            }
        }
        vec![e as u64]
    }
}
