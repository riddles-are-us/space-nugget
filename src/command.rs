use crate::nugget::{BidInfo, NuggetInfo};
use zkwasm_rest_convention::IndexedObject;
use zkwasm_rust_sdk::require;
use zkwasm_rest_abi::WithdrawInfo;
use crate::settlement::SettlementInfo;
use crate::player::GamePlayer;
use crate::state::GLOBAL_STATE;
use crate::error::*;

#[derive (Clone)]
pub enum Command {
    // standard activities
    Activity(Activity),
    // standard withdraw and deposit
    Withdraw(Withdraw),
    Deposit(Deposit),
    // standard player install and timer
    InstallPlayer,
    Tick,
}


pub trait CommandHandler {
    fn handle(&self, pid: &[u64; 2], nonce: u64, rand: &[u64; 4], counter: u64) -> Result<(), u32>;
}

#[derive (Clone)]
pub struct Withdraw {
    pub data: [u64; 3],
}

impl CommandHandler for Withdraw {
    fn handle(&self, pid: &[u64; 2], nonce: u64, _rand: &[u64; 4], _counter: u64) -> Result<(), u32> {
        let mut player = GamePlayer::get_from_pid(pid);
        match player.as_mut() {
            None => Err(ERROR_PLAYER_NOT_EXIST),
            Some(player) => {
                player.check_and_inc_nonce(nonce);
                let balance = player.data.balance;
                let amount = self.data[0] & 0xffffffff;
                unsafe { require(balance >= amount) };
                player.data.balance -= amount;
                let withdrawinfo =
                    WithdrawInfo::new(&[self.data[0], self.data[1], self.data[2]], 0);
                SettlementInfo::append_settlement(withdrawinfo);
                player.store();
                Ok(())
            }
        }
    }
}

#[derive (Clone)]
pub struct Deposit {
    pub data: [u64; 3],
}

impl CommandHandler for Deposit {
    fn handle(&self, pid: &[u64; 2], nonce: u64, _rand: &[u64; 4], _counter: u64) -> Result<(), u32> {
        let mut admin = GamePlayer::get_from_pid(pid).unwrap();
        admin.check_and_inc_nonce(nonce);
        let mut player = GamePlayer::get_from_pid(&[self.data[0], self.data[1]]);
        match player.as_mut() {
            None => Err(ERROR_PLAYER_NOT_EXIST),
            Some(player) => {
                player.data.balance += self.data[2];
                player.store();
                admin.store();
                Ok(())
            }
        }
    }
}

#[derive (Clone)]
pub enum Activity {
    // activities
    Create,
    Bid(u64, u64),
    Sell(u64),
    Explore(u64),
}

