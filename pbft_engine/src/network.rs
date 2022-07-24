use crate::{consensus::*, server, config, cmd, message, constants};
use futures::prelude::*;
use serde_json::json;
use tokio::net::TcpStream;
use tokio_serde::formats::*;
use tokio_util::codec::{FramedWrite, LengthDelimitedCodec};

pub async fn send_server(server: usize, msg_with_sig: message::MsgWithSignature) {
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
pub async fn broadcast_servers(msg_without_sig: message::Msg) {
    let msg_with_sig = message::sign_and_add_signature(msg_without_sig);

    for server in 0..config::SERVER_IP.len() {
        let send_msg = msg_with_sig.clone();
        tokio::spawn(async move {
            send_server(server, send_msg).await;
        });
    }
}