use chrono::{DateTime, Local};
use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Deserialize, Serialize)]
pub struct Clip {
    pub date: DateTime<Local>,
    pub payload: Vec<u8>,
}

impl From<&Clip> for Vec<u8> {
    fn from(data: &Clip) -> Self {
        rmp_serde::to_vec(&data).unwrap()
    }
}

impl From<&[u8]> for Clip {
    fn from(data: &[u8]) -> Self {
        rmp_serde::from_slice(&data).unwrap()
    }
}
