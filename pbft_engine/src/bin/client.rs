// a simple client 0 simulator, just used to test engine code.

use std::fs::OpenOptions;

use futures::prelude::*;
use pbft_engine::cryptography;
use pbft_engine::cryptography::load_private_key;
use serde_json::json;
use tokio::net::TcpStream;
use tokio_serde::formats::*;
use tokio_util::codec::{FramedWrite, LengthDelimitedCodec};

use pbft_engine::constants;
use pbft_engine::message;

use ring::{rand, signature};
use std::time::{Duration, SystemTime};
use p256::ecdsa::{SigningKey, VerifyingKey};
use std::time::UNIX_EPOCH;

// note: for sign msg, only sign message::ClientMsg, but not any enum which can wrap message::ClientMsg.
#[tokio::main]
pub async fn main() {
    
    // connect a server socket
    let socket = TcpStream::connect("127.0.0.1:2022").await.unwrap();

    // Delimit frames using a length header
    let length_delimited = FramedWrite::new(socket, LengthDelimitedCodec::new());

    // Serialize frames with JSON
    let mut serialized =
        tokio_serde::SymmetricallyFramed::new(length_delimited, SymmetricalJson::default());
    let client_request = message::ClientMsg{
        msg_type: constants::CLIENT_REQUEST,
        who_send: 0,
        operation: "this is operation".to_string(),
        time_stamp: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis().to_string(),
    };
    // sign it
    let sign_key = load_private_key("./key_pairs/client/0/pri_key".to_string());
    let signature = cryptography::sign_msg(&sign_key, &bincode::serialize(&client_request).unwrap());
    // wrap msg as enum
    let client_request_enum = message::Msg::ClientMsg(client_request);
    let send_obj = message::MsgWithSignature{msg_without_sig: client_request_enum, signature: signature};
    let json_obj = json!(&send_obj);
    // Send the value
    serialized
        .send(json_obj)
        .await
        .unwrap()
        
}