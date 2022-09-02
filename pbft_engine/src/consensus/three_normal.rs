//! This file contains three normal phases logic
//! (1) client msg come in
//! (2) receive pre-prepare
//! (3) receive prepare
//! (4) receive commit
 
use std::cmp::max;
use std::sync::{Arc, Mutex};
use std::vec;
use crate::network::message::{MsgWithSignature, ClientMsg, ClientReplyMsg};
use crate::{network::message, consensus::model, consensus::view_change,constants, config};
use tokio::time::{sleep, Duration};

pub async fn do_client_request(client_msg: message::ClientMsg, server_mutex: Arc<Mutex<model::Server>>, client_sig: Vec<u8>) {
    // print!("successfully enter client request");
    let mut who_leader = usize::max_value();
    {
        let mut server = server_mutex.lock().unwrap(); 
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
            view_change::do_view_change(server_mutex);
        }
    }
}

// only for leader process client_request (1) generate pre-prepare msg and store it into log (2) broadcast pp_msg to all servers
pub async fn leader_do_client_request(client_msg: message::ClientMsg, server_mutex: Arc<Mutex<model::Server>>, client_sig: Vec<u8>) {
    let new_log: model::Log_entry = Default::default();
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
            entry_status: constants::PRE_PREPARED,
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
            advanced_prepare: vec![vec![]; config::SERVER_NUM],
            advanced_commit: vec![vec![]; config::SERVER_NUM],
        };
        // 3. change 
        let log_assign = server.log_assign as usize;
        new_log.cert_prepare_vote[constants::get_i_am()] = true; 
        server.log.set(log_assign, new_log);
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
pub async fn do_pre_prepare(pre_prepare_msg: message::PrePrepareMsg, server_mutex: Arc<Mutex<model::Server>>) {

    // 1. check wether my log already has this msg, if not, add it
    let mut has_prev = false;
    let mut commit_msg = None;

    {
        let mut server = server_mutex.lock().unwrap();
        let server_h = server.h;
        let who_leader = server.who_leader;
        has_prev = !server.log.get((pre_prepare_msg.n - server_h) as usize).client_msg.is_none();
        if !has_prev { // add it
            let log_assigned = &mut server.log.get((pre_prepare_msg.n - server_h) as usize);
            log_assigned.client_msg = Some(pre_prepare_msg.client_msg.clone());
            let msg_check_sum = message::get_client_msg_sha256(&pre_prepare_msg.client_msg);
            log_assigned.client_msg_checksum = msg_check_sum;
            log_assigned.entry_status = constants::PRE_PREPARED;
            // for log type, since the node may receive prepare or commit msg before it receive pre-prepare msg
            // so we should check advanced_prepare and advanced_commit array before assign it
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
            if log_assigned.entry_status == constants::PRE_PREPARED {
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
    if !commit_msg.is_none() {
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

pub async fn do_prepare(prepare_msg: message::PrepareMsg, server_mutex: Arc<Mutex<model::Server>>) {
    let mut commit_msg: Option<message::CommitMsg> = None;
    // 1. change log and client_request
    {
        let mut server = server_mutex.lock().unwrap();
        let server_h = server.h;
        let log_assigned = &mut server.log.get((prepare_msg.n - server_h) as usize);
        // if we dont receive corresponding pre-prepare msg, we just store the check sum into lot_entry.advance_prepare
        // and dont do the next step process until received pre-prepare msg from leader
        if log_assigned.client_msg.is_none() {
            log_assigned.advanced_prepare[constants::get_i_am()] = prepare_msg.client_msg_checksum;
            return;
        }
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

pub async fn do_commit(commit_msg: message::CommitMsg, server_mutex: Arc<Mutex<model::Server>>) {
    let mut try_apply_ready = false;
    let mut client = usize::max_value();
    // 1. change log and client_request
    {
        let log_pointer = usize::max_value();
        let mut server = server_mutex.lock().unwrap();
        let log_pointer = (commit_msg.n - server.h) as usize;
        let log_assigned = server.log.get(log_pointer);
        // if commit msg come before pre-prepare msg
        if log_assigned.client_msg.is_none() {
            log_assigned.advanced_prepare[constants::get_i_am()] = commit_msg.client_msg_checksum;
            return;
        }
        log_assigned.cert_commit_num += 1;
        log_assigned.cert_commit_vote[commit_msg.who_send] = true;
        if log_assigned.cert_commit_num > 2 * config::F_NUM {
            try_apply_ready = true;
            log_assigned.entry_status = max(log_assigned.entry_status, constants::COMMIT);
            let client = log_assigned.client_msg.as_ref().unwrap().who_send;
            server.client_request[client].1 = max(server.client_request[client].1, constants::COMMIT);
        }
    }
    // 2. if apply_ready, reply client and try to apply it
    if try_apply_ready {
        reply_apply(commit_msg.n , server_mutex).await;
    }
}

// todo 
// after one log is commited, 
// (1) server should reply the sender that "your request is commited, you can send the next one"
// (2) server should check wether I can apply the log if previous logs are applied
pub async fn reply_apply(seq_num: i32, server_mutex: Arc<Mutex<model::Server>>) {
    // 1. if previous logs are applied, move apply pointer, and apply it. 
    // In engine context, apply means tell all application to apply the operation
    // besides, if applied pointer meet apply'n % k == 0, gc should start

    // 1.1 get applied sequence number
    let mut pointers = Vec::new();
    let mut applying_logs = Vec::new();
    let mut last_applied = -1;
    {
        let mut server = server_mutex.lock().unwrap();
        loop {
            let pointer = (server.applied + 1 - server.h) as usize;
            // server.
            let cur_log = server.log.get(pointer);
            if cur_log.client_msg.is_none() || pointer >= config::L{
                break;
            }
            pointers.push(pointer);
            applying_logs.push(cur_log.client_msg.clone());
            server.applied = cur_log.n;
        }        
        last_applied = server.applied;
    }
    // 1.2 apply them
    for try_apply in &applying_logs {
        todo!("apply them ");
    }
    // 1.3 change the state of server
    {
        let mut server = server_mutex.lock().unwrap();
        for pointer in pointers {
            let cur_log = server.log.get(pointer);
            cur_log.entry_status = constants::APPLIED;
        }
    }

    // 2. check applied sequence number, if it meet K, send checkpoint msg to others and change quorum_h
    // 2.1 understand wether I should broadcast cp msg and slid window if I can
    // let mut make_cp = false;
    // let mut log_entries_write = Vec::new();
    // {
    //     let mut server = server_mutex.lock().unwrap();
    //     let mut new_h = server.h;
    //     // find new check point and assign it to quorum_cp
    //     while new_h + config::K as i32 <= last_applied {
    //         new_h += config::K as i32;
    //         make_cp = true;
    //     }
    //     server.quorum_cp[constants::get_i_am()] = max(new_h, server.quorum_cp[constants::get_i_am()]);
    //     // 2.2 find the quorum's largest checkpoint
    //     let mut quorum_cp = server.quorum_cp.clone();
    //     quorum_cp.sort_by(|a, b| b.cmp(a));
    //     let q_cp = quorum_cp[(config::F_NUM * 2) as usize];
    //     let mut server_h = server.h;
    //     // find logs which should be truncated
    //     while server_h + config::K as i32 <= q_cp && server_h + config::K as i32 <= last_applied{
    //         for _ in 0..config::K {
    //             // let server_h = server.h;
    //             log_entries_write.push(server.log.get(server_h as usize).clone());
    //             server_h += 1;
    //         }
    //         server.log.slid_window(); // todo we have to store the logs, after unlock, then write to the disk
    //     }
    //     server.h = server_h;
    // }
    // // write logs to disk, log_entries_write
    // let check_num = log_entries_write.len() / config::K;
    // for i in 0..check_num {
    //     todo!("write to disk")
    // }


    // if I my last applied sequence number > previous + K, make check point msg, broadcast to all other replicas
    // if make_cp {
    //     todo!("why need state digest of state");
    // }
    // 3. return msg to client
    // let mut client = usize::max_value();
    // let mut msg_with_sig: Option<MsgWithSignature> = None;
    // {
    //     let mut server = server_mutex.lock().unwrap();
    //     if server.applied >= seq_num {

    //     }
    //     let reply_msg = message::ClientReplyMsg {};
    //     msg_with_sig = Some(message::sign_and_add_signature(message::Msg::ClientReplyMsg(reply_msg)));
    // }
    // return to client 
    for client_msg in &applying_logs {
        let msg_without_sig = message::Msg::ClientReplyMsg(ClientReplyMsg{});
        message::send_client(client_msg.as_ref().unwrap().who_send, msg_without_sig).await;
    }
    
    
}

