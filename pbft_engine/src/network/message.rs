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

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ClientMsg {
    pub msg_type: i32,
    pub who_send: usize,
    pub operation: String,
    pub time_stamp: String,
}


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

#[derive(Serialize, Deserialize, Clone)]
pub struct VcMsg {
    pub who_send: usize,
}


#[derive(Serialize, Deserialize, Clone)]
pub enum Msg {
    ClientMsg(ClientMsg),
    PrePrepareMsg(PrePrepareMsg),
    PrepareMsg(PrepareMsg),
    CimmitMsg(CommitMsg),
    VcMsg(VcMsg),
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
        Msg::PrePrepareMsg(msg_witout_signature) => bincode::serialize(msg_witout_signature).unwrap(),
        Msg::PrepareMsg(msg_witout_signature) => bincode::serialize(msg_witout_signature).unwrap(),
        Msg::CimmitMsg(msg_witout_signature) => bincode::serialize(msg_witout_signature).unwrap(),
        Msg::VcMsg(msg_witout_signature) => bincode::serialize(msg_witout_signature).unwrap()
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
    .await
    .unwrap()
}

// use server's private key sign the msg, and broadcast to all the servers except the server itself
pub async fn broadcast_servers(msg_without_sig:  Msg) {
    let msg_with_sig = sign_and_add_signature(msg_without_sig);

    for server in 0..config::SERVER_IP.len() {
        if server == constants::get_i_am() {
            continue;
        }
        let send_msg = msg_with_sig.clone();
        tokio::spawn(async move {
            send_server(server, send_msg).await;
        });
    }
}

