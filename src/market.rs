use std::slice::IterMut;
use crate::player::PlayerData;
use zkwasm_rest_abi::StorageData;
use zkwasm_rest_convention::BidInfo;
use zkwasm_rest_convention::IndexedObject;
use zkwasm_rest_convention::WithBalance;
use zkwasm_rest_convention::MarketInfo;
use zkwasm_rest_convention::BidObject;
use std::marker::PhantomData;
use crate::nugget::NuggetInfo;
use crate::Player;
use crate::error::*;
use crate::config::{NUGGET_INFO, MARKET_INFO};
use crate::config::MARKET_DEAL_DELAY;
use crate::state::GLOBAL_STATE;


impl BidObject<PlayerData> for MarketInfo<NuggetInfo, PlayerData> {
    const INSUFF:u32 = ERROR_BID_PRICE_INSUFFICIENT;
    const NOBID: u32 = ERROR_NO_BIDDER;
    fn get_bidder(&self) -> Option<BidInfo> {
        self.bid
    }

    fn set_bidder(&mut self, bidder: Option<BidInfo>) {
        self.bid = bidder;
    }

    fn get_owner(&self) -> [u64; 2] {
        self.owner
    }

    fn set_owner(&mut self, pid: [u64; 2]) { 
        self.owner = pid 
    }

}

pub struct MarketNugget (pub MarketInfo<NuggetInfo, PlayerData>);

impl MarketNugget {
    pub fn new(marketid: u64, askprice: u64, settleinfo: u64, bid: Option<BidInfo>, object: NuggetInfo, owner: [u64; 2]) -> Self {
        MarketNugget (MarketInfo {
            marketid,
            askprice,
            settleinfo,
            bid,
            object,
            owner,
            user: PhantomData
        })
    }
}

impl StorageData for MarketNugget {
    fn from_data(u64data: &mut IterMut<u64>) -> Self {
        MarketNugget (MarketInfo::<NuggetInfo, PlayerData>::from_data(u64data))
    }
    fn to_data(&self, data: &mut Vec<u64>) {
        self.0.to_data(data)
    }
}

impl IndexedObject<MarketNugget> for MarketNugget {
    const PREFIX: u64 = 0x1ee2;
    const POSTFIX: u64 = 0xfee2;
    const EVENT_NAME: u64 = 0x02;
}

pub fn bid(player: &mut Player<PlayerData>, mid: u64, price: u64, counter: u64) -> Result<(), u32> {
    let market_info = MarketNugget::get_object(mid);
    match market_info {
        Some(mut market) => {
            let lastbidder = market.data.0.replace_bidder(player, price)?;
            if price >= market.data.0.askprice {
                market.data.0.settleinfo = 2;
                market.data.0.deal()?;
                player.data.inventory.push(market.data.0.object.id);
                let mut n = NuggetInfo::get_object(market.data.0.object.id).unwrap();
                n.data.marketid = 0;
                n.store();
                market.store();
                NuggetInfo::emit_event(NUGGET_INFO, &n.data);
            } else {
                market.data.0.settleinfo = 1 + (counter << 16);
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

// 1. owner can settle the deal
// 2. the buyer can settle the deal if certain amount of time has passed
pub fn settle(player: &mut Player<PlayerData>, mid: u64, counter: u64) -> Result<(), u32> {
    let market_info = MarketNugget::get_object(mid);
    match market_info {
        None => {
            Err(INVALID_MARKET_INDEX)
        },
        Some(mut market)=> {
            if market.data.0.settleinfo == 0 || market.data.0.settleinfo == 2 {
                Err(INVALID_MARKET_INDEX)
            } else {
                let owner = market.data.0.get_owner();
                // calculate the time that has passed
                let delay = counter - (market.data.0.settleinfo >> 16);
                if player.player_id == owner || delay > MARKET_DEAL_DELAY {
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
                } else {
                    Err(INVALID_MARKET_INDEX)
                }
            }
        }
    }
}

pub fn list(player: &mut Player<PlayerData>, oid: u64, askprice: u64) -> Result<(), u32> {
    let mut nugget = NuggetInfo::get_object(oid).unwrap();
    if nugget.data.marketid != 0 {
        Err(NUGGET_IN_USE)
    } else {
        player.data.cost_balance(500)?;
        let mut global = GLOBAL_STATE.0.borrow_mut();
        // we should not fail after this point
        let market_id = global.total;
        global.total += 1;
        nugget.data.marketid = market_id;
        let market_nugget = MarketNugget::new(market_id, askprice, 0, None, nugget.data.clone(), player.player_id);
        let marketinfo = MarketNugget::new_object(market_nugget, market_id);
        nugget.store();
        marketinfo.store();
        NuggetInfo::emit_event(NUGGET_INFO, &nugget.data);
        MarketNugget::emit_event(MARKET_INFO, &marketinfo.data);
        Ok(())
    }
}



