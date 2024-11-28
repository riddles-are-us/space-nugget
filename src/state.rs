use crate::config::{get_action_duration, get_action_reward, ADMIN_PUBKEY};
use crate::player::{Owner, PuppyPlayer};
use crate::settlement::SettlementInfo;
use crate::Player;
use core::slice::IterMut;
use serde::Serialize;
use std::cell::RefCell;
use zkwasm_rest_abi::StorageData;
use zkwasm_rest_abi::{WithdrawInfo, MERKLE_MAP};
use zkwasm_rust_sdk::require;

#[derive(Clone, Serialize, Default, Copy)]
pub struct MemeInfo {
    pub rank: u64,
}

impl StorageData for MemeInfo {
    fn from_data(u64data: &mut IterMut<u64>) -> Self {
        MemeInfo {
            rank: *u64data.next().unwrap(),
        }
    }
    fn to_data(&self, data: &mut Vec<u64>) {
        data.push(self.rank);
    }
}

#[derive(Serialize)]
pub struct GlobalState {
    pub meme_list: Vec<MemeInfo>,
    pub counter: u64,
    pub txsize: u64,
}

#[derive(Serialize)]
pub struct QueryState {
    counter: u64,
    meme_list: Vec<MemeInfo>,
}

impl GlobalState {
    pub fn new() -> Self {
        GlobalState {
            meme_list: [MemeInfo::default(); 36].to_vec(),
            counter: 0,
            txsize: 0,
        }
    }

    pub fn update_meme_rank(index: u64) {
        let mut state = GLOBAL_STATE.0.borrow_mut();
        state.meme_list[index as usize].rank += 1;
    }

    pub fn snapshot() -> String {
        let meme_list = GLOBAL_STATE.0.borrow().meme_list.clone();
        let counter = GLOBAL_STATE.0.borrow().counter;
        serde_json::to_string(&QueryState { counter, meme_list }).unwrap()
    }

    pub fn get_state(pid: Vec<u64>) -> String {
        let player = PuppyPlayer::get(&pid.try_into().unwrap()).unwrap();
        serde_json::to_string(&player).unwrap()
    }

    pub fn preempt() -> bool {
        let counter = GLOBAL_STATE.0.borrow().counter;
        let txsize = GLOBAL_STATE.0.borrow().txsize;
        if counter % 600 == 0 || txsize >= 300 {
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
            let mut meme_list = vec![];
            while u64data.len() != 0 {
                meme_list.push(MemeInfo::from_data(&mut u64data))
            }
            self.counter = counter;
            self.meme_list = meme_list;
        }
    }

    pub fn store() {
        GLOBAL_STATE.0.borrow_mut().store_into_kvpair();
    }

    pub fn initialize() {
        GLOBAL_STATE.0.borrow_mut().fetch();
    }
}

pub struct SafeState(pub RefCell<GlobalState>);
unsafe impl Sync for SafeState {}

lazy_static::lazy_static! {
    pub static ref GLOBAL_STATE: SafeState = SafeState(RefCell::new(GlobalState::new()));
}

const SWAY: u64 = 0;
const CREATE_PLAYER: u64 = 1;
const SHAKE_FEET: u64 = 2;
const JUMP: u64 = 3;
const SHAKE_HEADS: u64 = 4;
const POST_COMMENTS: u64 = 5;
const LOTTERY: u64 = 6;
const CANCELL_LOTTERY: u64 = 7;
const WITHDRAW: u64 = 8;
const DEPOSIT: u64 = 9;

const ERROR_PLAYER_ALREADY_EXIST: u32 = 1;
const ERROR_PLAYER_NOT_EXIST: u32 = 2;
const ERROR_NOT_SELECTED_PLAYER: u32 = 3;
const SELECTED_PLAYER_NOT_EXIST: u32 = 4;
const PLAYER_ACTION_NOT_FINISHED: u32 = 5;
const PLAYER_LOTTERY_EXPIRED: u32 = 6;
const PLAYER_LOTTERY_PROGRESS_NOT_FULL: u32 = 7;
const PLAYER_NOT_ENOUGH_TICKET: u32 = 8;

pub struct Transaction {
    command: u64,
    nonce: u64,
    data: Vec<u64>,
}

