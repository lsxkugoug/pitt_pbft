use std::sync::{Arc, Mutex};
use tokio::net::{TcpListener, TcpStream};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use serde_json::Value;
use crate::{message::*, server, constants, config};
use tokio::time::{sleep, Duration};

pub async fn do_client_request(msg: &ClientMsg, server_mutex: Arc<Mutex<server::Server>>) {
    print!("successfully enter client request");
    let who_leader = usize::max_value();
    {
        let mut server = server_mutex.lock().unwrap(); 
        server.client_request[msg.who_send] = (msg.time_stamp.clone(), constants::PRE_PREPARED);
    }
    // do leader operation
    if constants::get_i_am() == who_leader {
        // enter pre prepare
        send_pre_prepare(msg, server_mutex)
    }else {
    // do backup operation: make a timer
    // after timeout, check the request status, if it is APPLIED or timestamp is already upated, do normal, else make view change
        sleep(Duration::from_millis(constants::TIMEOUT)).await;
        let mut failed = false;
        {
            let mut server = server_mutex.lock().unwrap(); 
            if server.client_request[msg.who_send].0 == msg.time_stamp && server.client_request[msg.who_send as usize].1 == constants::PRE_PREPARED{
                 failed = true;
            }
        }
        if failed {
            // to do view change
        }
    }
}

// only for leader send pre_prepare msg
pub fn send_pre_prepare(msg: &ClientMsg, server_mutex: Arc<Mutex<server::Server>>) {
    let mut new_log: server::Log_entry = Default::default();
    {
        let mut server = server_mutex.lock().unwrap(); 

        // 1. check wether there are enough slot, related resend todo 
        if server.log_assign >= config::L as i32 {
            todo!()
        }
        // 0. generate log  
        let mut new_log = server::Log_entry {
            log_type: constants::CLIENT_REQUEST,
            v: server.my_view,
            n: server.h + server.log_assign,
            client: msg.who_send,
            who_send: constants::get_i_am(),
            cert_prepare_num: 1,
            cert_prepare_vote: vec![false;  config::SERVER_NUM],
            cert_commit_num: 0,
            cert_commit_vote: vec![false;  config::SERVER_NUM],
        };
        let log_assign = server.log_assign as usize;
        new_log.cert_prepare_vote[constants::get_i_am()] = true; 
        server.log[log_assign] = new_log;
        server.log_assign += 1;
    }
    // let pre_prepare_msg = message::Pre_prepare_msg


}

// only for backup who receive pre_prepare msg
pub fn rec_pre_prepare() {

}

pub fn do_prepare() {

}

pub fn do_commit() {

}

pub fn do_vc() {

}