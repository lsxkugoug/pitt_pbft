//! This file contains three normal phases logic
//! (1) client msg come in
//! (2) receive pre-prepare
//! (3) receive prepare
//! (4) receive commit
 
use std::cmp::max;
use std::sync::{Arc, Mutex};
use std::vec;
use crate::{network::message, consensus::model, constants, config};
use tokio::time::{sleep, Duration};

pub async fn do_client_request(client_msg: message::ClientMsg, server_mutex: &Arc<Mutex<model::Server>>, client_sig: Vec<u8>) {
    print!("successfully enter client request");
    let who_leader = usize::max_value();
    {
        let mut server = server_mutex.lock().unwrap(); 
        server.client_request[client_msg.who_send] = (client_msg.time_stamp.clone(), constants::CLIENT_SEND);
    }
    // do leader operation
    if constants::get_i_am() == who_leader {
        // enter pre prepare
        leader_do_client_request(client_msg, server_mutex, client_sig).await
    }else {
    // do backup operation: make a timer
    // after timeout, check the request status, if it is APPLIED or timestamp is already upated, do normal, else make view change
        sleep(Duration::from_millis(constants::TIMEOUT)).await;
        let mut failed = false;
        {
            let mut server = server_mutex.lock().unwrap(); 
            if server.client_request[client_msg.who_send].0 == client_msg.time_stamp && server.client_request[client_msg.who_send as usize].1 < constants::PRE_PREPARED{
                 failed = true;
            }
        }
        if failed {
            // to do view change
        }
    }
}

// only for leader process client_request (1) generate pre-prepare msg and store it into log (2) broadcast pp_msg to all servers
pub async fn leader_do_client_request(client_msg: message::ClientMsg, server_mutex: &Arc<Mutex<model::Server>>, client_sig: Vec<u8>) {
    let mut new_log: model::Log_entry = Default::default();
    let mut v = -1;
    let mut n = -1;
    // 1. generate pre-prepare msg and store it into log 
    {
        let mut server = server_mutex.lock().unwrap(); 
        // 1. check wether there are enough slot, related resend todo 
        if server.log_assign >= config::L as i32 {
            todo!()
        }
        // 2. generate log  
        let mut new_log = model::Log_entry {
            log_type: constants::PRE_PREPARED,
            client_msg: Some(client_msg.clone()),
            client_msg_checksum: message::get_client_msg_sha256(&client_msg),
            v: server.my_view,
            n: server.h + server.log_assign,
            client: client_msg.who_send,
            who_send: constants::get_i_am(),
            cert_prepare_num: 1,
            cert_prepare_vote: vec![false;  config::SERVER_NUM],
            cert_commit_num: 0,
            cert_commit_vote: vec![false;  config::SERVER_NUM],
        };
        // 3. change 
        let log_assign = server.log_assign as usize;
        new_log.cert_prepare_vote[constants::get_i_am()] = true; 
        server.log[log_assign] = new_log;
        server.log_assign += 1;
        v = server.my_view;
        n = server.h + server.log_assign;
    }

    // 2. construct pre-prepare msg and broadcast to all servers
    let pp_msg = message::PrePrepareMsg {
        msg_type: constants::PRE_PREPARE,
        client_msg: client_msg,
        client_msg_sig: client_sig,
        who_send: constants::get_i_am(),
        v: v,
        n: n,
    };
    message::broadcast_servers(message::Msg::PrePrepareMsg(pp_msg)).await;
}

