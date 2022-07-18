use serde::{Deserialize, Serialize};
use serde_json::Result;

#[derive(Serialize, Deserialize)]
pub struct Client_msg {
    pub msg_type: i32,
    pub operation: String,
    pub time_stamp: String,
    pub signature: &[u8]    
}


#[derive(Serialize, Deserialize)]
pub struct Pre_prepare_msg {
    pub client_msg: i32,
    pub v: i32,
    pub n: i32,
    pub signature: &[u8]
}


#[derive(Serialize, Deserialize)]
pub struct Prepare_msg {
    pub client_msg: i32,

}

#[derive(Serialize, Deserialize)]
pub struct Commit_msg {
    pub client_msg: i32,

}

pub struct Vc_msg {
    
}

// sign and wrap msg before send
pub fn send_msg(&Str:msg) {
    
}

enum Msg {
    Client_msg,
    Pre_prepare_msg,
    Prepare_msg,
    Commit_msg
}

// pub fn parse_msg(json: Value) -> Msg {

// }