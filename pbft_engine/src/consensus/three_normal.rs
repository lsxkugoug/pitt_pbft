//! This file contains three normal phases logic
//! (1) client msg come in
//! (2) receive pre-prepare
//! (3) receive prepare
//! (4) receive commit
 
use std::cmp::max;
use std::sync::{Arc, Mutex};
use std::vec;
use crate::network::message::{MsgWithSignature, ClientMsg, ClientReplyMsg};
use crate::{network::message, consensus::model::*, consensus::view_change,constants, config, network::check_msg::*};
use tokio::time::{sleep, Duration};


pub async fn do_client_request(client_msg: message::ClientMsg, server_mutex: Arc<Mutex<Server>>, client_sig: &Vec<u8>) {
    let mut who_leader = usize::max_value();
    {
        let mut server = server_mutex.lock().unwrap(); 
        if !check_client_request(&client_msg, client_sig, &mut server){
            return ;
        };
        server.client_request[client_msg.who_send] = (client_msg.time_stamp.clone(), constants::LOG_ENTRY_INIT);
        who_leader = server.who_leader as usize;
    }
    // do leader operation
    if constants::get_i_am() == who_leader {
        // enter pre prepare
        leader_do_client_request(client_msg, server_mutex, client_sig).await
    }else {
    // do backup operation: make a timer
    // after timeout, check the request status, if it is APPLIED or timestamp is already upated, do normal, else make view change
        sleep(Duration::from_millis(config::TIMEOUT)).await;
        let mut failed = false;
        {
            let mut server = server_mutex.lock().unwrap(); 
            if server.client_request[client_msg.who_send].0 == client_msg.time_stamp && server.client_request[client_msg.who_send as usize].1 < constants::PRE_PREPARED{
                 failed = true;
            }
        }
        if failed {
            // to do view change
            // 1. avoid race condition
            {
                let mut server = server_mutex.lock().unwrap(); 
                // delete client request
                // server.client_request[client_msg.who_send] = (-1, constan);
                if server.status == constants::DO_VIEW_CHANGE{
                    return;
                }
                server.status = constants::DO_VIEW_CHANGE;
                // change all of the log entries whose status < prepared to CAN_RESEND
                for i in 0..config::K {
                    let mut is_resend = !server.log.get_offset(i as i32).is_none() &&
                                                server.log.get_offset(i as i32).unwrap().entry_status < constants::PREPARED;
                    
                    if is_resend {
                        let who_send = server.log.get_offset(i as i32).unwrap().who_send;
                        server.client_request[who_send].1 = constants::CAN_RECEND;
                    }
                }
            }
            view_change::start_view_change(&server_mutex).await;
            {
                let mut server = server_mutex.lock().unwrap(); 
                server.status = constants::NORMAL;
            }
        }
    }
}

// only for leader process client_request (1) generate pre-prepare msg and store it into log (2) broadcast pp_msg to all servers
pub async fn leader_do_client_request(client_msg: message::ClientMsg, server_mutex: Arc<Mutex<Server>>, client_sig: &Vec<u8>) {
    let new_log: LogEntry = Default::default();
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
        let mut new_log = LogEntry {
            entry_status: constants::PRE_PREPARED,
            client_msg: Some(client_msg.clone()),
            client_msg_checksum: message::get_client_msg_sha256(&client_msg),
            v: server.my_view,
            n: server.get_h() + server.log_assign,
            client: client_msg.who_send,
            who_send: constants::get_i_am(),
            cert_prepare_num: 1,
            cert_prepare_vote: vec![false;  config::SERVER_NUM],
            cert_commit_num: 0,
            cert_commit_vote: vec![false;  config::SERVER_NUM],
            advanced_prepare: vec![vec![]; config::SERVER_NUM],
            advanced_commit: vec![vec![]; config::SERVER_NUM],
        };
        // 3. change 
        let log_assign = server.log_assign as usize;
        new_log.cert_prepare_vote[constants::get_i_am()] = true; 
        server.log.set_offset(log_assign as i32, new_log);
        server.log_assign += 1;
        v = server.my_view;
        n = server.get_h() + server.log_assign;
    }

    // 2. construct pre-prepare msg and broadcast to all servers
    let pp_msg = message::PrePrepareMsg {
        msg_type: constants::PRE_PREPARE,
        client_msg: client_msg,
        client_msg_sig: client_sig.clone(),
        who_send: constants::get_i_am(),
        v: v,
        n: n,
    };
    message::broadcast_servers(message::Msg::PrePrepareMsg(pp_msg)).await;

}

