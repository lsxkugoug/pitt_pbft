//! This file contains
//! (1) several message types
//! (2) some functions to send the message
//! (3) some function to sign and verify msg. The function based on cryptography mod
//! 
//! The msgs relationship:
//! 
//! MsgWithSignature{
//!     enum Msg{CM, PPM, PM, CM, VCM}, 
//!     signature for {CM, PPM, PM, CM, VCM} signd by sender, note: The signature based on bytes of {CM, PPM, PM, CM, VCM}
//! }
//! 

use serde::{Deserialize, Serialize};
use crate::{cryptography, constants, config};

use futures::prelude::*;
use serde_json::json;
use tokio::net::TcpStream;
use tokio_serde::formats::*;
use tokio_util::codec::{FramedWrite, LengthDelimitedCodec};

// 1. client msg
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ClientMsg {
    pub msg_type: i32,
    pub who_send: usize,
    pub operation: String,
    pub time_stamp: String,
}

    // just tell client "your request is commited"
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ClientReplyMsg {

}

// 2. three normal phase
#[derive(Serialize, Deserialize, Clone)]
pub struct PrePrepareMsg {
    pub msg_type: i32,
    pub client_msg: ClientMsg,
    pub client_msg_sig: Vec<u8>, 
    pub who_send: usize,
    pub v: i32,
    pub n: i32,
}


#[derive(Serialize, Deserialize, Clone)]
pub struct PrepareMsg {
    pub msg_type: i32,
    pub client_msg_checksum: Vec<u8>, 
    pub who_send: usize,    
    pub v: i32,
    pub n: i32,    
}

#[derive(Serialize, Deserialize, Clone)]
pub struct CommitMsg {
    pub msg_type: i32,
    pub client_msg_checksum: Vec<u8>, 
    pub who_send: usize,    
    pub v: i32,
    pub n: i32,    
}

// 3. view change msg
#[derive(Serialize, Deserialize, Clone)]
pub struct VcMsg {
    pub msg_type: i32,
    pub v: i32,
    pub n: i32,
    pub who_send: usize,
}



// 4. checkpoint management msg
#[derive(Serialize, Deserialize, Clone)]
pub struct CheckPointMsg {
    pub msg_type: i32,
    pub n: i32, // n % k should == 0
    // pub d: Vec<u8>, //? why need that todo
    pub who_send: usize,

}
// 5. retransmition msg
#[derive(Serialize, Deserialize, Clone)]
pub struct RtMsg {
    pub node_status: i32,
    pub v: i32,
    pub h: i32,     // last stable checkpoint sequence number
    pub le: i32,    // le is the last exected sequence number
    pub who_send: usize,
    pub log_status: Vec<i32>
}



#[derive(Serialize, Deserialize, Clone)]
pub enum Msg {
    ClientMsg(ClientMsg),
    ClientReplyMsg(ClientReplyMsg),
    PrePrepareMsg(PrePrepareMsg),
    PrepareMsg(PrepareMsg),
    CommitMsg(CommitMsg),
    VcMsg(VcMsg),
    RtMsg(RtMsg),
    CheckPointMsg(CheckPointMsg)
}

#[derive(Serialize, Deserialize, Clone)]
pub struct  MsgWithSignature {
    pub msg_without_sig: Msg,
    pub signature: Vec<u8>,
}



// use my private key sign the msg, (note: sign objet, but not Msg enum) and generate Msg_with_signature object
pub fn sign_and_add_signature(msg_witout_signature: Msg) -> MsgWithSignature {
    let msg_bytes = match &msg_witout_signature {
        Msg::ClientMsg(msg_witout_signature) => bincode::serialize(msg_witout_signature).unwrap(),
        Msg::ClientReplyMsg(msg_witout_signature) => bincode::serialize(msg_witout_signature).unwrap(),
        Msg::PrePrepareMsg(msg_witout_signature) => bincode::serialize(msg_witout_signature).unwrap(),
        Msg::PrepareMsg(msg_witout_signature) => bincode::serialize(msg_witout_signature).unwrap(),
        Msg::CommitMsg(msg_witout_signature) => bincode::serialize(msg_witout_signature).unwrap(),
        Msg::VcMsg(msg_witout_signature) => bincode::serialize(msg_witout_signature).unwrap(),
        Msg::RtMsg(msg_witout_signature) => bincode::serialize(msg_witout_signature).unwrap(),
        Msg::CheckPointMsg(msg_witout_signature) => bincode::serialize(msg_witout_signature).unwrap(),

    };
    let signature = cryptography::sign_msg(&constants::get_my_prikey().unwrap(), &msg_bytes);
    MsgWithSignature{msg_without_sig: msg_witout_signature, signature: signature}
}

pub fn get_client_msg_sha256(client_msg: & ClientMsg) -> Vec<u8> {
    cryptography::sha256(&bincode::serialize(client_msg).unwrap())
}


pub async fn send_server(server: usize, msg_with_sig: MsgWithSignature) {
    let socket = TcpStream::connect(config::SERVER_IP[server]).await.unwrap();
    let length_delimited = FramedWrite::new(socket, LengthDelimitedCodec::new());
    let mut serialized =
    tokio_serde::SymmetricallyFramed::new(length_delimited, SymmetricalJson::default());
    let json_obj = json!(&msg_with_sig);
    serialized
    .send(json_obj)
    .await.expect(&format!("send {}, failed", server.to_string()));
}

// use server's private key sign the msg, and broadcast to all the servers except the server itself
pub async fn broadcast_servers(msg_without_sig:  Msg) {
    let msg_with_sig = sign_and_add_signature(msg_without_sig);
    let mut tasks = Vec::new();
    for server in 0..config::SERVER_IP.len() {
        if server == constants::get_i_am() {
            continue;
        }
        let send_msg = msg_with_sig.clone();
        tasks.push(tokio::spawn(async move {
            send_server(server, send_msg).await;
        }));
    }
    for t in tasks {
        t.await;
    }
}


pub async fn send_client(client: usize, msg_without_sig: Msg) {
    let msg_with_sig = sign_and_add_signature(msg_without_sig);
    let socket = TcpStream::connect(config::CLIENT_IP[client]).await.unwrap();
    let length_delimited = FramedWrite::new(socket, LengthDelimitedCodec::new());
    let mut serialized =
    tokio_serde::SymmetricallyFramed::new(length_delimited, SymmetricalJson::default());
    let json_obj = json!(&msg_with_sig);
    serialized
    .send(json_obj)
    .await
    .unwrap()
}



