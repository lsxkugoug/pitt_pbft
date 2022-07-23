use pitt_pbft::{consensus::*, server, config, cmd, message, constants};
use clap::Parser;
use std::sync::{Arc,Mutex};
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
        std::env::set_var("RUST_LOG", "info");
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
    
    constants::init_constants(i_am);
    let server: server::Server = Default::default();
    // make view change todo

    let server_mutex = Arc::new(Mutex::new(server));
    loop {
        // The second item contains the IP and port of the new connection.
        let (mut socket, _) = listener.accept().await.unwrap();
        let server_mutex = Arc::clone(&server_mutex);
        tokio::spawn(async move {
            preprocess(socket,server_mutex).await;
        });
    }
}

//parse requests
async fn preprocess(mut socket: TcpStream, server_mutex: Arc<Mutex<server::Server>>) {
        // 1. parse incoming request
        let length_delimited = FramedRead::new(socket, LengthDelimitedCodec::new());
        let mut deserialized = tokio_serde::SymmetricallyFramed::new(
            length_delimited,
            SymmetricalJson::<Value>::default(),
        );

        // get msg type abnd do some check
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

        // 2. process the job, and check data
        match msg_type {
            CLIENT_REQUEST =>{
                let msg: message::Client_msg = match serde_json::from_value(msg){
                    Ok(msg) => msg,
                    Err(msg) => {
                        log::info!("cant convert to expected msg object, check received msg: {}", msg);
                        return 
                    },
                };
                if !message::check_client_request(&msg) {
                    log::info!("signature verification failed");
                    return;
                }
                do_client_request(&msg, server_mutex).await;
            } 
            PRE_PREPARE => todo!(),
            PREPARE => todo!(),
            COMMIT => todo!(),
            VIEW_CHANGE => todo!(),
            _ => {
                log::info!("no match type, stop process this message");
            }
        }
    //
}

