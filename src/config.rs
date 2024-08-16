use crate::settlement::SettleMentInfo;
use serde::Serialize;

#[derive(Serialize, Clone)]
pub struct Config {
    cards: Vec<u32>,
}

lazy_static::lazy_static! {
    pub static ref CONFIG: Config = Config {
        cards: vec![0,1,2,3],
    };
}

impl Config {
    pub fn to_json_string() -> String {
        serde_json::to_string(&CONFIG.clone()).unwrap()
    }
    pub fn flush_settlement() -> Vec<u8> {
        SettleMentInfo::flush_settlement()
    }
}
