use serde::{Deserialize, Serialize};
use serde_json::Result;

#[derive(Serialize, Deserialize)]
pub struct Client_msg {
    pub msg_type: i32,
    pub operation: String,
    pub time_stamp: String,
    pub signature: Vec<u8>    
}