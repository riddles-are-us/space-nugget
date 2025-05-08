use crate::nugget::MarketNugget;
use crate::nugget::NuggetInfo;
use zkwasm_rest_convention::IndexedObject;
use zkwasm_rust_sdk::require;
use zkwasm_rest_abi::WithdrawInfo;
use zkwasm_rest_convention::WithBalance;
use zkwasm_rest_convention::BidObject;
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
    Recycle(u64),
    Explore(u64),
    List(u64, u64),
}

const NUGGET_INFO:u64 = 1;
const MARKET_INFO:u64 = 2;

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
                            player.data.cost_balance(5000)?;
                            let mut global = GLOBAL_STATE.0.borrow_mut();
                            let mut nugget = NuggetInfo::new_object(NuggetInfo::new(global.total, rand[1]), global.total);
                            nugget.data.compute_sysprice();
                            nugget.store();
                            NuggetInfo::emit_event(NUGGET_INFO, &nugget.data);
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
                            if nugget.data.marketid != 0 {
                                Err(NUGGET_IN_USE)
                            } else {
                                player.data.cost_balance(nugget.data.sysprice / 4)?;
                                nugget.data.explore(rand[2])?;
                                nugget.data.compute_sysprice();
                                NuggetInfo::emit_event(NUGGET_INFO, &nugget.data);
                                nugget.store();
                                player.store();
                                Ok(())
                            }
                        }
                    },

                    Activity::List(index, askprice) => {
                        if player.data.inventory.len() <= (*index) as usize {
                            Err(INVALID_NUGGET_INDEX)
                        } else {
                            let nuggetid = player.data.inventory[*index as usize];
                            let mut nugget = NuggetInfo::get_object(nuggetid).unwrap();
                            if nugget.data.marketid != 0 {
                                Err(NUGGET_IN_USE)
                            } else {
                                player.data.inventory.swap_remove(*index as usize); // remove
                                player.data.cost_balance(500)?;
                                let mut global = GLOBAL_STATE.0.borrow_mut();
                                // we should not fail after this point
                                let market_id = global.total;
                                global.total += 1;
                                nugget.data.marketid = market_id;
                                let market_nugget = MarketNugget::new(market_id, *askprice, 0, None, nugget.data.clone(), player.player_id);
                                let marketinfo = MarketNugget::new_object(market_nugget, market_id);
                                nugget.store();
                                marketinfo.store();
                                player.store();
                                NuggetInfo::emit_event(NUGGET_INFO, &nugget.data);
                                MarketNugget::emit_event(MARKET_INFO, &marketinfo.data);
                                Ok(())
                            }
                        }
                    },


                    Activity::Recycle(index) => {
                        if player.data.inventory.len() <= (*index) as usize {
                            Err(INVALID_NUGGET_INDEX)
                        } else {
                            let nuggetid = player.data.inventory[*index as usize];
                            let mut nugget = NuggetInfo::get_object(nuggetid).unwrap();
                            player.data.inc_balance(nugget.data.sysprice);
                            nugget.data.cycle = 1;
                            player.data.inventory.swap_remove(*index as usize);
                            nugget.store();
                            player.store();
                            Ok(())
                        }
                    },
                    Activity::Sell(index) => {
                        let market_info = MarketNugget::get_object(*index);
                        match market_info {
                            None => {
                                Err(INVALID_MARKET_INDEX)
                            },
                            Some(mut market)=> {
                                let mut bidder = market.data.0.deal()?;
                                let mut nugget = NuggetInfo::get_object(market.data.0.object.id).unwrap();
                                market.data.0.settleinfo = 2;
                                nugget.data.marketid = 0;
                                bidder.data.inventory.push(nugget.data.id);
                                nugget.store();
                                market.store();
                                bidder.store();
                                MarketNugget::emit_event(MARKET_INFO, &market.data);
                                NuggetInfo::emit_event(NUGGET_INFO, &nugget.data);
                                Ok(())
                            }
                        }
                    },

                    Activity::Bid(mid, price) => {
                        let market_info = MarketNugget::get_object(*mid);
                        match market_info {
                            Some(mut market) => {
                                let lastbidder = market.data.0.replace_bidder(player, *price)?;
                                if *price >= market.data.0.askprice {
                                    market.data.0.settleinfo = 2;
                                    market.data.0.deal()?;
                                    player.data.inventory.push(market.data.0.object.id);
                                    let mut n = NuggetInfo::get_object(market.data.0.object.id).unwrap();
                                    n.data.marketid = 0;
                                    n.store();
                                    market.store();
                                    NuggetInfo::emit_event(NUGGET_INFO, &n.data);
                                } else {
                                    market.data.0.settleinfo = 1;
                                    market.store();
                                }
                                lastbidder.map(|p| p.store());
                                player.store();
                                MarketNugget::emit_event(MARKET_INFO, &market.data);
                                Ok(())
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
        INVALID_MARKET_INDEX => "InvalidMarketIndex",
        ERROR_NO_BIDDER => "NoBidderForThisItem",
        ERROR_NOT_LISTED => "NuggetNotListed",
        _ => "Unknown",
    }
}
