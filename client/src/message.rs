use serde::{Deserialize, Serialize};
use serde_json::Result;
use ring::{
    rand,
    signature::{self, KeyPair},
};
#[derive(Serialize, Deserialize)]
pub struct Client_msg {
    pub msg_type: i32,
    pub operation: String,
    pub time_stamp: String,
    // pub signature: Vec<u8>     
}

pub struct  Msg_with_signature {
    msg: Client_msg,
    pub signature: Vec<u8>    
}