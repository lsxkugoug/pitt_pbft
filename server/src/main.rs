mod cmd;
mod config;
mod server;
mod constant;
mod consensus;

use clap::Parser;
use consensus::*;
use std::sync::{Arc};
use tokio::sync::{Mutex};
use tokio::net::{TcpListener, TcpStream};
use tokio::io::{AsyncReadExt, AsyncWriteExt};


use futures::prelude::*;
use serde_json::Value;
use tokio_serde::formats::*;
use tokio_util::codec::{FramedRead, LengthDelimitedCodec};
// firstly check config, and return server's number based on config::SERVER_IP
async fn get_sever_num() -> (i32, String) {
    let args = cmd::Args::parse();
    let mut i_am: i32 = -1;
    let my_ip = args.ip;
    // do some check
    for (idx, ip) in config::SERVER_IP.iter().enumerate() {
        if my_ip == *ip {
            i_am = idx as i32;
        }
    };
    if i_am == -1 {
        log::error!("check config, the ip of this server is {}, but no record in config file", my_ip);
    }
    return (i_am, my_ip);
}

#[tokio::main]
async fn main(){
    // read public key and my private key todo

    if let Err(_) = std::env::var("RUST_LOG") {
        std::env::set_var("RUST_LOG", "debug");
    }
    pretty_env_logger::init();

    // Bind the listener to the address
    let (i_am, bind_ip) = get_sever_num().await;
    let listener = match TcpListener::bind(&bind_ip).await {
        Ok(listener) => listener,
        Err(err) => {
            log::error!("Could not bind to {}",&bind_ip);
            std::process::exit(1);
        }
    };
    log::info!("Listening for requests on {}, and my server number is {}", &bind_ip, i_am);
    
    let server: server::Server = Default::default();

    let server_mutex = Arc::new(Mutex::new(server));
    loop {
        // The second item contains the IP and port of the new connection.
        let (mut socket, _) = listener.accept().await.unwrap();
        let server_mutex = server_mutex.clone();
        tokio::spawn(async move {
            preprocess(socket,server_mutex).await;
        });
    }
}

//parse requests
async fn preprocess(mut socket: TcpStream, server: Arc<Mutex<server::Server>>) {
        // 1. parse incoming request
        
        let length_delimited = FramedRead::new(socket, LengthDelimitedCodec::new());
        let mut deserialized = tokio_serde::SymmetricallyFramed::new(
            length_delimited,
            SymmetricalJson::<Value>::default(),
        );

        
        let msg = match deserialized.try_next().await {
            Ok(msg) => {
                match msg {
                    Some(msg) => msg,
                    None => {
                        log::info!("server receives msg type wrong, stop solve this msg");
                        return
                    },
                }
            },
            Err(_) => {
                log::info!("server receives msg type wrong, stop solve this msg");
                return;
            },
        };

        let msg_type = match msg.get("msg_type")  {
            Some(msg_type) => match msg_type.to_string().parse::<i32>() {
                Ok(msg_type) => msg_type,
                Err(_) => {
                    log::info!("cant get correct message type, stop process this message");
                    return;
                }
            },
            None => {
                log::info!("cant get correct message type, stop process this message");
                return;
            },
        };

        match msg_type {
            Client_msg => do_client_request(msg, server),
            Pre_prepare_msg => todo!(),
            Prepare_msg => todo!(),
            Commit_msg => todo!(),
            _ => {
                log::info!("no match type, stop process this message");
            }
        }
    //
}

