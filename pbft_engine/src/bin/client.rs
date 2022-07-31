//! a simple client 0 simulator, just used to test engine code. Send msg to all of replicas

use std::fs::OpenOptions;

use clap::Parser;
use futures::prelude::*;
use pbft_engine::{cryptography, config, cmd, network::message};
use pbft_engine::cryptography::load_private_key;
use serde_json::json;
use tokio::net::TcpStream;
use tokio_serde::formats::*;
use tokio_util::codec::{FramedWrite, LengthDelimitedCodec};

use pbft_engine::constants;

use std::time::{Duration, SystemTime};
use std::time::UNIX_EPOCH;


// set the number of operations you want to send to pbft engine
const REQUEST_NUM: i32 = 100; 

#[tokio::main]
pub async fn main() {
    let args = cmd::Args::parse();
    let mut i_am: usize = usize::max_value();
    let my_ip = args.ip;
    // get my identification
    for (idx, ip) in config::CLIENT_IP.iter().enumerate() {
        if my_ip == *ip {
            i_am = idx;
        }
    };
    unsafe{
        constants::init_constants(i_am);
    }
    // get my private key
    let sign_key = load_private_key(format!("./key_pairs/client/{}/pri_key", i_am));


    // open another thread to receive servers' msg


    for i in 0..REQUEST_NUM {
        for server in 0..config::SERVER_NUM {
            let client_request = message::ClientMsg{
                msg_type: constants::CLIENT_REQUEST,
                who_send: 0,
                operation: format!("this is msg {} from client {}", i, i_am).to_string(),
                time_stamp: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis().to_string(),
            };
            // sign it
            let signature = cryptography::sign_msg(&sign_key, &bincode::serialize(&client_request).unwrap());
            // wrap msg as enum
            let client_request_enum = message::Msg::ClientMsg(client_request);
            let send_obj = message::MsgWithSignature{msg_without_sig: client_request_enum, signature: signature};
            // Send the valuen  
            message::send_server(server, send_obj).await;        
        };
    };

        
}