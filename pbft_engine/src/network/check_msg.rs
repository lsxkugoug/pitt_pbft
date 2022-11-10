//! This file constains functions which check msg before process it. It never change the server struct, just do check!
//! When servers receive any msg, it should check them. If msg pass the corresponding checking, go to process functions.

use std::{sync::{Arc,Mutex, MutexGuard}, time::{SystemTime, UNIX_EPOCH}};

use crate::{network::message, consensus::model::*, constants, config, cryptography, network::msg_rt};


// pub fn check_msg(msg_with_sig: &message::MsgWithSignature, server_mutex: &Arc<Mutex<model::Server>>) ->bool {
//     let signature = &msg_with_sig.signature;
//     match &msg_with_sig.msg_without_sig {
//         message::Msg::ClientMsg(msg_without_sig) => {
//             check_client_request(&msg_without_sig, signature, server_mutex)
//         },
//         message::Msg::PrePrepareMsg(msg_without_sig) => check_pre_prepare(msg_without_sig, signature, server_mutex),
//         message::Msg::PrepareMsg(msg_without_sig) => check_prepare(msg_without_sig, signature, server_mutex),
//         message::Msg::CommitMsg(msg_without_sig) => check_commit(msg_without_sig, signature, server_mutex),
//         message::Msg::VcMsg(msg_without_sig) => todo!(),
//         message::Msg::RtMsg(msg_without_sig) => check_rt(msg_without_sig, signature, server_mutex),
//         message::Msg::ClientReplyMsg(msg_without_sig) => false,
//         message::Msg::NewViewMsg(msg_without_sig) => todo!(),
//         message::Msg::RtRplMsg(msg_without_sig) => todo!(),
//     }
// }


pub fn check_client_request(msg_without_sig: &message::ClientMsg, signature: &Vec<u8>, server: &mut MutexGuard<Server>) -> bool {
    let mut result = true;
    let client = msg_without_sig.who_send;
    // check wether in view change status
    if server.status == constants::DO_VIEW_CHANGE {
        log::info!("receive client msg, but not is in view-change status");
        return false
    }
    // check signature
    result = result && cryptography::verify_sig(&constants::get_client_pub(client).unwrap(), &bincode::serialize(&msg_without_sig).unwrap(), signature);

    // check tampstemp
    // client use timestamp to ditinguish the request
    if msg_without_sig.time_stamp < server.client_request[client].0 {
        result = false;
    }
    // if msg.timestamp > stored, means last request is chosen, do the next
    if msg_without_sig.time_stamp > server.client_request[client].0 {
        result = result && server.client_request[client].1 >= constants::PREPARED;
    }
    // client resend the msg
    if  msg_without_sig.time_stamp == server.client_request[client].0 {
        result = result && server.client_request[client].1 != constants::CAN_RECEND;
    }
    result = result && msg_without_sig.time_stamp > server.client_request[client].0 && server.client_request[client].1 >= constants::PREPARED;
    return result
}

pub fn check_pre_prepare(msg_without_sig: &message::PrePrepareMsg, signature: &Vec<u8>,server: &mut MutexGuard<Server>) -> bool {
    let mut result = true;
    // 1. check wether in view change status
    if server.status == constants::DO_VIEW_CHANGE {
        log::info!("receive client msg, but not is in view-change status");
        return false
    };
    // 2. check signature, for preprepare msg, we should check two signature, 1. client 2. the server sen
    // 2.1 check pre-prepare's client signature
    result = result && cryptography::verify_sig(&constants::get_client_pub(msg_without_sig.client_msg.who_send).unwrap(), 
                                                &bincode::serialize(&msg_without_sig.client_msg).unwrap(), 
                                                &msg_without_sig.client_msg_sig); 
    // 2.2 check server sender's signature
    result = result && cryptography::verify_sig(&constants::get_server_pub(msg_without_sig.who_send).unwrap(), 
                                            &bincode::serialize(&msg_without_sig).unwrap(), signature);
    // 3. check v
    result = result && server.my_view == msg_without_sig.v;
    // 4. check n
    result = result && msg_without_sig.n < server.get_h() + config::L as i32 && msg_without_sig.n >= server.get_h();
    // 5. if the log's slot is null or this pre-prepare msg is as same as precvious one
    let log_pointer = msg_without_sig.n - server.get_h();
    result = result && (server.log.get_seq(msg_without_sig.n).is_none() || server.log.get_seq(msg_without_sig.n).unwrap().entry_status == constants::LOG_ENTRY_INIT ||
                        server.log.get_seq(msg_without_sig.n).unwrap().client_msg_checksum == message::get_client_msg_sha256(&msg_without_sig.client_msg));
    result
}