impl CommandHandler for Activity {
    fn handle(&self, pid: &[u64; 2], nonce: u64, rand: &[u64; 4], _counter: u64) -> Result<(), u32> {
        let mut player = GamePlayer::get_from_pid(pid);
        match player.as_mut() {
            None => Err(ERROR_PLAYER_NOT_EXIST),
            Some(player) => {
                player.check_and_inc_nonce(nonce);
                match self {
                    Activity::Create => {
                        if player.data.inventory.len() > player.data.inventory_size as usize {
                            Err(PLAYER_NOT_ENOUGH_INVENTORY)
                        } else {
                            let mut global = GLOBAL_STATE.0.borrow_mut();
                            let mut nugget = NuggetInfo::new_object(NuggetInfo::new(global.total, rand[1]), global.total);
                            nugget.data.compute_sysprice();
                            nugget.store();
                            NuggetInfo::emit_event(global.total, &nugget.data);
                            global.total += 1;
                            player.data.inventory.push(nugget.data.id);
                            player.store();
                            Ok(())
                        }
                    },

                    Activity::Explore(index) => {
                        if player.data.inventory.len() <= (*index) as usize {
                            Err(INVALID_NUGGET_INDEX)
                        } else {
                            let nuggetid = player.data.inventory[*index as usize];
                            let mut nugget = NuggetInfo::get_object(nuggetid).unwrap();
                            player.data.cost_balance(nugget.data.sysprice / 4)?;
                            nugget.data.explore(rand[2])?;
                            nugget.data.compute_sysprice();
                            if let Some(bidder) = nugget.data.bid {
                                let mut last_player= GamePlayer::get_from_pid(&bidder.bidder).unwrap();
                                last_player.data.inc_balance(bidder.bidprice);
                                last_player.store();
                                nugget.data.bid = None;
                            };
                            NuggetInfo::emit_event(nugget.data.id, &nugget.data);
                            nugget.store();
                            player.store();
                            Ok(())
                        }
                    },

                    Activity::Sell(index) => {
                        if player.data.inventory.len() <= (*index) as usize {
                            Err(INVALID_NUGGET_INDEX)
                        } else {
                            let nuggetid = player.data.inventory[*index as usize];
                            let mut nugget = NuggetInfo::get_object(nuggetid).unwrap();
                            match nugget.data.bid {
                                None => {
                                    // sell at system price
                                    player.data.inc_balance(nugget.data.sysprice);
                                    nugget.data.cycle = 1;
                                    player.data.inventory.swap_remove(*index as usize);
                                    nugget.store();
                                    player.store();
                                },
                                Some (bidder) => {
                                    player.data.inc_balance(bidder.bidprice);
                                    let mut last_player= GamePlayer::get_from_pid(&bidder.bidder).unwrap();
                                    last_player.data.inventory.push(nugget.data.id);
                                    player.data.inventory.swap_remove(*index as usize);
                                    nugget.store();
                                    player.store();
                                    last_player.store();
                                }
                            }
                            NuggetInfo::emit_event(nugget.data.id, &nugget.data);
                            Ok(())
                        }
                    },

                    Activity::Bid(nid, price) => {
                        player.data.cost_balance(*price)?;
                        let nugget = NuggetInfo::get_object(*nid);
                        match nugget {
                            Some(mut n) => {
                                match n.data.bid {
                                    Some(bidder) => {
                                        if bidder.bidprice >= *price {
                                            Err(ERROR_BID_PRICE_INSUFFICIENT)
                                        } else {
                                            let mut last_player= GamePlayer::get_from_pid(&bidder.bidder).unwrap();
                                            last_player.data.inc_balance(bidder.bidprice);
                                            n.data.bid = Some(BidInfo {
                                                bidprice: *price,
                                                bidder: pid.clone(),
                                            });
                                            last_player.store();
                                            n.store();
                                            NuggetInfo::emit_event(n.data.id, &n.data);
                                            Ok(())
                                        }
                                    },
                                    None => {
                                        n.data.bid = Some(BidInfo {
                                            bidprice: *price,
                                            bidder: pid.clone(),
                                        });
                                        n.store();
                                        NuggetInfo::emit_event(n.data.id, &n.data);
                                        Ok(())
                                    }
                                }
                            },
                            None => Err(INVALID_NUGGET_INDEX)
                        }
                    }
                }
            }
        }
    }
}

pub fn decode_error(e: u32) -> &'static str {
    match e {
        ERROR_PLAYER_NOT_EXIST => "PlayerNotExist",
        ERROR_PLAYER_ALREADY_EXIST => "PlayerAlreadyExist",
        ERROR_NOT_SELECTED_PLAYER => "PlayerNotSelected",
        SELECTED_PLAYER_NOT_EXIST => "SelectedPlayerNotExist",
        PLAYER_NOT_ENOUGH_BALANCE=> "PlayerNotEnoughBalance",
        INVALID_NUGGET_INDEX => "SpecifiedNuggetIndexNotFound",
        PLAYER_NOT_ENOUGH_INVENTORY=> "PlayerInventoryFull",
        ERROR_BID_PRICE_INSUFFICIENT => "BidPriceInsufficient",
        ERROR_NUGGET_ATTRIBUTES_ALL_EXPLORED => "NuggetAttributeAllExplored",
        _ => "Unknown",
    }
}
