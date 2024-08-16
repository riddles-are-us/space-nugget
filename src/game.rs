use serde::{Serialize, Serializer, ser::SerializeSeq};


#[derive(Clone)]
pub struct CommitmentInfo ([u64; 2]);

// Custom serializer for `u64` as a string.
fn serialize_commitment_info<S>(value: &CommitmentInfo, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut seq = serializer.serialize_seq(Some(value.0.len()))?;
        for e in value.0.iter() {
            seq.serialize_element(&e.to_string())?;
        }
        seq.end()
    }

#[derive(Serialize, Clone)]
pub struct Content {
    #[serde(serialize_with="serialize_commitment_info")]
    pub commitment: CommitmentInfo,
    pub content: Option<Vec<u8>> // card contents
}

impl CommitmentInfo {
    pub fn new(c0: u64, c1: u64) -> Self {
        CommitmentInfo([c0, c1])
    }
}

#[derive(Serialize, Clone)]
pub struct Game {
    pub game_id: u64,
    pub contents: Vec<Content>,
}

impl Game {
    pub fn settle(&mut self) {
        todo!()
    }
}