impl Transaction {
    pub fn decode_error(e: u32) -> &'static str {
        match e {
            ERROR_PLAYER_NOT_EXIST => "PlayerNotExist",
            ERROR_PLAYER_ALREADY_EXIST => "PlayerAlreadyExist",
            ERROR_NOT_SELECTED_PLAYER => "PlayerNotSelected",
            SELECTED_PLAYER_NOT_EXIST => "SelectedPlayerNotExist",
            PLAYER_ACTION_NOT_FINISHED => "PlayerActionNotFinished",
            PLAYER_LOTTERY_EXPIRED => "PlayerLotteryExpired",
            PLAYER_LOTTERY_PROGRESS_NOT_FULL => "PlayerLotteryProgressNotFull",
            PLAYER_NOT_ENOUGH_TICKET => "PlayerNotEnoughTicket",
            _ => "Unknown",
        }
    }

    pub fn decode(params: [u64; 4]) -> Self {
        let command = params[0] & 0xff;
        let nonce = params[0] >> 16;
        let data = if command == WITHDRAW {
            vec![params[1], params[2], params[3]] // address of withdraw(Note:amount in params[1])
        } else {
            vec![params[1]] // meme coin id
        };

        Transaction {
            command,
            nonce,
            data,
        }
    }

    pub fn create_player(&self, pkey: &[u64; 4]) -> u32 {
        let player = PuppyPlayer::get(pkey);
        match player {
            Some(_) => ERROR_PLAYER_ALREADY_EXIST,
            None => {
                let player = Player::new(pkey);
                player.store();
                0
            }
        }
    }

    pub fn action(&self, pkey: &[u64; 4], action: u64, _rand: &[u64; 4]) -> u32 {
        let mut player = PuppyPlayer::get(pkey);
        let counter = {
            let state = GLOBAL_STATE.0.borrow();
            state.counter
        };

        match player.as_mut() {
            None => ERROR_PLAYER_NOT_EXIST,
            Some(player) => {
                // Check for Lottery action
                if action == LOTTERY {
                    // This is the selected player; allow them to open the blind box
                    if player.data.progress == 1000 {
                        if player.data.last_lottery_timestamp + 10 > counter {
                            // Update player's state to reflect that the lottery is complete
                            player.data.balance += 10; // change 10 to random reward
                            player.data.action = SWAY;
                            player.data.progress = 0;
                            player.data.last_lottery_timestamp = 0;
                            player.data.last_action_timestamp = 0;
                            player.check_and_inc_nonce(self.nonce);
                            player.store();
                            0
                        } else {
                            player.data.action = SWAY;
                            player.data.progress = 0;
                            player.data.last_lottery_timestamp = 0;
                            player.store();
                            0 // return 0 here instead of return error
                              // PLAYER_LOTTERY_EXPIRED
                        }
                    } else {
                        PLAYER_LOTTERY_PROGRESS_NOT_FULL
                    }
                } else if action == CANCELL_LOTTERY {
                    player.data.action = SWAY;
                    player.data.progress = 0;
                    player.data.last_lottery_timestamp = 0;
                    player.store();
                    0
                }  else if action == WITHDRAW {
                    let mut player = PuppyPlayer::get(pkey);
                    match player.as_mut() {
                        None => ERROR_PLAYER_NOT_EXIST,
                        Some(player) => {
                            player.check_and_inc_nonce(self.nonce);
                            let balance = player.data.balance;
                            let amount = (self.data[0] & 0xffffffff) as u32;
                            unsafe { require(balance >= amount) };
                            player.data.balance -= amount;
                            let withdrawinfo =
                                WithdrawInfo::new(&[self.data[0], self.data[1], self.data[2]], 0);
                            SettlementInfo::append_settlement(withdrawinfo);
                            player.store();
                            0
                        }
                    }
                } else {
                    let action_duration = get_action_duration();
                    if player.data.ticket < 1 {
                        PLAYER_NOT_ENOUGH_TICKET
                    } else if player.data.last_action_timestamp != 0
                        && counter < player.data.last_action_timestamp + action_duration
                    {
                        PLAYER_ACTION_NOT_FINISHED
                    } else {
                        player.data.ticket -= 1;
                        let action_reward = get_action_reward();

                        player.data.action = action;
                        player.data.last_action_timestamp = counter;
                        player.data.progress += action_reward;
                        if player.data.progress == 1000 {
                            player.data.last_lottery_timestamp = counter;
                        } else if player.data.progress > 1000 {
                            player.data.progress = 1000;
                        }

                        player.check_and_inc_nonce(self.nonce);
                        GlobalState::update_meme_rank(self.data[0]);
                        player.store();
                        0
                    }
                }
            }
        }
    }

    fn deposit(&self, pid: &[u64; 2]) -> Result<(), u32> {
        let mut admin = PuppyPlayer::get_from_pid(pid).unwrap();
        admin.check_and_inc_nonce(self.nonce);
        let mut player = PuppyPlayer::get_from_pid(&[self.data[0], self.data[1]]);
        match player.as_mut() {
            None => Err(ERROR_PLAYER_NOT_EXIST),
            Some(player) => {
                player.data.ticket += 1;
                player.store();
                admin.store();
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

    pub fn process(&self, pkey: &[u64; 4], rand: &[u64; 4]) -> u32 {
        let res = if self.command == SWAY {
            self.tick();
            0
        } else {
            let res = match self.command {
                CREATE_PLAYER => self.create_player(pkey),
                SHAKE_FEET => self.action(pkey, SHAKE_FEET, rand),
                JUMP => self.action(pkey, JUMP, rand),
                SHAKE_HEADS => self.action(pkey, SHAKE_HEADS, rand),
                POST_COMMENTS => self.action(pkey, POST_COMMENTS, rand),
                LOTTERY => self.action(pkey, LOTTERY, rand),
                CANCELL_LOTTERY => self.action(pkey, CANCELL_LOTTERY, rand),
                WITHDRAW => self.action(pkey, WITHDRAW, rand),
                DEPOSIT => {
                    unsafe { require(*pkey == *ADMIN_PUBKEY) };
                    self.deposit(&PuppyPlayer::pkey_to_pid(pkey))
                        .map_or_else(|e| e, |_| 0)
                },

                _ => {
                    unreachable!();
                }
            };
            if res == 0 {
                self.inc_tx_number();
                self.tick();
            }
            res
        };
        /* debug for tx error
        let root = unsafe { &mut MERKLE_MAP.merkle.root };
        zkwasm_rust_sdk::dbg!("tx info {}, {}\n",
            { GLOBAL_STATE.0.borrow().txsize },
            { GLOBAL_STATE.0.borrow().counter}
        );
        zkwasm_rust_sdk::dbg!("post root {:?}\n", root);
        */
        res
    }
}
