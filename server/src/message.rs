use std::thread::panicking;
use serde::{Deserialize, Serialize};
use serde_json::Result;

#[derive(Serialize, Deserialize)]
pub struct Client_msg {
    pub msg_type: i32,
    pub who_send: usize,
    pub operation: String,
    pub time_stamp: u128,
}

pub struct  Client_msg_signature {
    pub msg: Client_msg,
    pub signature: Vec<u8>
}


#[derive(Serialize, Deserialize)]
pub struct Pre_prepare_msg {
    pub client_msg: Client_msg,
    pub who_send: usize,
    pub v: i32,
    pub n: i32,
    pub signature: Vec<u8>    
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

// // sign and wrap msg before send
// pub fn send_msg(&Str:msg) {
    
// }

pub enum Msg {
    Client_msg,
    Pre_prepare_msg,
    Prepare_msg,
    Commit_msg
}

pub struct  Msg_with_signature {
    msg: Msg,
    pub signature: Vec<u8>    
}

// pub fn parse_msg(json: Value) -> Msg {

// }

// check function todo
pub fn check_client_request(msg: &Client_msg) -> bool {
    // check tampstemp

    // check signature


    return true
}