pub fn check_prepare(msg_without_sig: &message::PrepareMsg, signature: &Vec<u8>, server: &mut MutexGuard<Server>) -> bool {
    let mut result = true;
    // 1. check wether in view change status
    if server.status == constants::DO_VIEW_CHANGE {
        log::info!("receive client msg, but not is in view-change status");
        return false
    };
    // 2 check server sender's signature
    result = result && cryptography::verify_sig(&constants::get_server_pub(msg_without_sig.who_send).unwrap(), 
                                            &bincode::serialize(&msg_without_sig).unwrap(), signature);
    // 3. check v
    result = result && server.my_view == msg_without_sig.v;
    // 4. check n
    result = result && msg_without_sig.n < server.get_h() + config::L as i32 && msg_without_sig.n >= server.get_h();
    // 5. check checksum if I have already receive pre-prepare msg, if I didnt receive it, just store the msg into
    // LogEntry.advanced_prepare
    let log_assigned = &server.log.get_seq(msg_without_sig.n);
    if log_assigned.is_some() && log_assigned.as_ref().unwrap().entry_status > constants::LOG_ENTRY_INIT as i32 {
        result = result && msg_without_sig.client_msg_checksum == log_assigned.as_ref().unwrap().client_msg_checksum;
    }
    // 6. check whether the sender has already vote for this n
    if log_assigned.is_some(){
        result = result && !log_assigned.as_ref().unwrap().cert_prepare_vote[msg_without_sig.who_send];
    }
    return result
}

pub fn check_commit(msg_without_sig: &message::CommitMsg, signature: &Vec<u8>, server: &mut MutexGuard<Server>) -> bool {
    let mut result = true;
    // 1. check wether in view change status
    if server.status == constants::DO_VIEW_CHANGE {
        log::info!("receive client msg, but not is in view-change status");
        return false
    };
    // 2 check server sender's signature
    result = result && cryptography::verify_sig(&constants::get_server_pub(msg_without_sig.who_send).unwrap(), 
                                            &bincode::serialize(&msg_without_sig).unwrap(), signature);
    // 3. check v
    result = result && server.my_view == msg_without_sig.v;
    // 4. check n
    result = result && msg_without_sig.n < server.get_h() + config::L as i32 && msg_without_sig.n >= server.get_h();


    // 5. check checksum if I have already receive pre-prepare msg, if I didnt receive it, just store the msg into
    // LogEntry.advanced_commit
    let log_assigned = &&server.log.get_seq(msg_without_sig.n);
    if log_assigned.is_some() && log_assigned.as_ref().unwrap().entry_status > constants::LOG_ENTRY_INIT {
        result = result && msg_without_sig.client_msg_checksum == log_assigned.as_ref().unwrap().client_msg_checksum;
        result = result && !log_assigned.as_ref().unwrap().cert_commit_vote[msg_without_sig.who_send];
    }

    // 6. check whether the sender has already vote for this n
    return result
}

//
pub fn check_rt(msg_without_sig: &message::RtMsg, signature: &Vec<u8>, server: &mut MutexGuard<Server>) -> bool {
    let mut result = true;
    // 1. check signature
    result = result && cryptography::verify_sig(&constants::get_server_pub(msg_without_sig.who_send).unwrap(), 
                                            &bincode::serialize(&msg_without_sig).unwrap(), signature);

    // check last retransmission msg, if it is not meet interval, ommit it
    let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis();
    let last_time = server.last_rcv[msg_without_sig.who_send];
    if now - last_time < config::RT_INTERV as u128 {
        result = false;
    }
    result
}

pub fn check_vc(msg_without_sig: &message::VcMsg, signature: Vec<u8>, server_mutex: &Arc<Mutex<Server>>) -> bool {
    todo!();
    let mut result = true;
    // 1. check signature
    // result = result && cryptography::verify_sig(&constants::get_server_pub(msg_without_sig.who_send).unwrap(), 
    //                                         &bincode::serialize(&msg_without_sig).unwrap(), signature);
    // {
    //     let server = server_mutex.lock().unwrap();
    //     result = result && server.new_view == msg_without_sig.v && !server.vc_vote[msg_without_sig.who_send];
    //     result = result && server.vc_num <= config::F_NUM * 2;
    //     result = result && msg_without_sig.prepared_set.len() <= config::L; // avoid too large malicious msg
    //     result = result &&  msg_without_sig.v == server.my_view ;
    //     // check wether all of prepared msg's v < msg.view
    //     for msg in msg_without_sig.prepared_set.iter() {
    //         // result = result &&  *ms
    //     }
    // }
    result
}

pub fn check_rt_rpl(msg_without_sig: &message::RtRplMsg, signature: &Vec<u8>, server: &mut MutexGuard<Server>) -> bool {
    let mut result = true;
    // 1. check signature
    result = result && cryptography::verify_sig(&constants::get_server_pub(msg_without_sig.who_send).unwrap(), 
                                            &bincode::serialize(&msg_without_sig).unwrap(), signature);

    // some check code in do_ function
    return result
}