// only backups receive pre-prepare msg, leader doesn't
// if execute this function, that means msg already pass the check function. So, it has two meaning
// (1) This is the first time I receive this pre_prepare_msg. So the corresponding logEntry.client_msg == None, in this case, add it to log first
// (2) I have received thie pre_prepare_msg preveiously, and the corresponding logEntry contains it, in this case, just broadcast prepare
pub async fn do_pre_prepare(pre_prepare_msg: message::PrePrepareMsg, server_mutex: &Arc<Mutex<model::Server>>) {

    // 1. check wether my log already has this msg, if not, add it
    let mut has_prev = false;
    {
        let mut server = server_mutex.lock().unwrap();
        let server_h = server.h;
        has_prev = !server.log[(pre_prepare_msg.n - server.h) as usize].client_msg.is_none();
        if !has_prev { // add it
            let mut new_log = model::Log_entry {
                client_msg: Some(pre_prepare_msg.client_msg.clone()),
                client_msg_checksum: message::get_client_msg_sha256(&pre_prepare_msg.client_msg),
                log_type: constants::PRE_PREPARED,
                v: pre_prepare_msg.v,   // it should be rec_msg.v and n, since server.status == vc when execute this function
                n: pre_prepare_msg.n,
                client: pre_prepare_msg.client_msg.who_send,
                who_send: pre_prepare_msg.who_send,
                cert_prepare_num: 2,
                cert_prepare_vote: vec![false; config::SERVER_NUM],
                cert_commit_num: 0,
                cert_commit_vote: vec![false; config::SERVER_NUM],
                };
            new_log.cert_commit_vote[server.who_leader as usize] = true;
            new_log.cert_commit_vote[constants::get_i_am()] = true;
            server.log[(pre_prepare_msg.n - server_h) as usize] = new_log;
        }
    }
    
    // create prepare msg
    let prepare_msg = message::PrepareMsg {
        msg_type: constants::PREPARE,
        client_msg_checksum: message::get_client_msg_sha256(&pre_prepare_msg.client_msg),
        who_send: constants::get_i_am(),
        v: pre_prepare_msg.v,
        n: pre_prepare_msg.n,
    };
    message::broadcast_servers(message::Msg::PrepareMsg(prepare_msg)).await;
    
}

pub async fn do_prepare(prepare_msg: message::PrepareMsg, server_mutex: &Arc<Mutex<model::Server>>) {
    let mut commit_msg: Option<message::CommitMsg> = None;
    // 1. change log and client_request
    {
        let mut server = server_mutex.lock().unwrap();
        let server_h = (prepare_msg.n - server.h) as usize;
        
        server.log[server_h].cert_prepare_num += 1;
        server.log[server_h].cert_prepare_vote[prepare_msg.who_send] = true;
        if server.log[server_h].cert_prepare_num > 2 * config::F_NUM {
            server.log[server_h].log_type = max(server.log[server_h].log_type, constants::PREPARED);
            let client = server.log[server_h].client_msg.as_ref().unwrap().who_send;
            server.client_request[client].1 = max(server.client_request[client].1, constants::PREPARED);
            commit_msg = Some(message::CommitMsg {
                msg_type: constants::COMMIT,
                client_msg_checksum: server.log[server_h].client_msg_checksum.clone(),
                who_send: constants::get_i_am(),
                v: prepare_msg.v,
                n: prepare_msg.n,
            });
         }
    }
    // 2. if meet 2f + 1, start commit phase
    if !commit_msg.is_none() {
        message::broadcast_servers(message::Msg::CimmitMsg(commit_msg.unwrap())).await;
    }

}

pub async fn do_commit(commit_msg: message::CommitMsg, server_mutex: &Arc<Mutex<model::Server>>) {
    let mut apply_ready = false;
    let mut client = usize::max_value();
    // 1. change log and client_request
    {
        let mut server = server_mutex.lock().unwrap();
        let server_h = (commit_msg.n - server.h) as usize;
        server.log[server_h].cert_commit_num += 1;
        server.log[server_h].cert_commit_vote[commit_msg.who_send] = true;
        if server.log[server_h].cert_commit_num > 2 * config::F_NUM {
            server.log[server_h].log_type = max(server.log[server_h].log_type, constants::COMMIT);
            let client = server.log[server_h].client_msg.as_ref().unwrap().who_send;
            server.client_request[client].1 = max(server.client_request[client].1, constants::COMMIT);
        }
    }
    // 2. if apply_ready, reply client and try to apply it
    if apply_ready {
        // 1. return msg to client

        // 2. apply it, or just store it as a distributed database
    }
}

pub async fn do_vc() {

}