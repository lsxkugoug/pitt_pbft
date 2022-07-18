use std::fs::OpenOptions;

use futures::prelude::*;
use serde_json::json;
use tokio::net::TcpStream;
use tokio_serde::formats::*;
use tokio_util::codec::{FramedWrite, LengthDelimitedCodec};

mod constant;
mod message;

#[tokio::main]
pub async fn main() {
    
    // Bind a server socket
    let socket = TcpStream::connect("127.0.0.1:2022").await.unwrap();

    // Delimit frames using a length header
    let length_delimited = FramedWrite::new(socket, LengthDelimitedCodec::new());

    // Serialize frames with JSON
    let mut serialized =
        tokio_serde::SymmetricallyFramed::new(length_delimited, SymmetricalJson::default());
    print!("send json");
    let client_request = message::Client_msg{msg_type: constant::CLIENT_REQUEST, operation:"1234".to_string(), time_stamp: "1234".to_string(), signature: vec![1,2,3,4]};
    let json_obj = json!(&client_request);
    
    
    // Send the value
    serialized
        .send(json_obj)
        .await
        .unwrap()
        
}