// only backups receive pre-prepare msg, leader doesn't
// if execute this function, that means msg already pass the check function. So, it has two meaning
// (1) This is the first time I receive this pre_prepare_msg. So the corresponding logEntry == None, in this case, add it to log first
// (2) I have received thie pre_prepare_msg preveiously, and the corresponding logEntry contains it, in this case, just broadcast prepare
pub async fn do_pre_prepare(pre_prepare_msg: message::PrePrepareMsg, server_mutex: Arc<Mutex<Server>>, msg_sig: &Vec<u8>) {

    // 1. check wether my log already has this msg, if not, add it
    let mut has_prev = false;
    let mut commit_msg = None;

    {
        let mut server = server_mutex.lock().unwrap();
        if !check_pre_prepare(&pre_prepare_msg, msg_sig, &mut server) {
            return ;
        }
        let server_h = server.get_h();
        let who_leader = server.who_leader;
        let offset = (pre_prepare_msg.n - server_h) as usize;
        has_prev = !server.log.get_offset(offset as i32).is_none();
        if !has_prev { // add it
            let log_assigned = &mut server.log.get_offset(offset as i32).unwrap();

            log_assigned.client_msg = Some(pre_prepare_msg.client_msg.clone());
            let msg_check_sum = message::get_client_msg_sha256(&pre_prepare_msg.client_msg);
            log_assigned.client_msg_checksum = msg_check_sum;
            log_assigned.entry_status = constants::PRE_PREPARED;
            // for log type, since the node may receive prepare or commit msg before it receive pre-prepare msg
            // so we should check advanced_prepare and advanced_commit array before assign it, if the checksum equals, add count
            // check advanced_prepare
            let mut log_type = constants::PRE_PREPARED;
            let mut count_prepare = 0;
            for (server_id, prepare_check_sum) in log_assigned.advanced_prepare.iter().enumerate() {
                if *prepare_check_sum == log_assigned.client_msg_checksum {
                    log_assigned.cert_prepare_vote[server_id] = true;
                    count_prepare += 1;
                }
            }
            if count_prepare >= 2 * config::F_NUM - 1 {
                log_type = max(constants::PREPARED, log_type);
            }
            // check advanced commit
            let mut count_commit = 0;
            for (server_id, commit_check_sum) in log_assigned.advanced_commit.iter().enumerate() {
                if *commit_check_sum == log_assigned.client_msg_checksum {
                    log_assigned.cert_prepare_vote[server_id] = true;
                    count_commit += 1;
                }
            }
            if count_commit >= 2 * config::F_NUM - 1 {
                log_type = max(constants::COMMIT, log_type);
            }
            log_assigned.v = pre_prepare_msg.v;
            log_assigned.n = pre_prepare_msg.n;
            log_assigned.client = pre_prepare_msg.client_msg.who_send;
            log_assigned.who_send = pre_prepare_msg.who_send;
            log_assigned.cert_prepare_num = 2 + count_prepare;
            log_assigned.cert_prepare_vote[who_leader as usize] = true;
            log_assigned.cert_prepare_vote[constants::get_i_am()] = true;

            // if msg is already prepare msg, should broadcast prepare msg
            if log_assigned.entry_status >= constants::PREPARED {
                commit_msg = Some(message::CommitMsg {
                    msg_type: constants::COMMIT,
                    client_msg_checksum: log_assigned.client_msg_checksum.clone(),
                    who_send: constants::get_i_am(),
                    v: log_assigned.v,
                    n: log_assigned.n,
                });
            }
        }
    }

    //2. if it is already prepared, send commit 
    if commit_msg.is_some() {
        message::broadcast_servers(message::Msg::CommitMsg(commit_msg.unwrap())).await;
        return;
    }

    //3. create prepare msg
    let prepare_msg = message::PrepareMsg {
        msg_type: constants::PREPARE,
        client_msg_checksum: message::get_client_msg_sha256(&pre_prepare_msg.client_msg),
        who_send: constants::get_i_am(),
        v: pre_prepare_msg.v,
        n: pre_prepare_msg.n,
    };
    message::broadcast_servers(message::Msg::PrepareMsg(prepare_msg)).await;
    
}

