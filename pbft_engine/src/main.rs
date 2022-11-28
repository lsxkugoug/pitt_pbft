use pbft_engine::{consensus::three_normal, consensus::model, config, cmd, network::{message, self}, constants, network::msg_rt, checkpoint_gc::*};
use clap::Parser;
use std::sync::{Arc,Mutex};
use tokio::net::{TcpListener, TcpStream};

use futures::prelude::*;
use serde_json::Value;
use tokio_serde::formats::*;
use tokio_util::codec::{FramedRead, LengthDelimitedCodec};


// firstly check config, and return server's number based on config::SERVER_IP
async fn get_sever_num() -> (usize, String) {
    let args = cmd::Args::parse();
    let mut i_am: usize = usize::max_value();
    let my_ip = args.ip;
    // do some check
    for (idx, ip) in config::SERVER_IP.iter().enumerate() {
        if my_ip == *ip {
            i_am = idx;
        }
    };
    if i_am == usize::max_value() {
        log::error!("check config, the ip of this server is {}, but no record in config file", my_ip);
    }
    return (i_am, my_ip);
}

#[tokio::main]
async fn main() {
    // to do split this part to init server and bind port
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
    unsafe{
        constants::init_constants(i_am);
    }

    let server: model::Server = Default::default();
    // open a new thread to make msg retransmission


    let server_mutex = Arc::new(Mutex::new(server));
    let server_mutex_rt = Arc::clone(&server_mutex);

    tokio::spawn(async move {
        network::period_rt(server_mutex_rt).await;
    });


    
    loop {
        // The second item contains the IP and port of the new connection.
        let (socket, _) = listener.accept().await.unwrap();
        let server_mutex_pro = Arc::clone(&server_mutex);
        tokio::spawn(async move {
            preprocess_route(socket, server_mutex_pro).await;
        });
        // note: this tokio should not use join or await, since it wrapped by a loop, the tasks of sub thread must complete
    }
}

//parse requests
async fn preprocess_route(socket: TcpStream, server_mutex: Arc<Mutex<model::Server>>) {
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
        println!("{}", msg);
        let msg: message::MsgWithSignature = match serde_json::from_value(msg)  {
            Ok(msg) => msg,
            Err(_) => {
                log::info!("receive bad msg, omit it");
                return;
            },
        };



        // 3. process the job
        // match msg.msg_without_sig {
        //     message::Msg::ClientMsg(msg_without_sig) => {
        //         three_normal::do_client_request(msg_without_sig, server_mutex, &msg.signature, &msg.signature).await;
        //     },
        //     message::Msg::PrePrepareMsg(msg_without_sig)=> three_normal::do_pre_prepare(msg_without_sig, server_mutex,&msg.signature).await,
        //     message::Msg::PrepareMsg(msg_without_sig)=> three_normal::do_prepare(msg_without_sig, server_mutex, &msg.signature).await,
        //     message::Msg::CommitMsg(msg_without_sig)=> three_normal::do_commit(msg_without_sig, server_mutex, &msg.signature).await,
        //     message::Msg::VcMsg(msg_without_sig)=> todo!(),
        //     message::Msg::RtMsg(msg_without_sig)=> msg_rt::do_rt(msg_without_sig, server_mutex, &msg.signature).await,
        //     _ => {
        //         log::info!("no match type, stop process this message");
        //     }
        // }
    
}