pub async fn do_prepare(prepare_msg: message::PrepareMsg, server_mutex: Arc<Mutex<Server>>, msg_sig: &Vec<u8>) {
    let mut commit_msg: Option<message::CommitMsg> = None;
    // 1. change log and client_request
    {
        let mut server = server_mutex.lock().unwrap();
        if !check_prepare(&prepare_msg, msg_sig, &mut server) {
            return;
        }
        let server_h = server.get_h();
        let offset = (prepare_msg.n - server_h) as i32;
        // if we dont receive corresponding pre-prepare msg, we just store the check sum into lot_entry.advance_prepare
        // and dont do the next step process until received pre-prepare msg from leader
        if server.log.get_offset(offset).is_none() {
            server.log.set_offset(offset, LogEntry::default());
            let mut log_assigned = server.log.get_offset(offset).unwrap();
            log_assigned.advanced_prepare[constants::get_i_am()] = prepare_msg.client_msg_checksum;
            return;
        }
        let mut log_assigned = server.log.get_offset(offset).unwrap();
        log_assigned.cert_prepare_num += 1;
        log_assigned.cert_prepare_vote[prepare_msg.who_send] = true;
        if log_assigned.cert_prepare_num > 2 * config::F_NUM {
            log_assigned.entry_status = max(log_assigned.entry_status, constants::PREPARED);
            let client = log_assigned.client_msg.as_ref().unwrap().who_send;
            server.client_request[client].1 = max(server.client_request[client].1, constants::PREPARED);
            commit_msg = Some(message::CommitMsg {
                msg_type: constants::COMMIT,
                client_msg_checksum: prepare_msg.client_msg_checksum,
                who_send: constants::get_i_am(),
                v: prepare_msg.v,
                n: prepare_msg.n,
            });
         }
    }
    // 2. if meet 2f + 1, start commit phase
    if !commit_msg.is_none() {
        message::broadcast_servers(message::Msg::CommitMsg(commit_msg.unwrap())).await;
    }
}

pub async fn do_commit(commit_msg: message::CommitMsg, server_mutex: Arc<Mutex<Server>>, msg_sig: &Vec<u8>) {
    let mut client = usize::max_value();
    // 1. change log and client_request
    {
        let offset = usize::max_value();
        let mut server = server_mutex.lock().unwrap();
        if !check_commit(&commit_msg, msg_sig, &mut server) {
            return;
        }
        let offset = (commit_msg.n - server.get_h()) as usize;
        // if commit msg come before pre-prepare msg
        if server.log.get_offset(offset as i32).is_none() {
            server.log.set_offset(offset as i32, LogEntry::default());
            server.log.get_offset(offset as i32).unwrap().advanced_prepare[constants::get_i_am()] = commit_msg.client_msg_checksum;
            return;
        }
        let mut log_assigned = server.log.get_offset(offset as i32).unwrap();
        log_assigned.cert_commit_num += 1;
        log_assigned.cert_commit_vote[commit_msg.who_send] = true;
        if log_assigned.cert_commit_num > 2 * config::F_NUM {
            log_assigned.entry_status = max(log_assigned.entry_status, constants::COMMITED);
            let client = log_assigned.client_msg.as_ref().unwrap().who_send;
            server.client_request[client].1 = max(server.client_request[client].1, constants::COMMITED);
        }
    }
    // 2. try apply
    try_reply_apply(server_mutex).await;
}

// todo 
// after one log is commited, 
// (1) server should reply the sender that "your request is commited, you can send the next one"
// (2) server should check wether I can apply the log if previous logs are applied
pub async fn try_reply_apply(server_mutex: Arc<Mutex<Server>>) {
    // 1. if previous logs are applied, move apply pointer, and apply it. 
    // In engine context, apply means tell all application to apply the operation
    // besides, if applied pointer meet apply'n % k == 0, gc should start

    // 1.1 get applied sequence number
    let mut pointers = Vec::new();
    let mut applying_logs = Vec::new();
    {
        let mut server = server_mutex.lock().unwrap();
        loop {
            let pointer = (server.applied + 1 - server.get_h()) as usize;
            // server.
            let cur_log = server.log.get_offset(pointer as i32);
            if cur_log.is_none() || pointer >= config::L{
                break;
            }
            pointers.push(pointer);
            applying_logs.push(cur_log.unwrap().clone());
            server.applied += 1;
        }        
    }
    // 1.2 apply them
    for try_apply in &applying_logs {
        todo!("apply them ");
    }
    // 1.3 change the state of server
    {
        let mut server = server_mutex.lock().unwrap();
        for pointer in pointers {
            
            let cur_log = server.log.get_offset(pointer as i32);
            cur_log.unwrap().entry_status = constants::APPLIED;
        }
    }

    for log_entry in &applying_logs {
        let msg_without_sig = message::Msg::ClientReplyMsg(ClientReplyMsg{});
        message::send_client(log_entry.client_msg.as_ref().unwrap().who_send, msg_without_sig).await;
    }
    
    
}

