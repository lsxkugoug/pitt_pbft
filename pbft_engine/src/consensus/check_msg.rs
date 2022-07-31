//! This file constains functions which check msg before process it. It never change the server struct, just do check!
//! When servers receive any msg, it should check them. If msg pass the corresponding checking, go to process functions.

use std::sync::{Arc,Mutex};

use crate::{network::message, consensus::model, constants, config, cryptography};

pub fn check_msg(msg_with_sig: &message::MsgWithSignature, server_mutex: &Arc<Mutex<model::Server>>) ->bool {
    let signature = &msg_with_sig.signature;
    match &msg_with_sig.msg_without_sig {
        message::Msg::ClientMsg(msg_without_sig) => {
            check_client_request(&msg_without_sig, signature, server_mutex)
        },
        message::Msg::PrePrepareMsg(msg_without_sig) => check_pre_prepare(msg_without_sig, signature, server_mutex),
        message::Msg::PrepareMsg(msg_without_sig) => check_prepare(msg_without_sig, signature, server_mutex),
        message::Msg::CommitMsg(msg_without_sig) => check_commit(msg_without_sig, signature, server_mutex),
        message::Msg::VcMsg(msg_without_sig) => todo!()
    }
}


pub fn check_client_request(msg_without_sig: &message::ClientMsg, signature: &[u8], server_mutex: &Arc<Mutex<model::Server>>) -> bool {
    let mut result = true;
    let client = msg_without_sig.who_send;
    // check wether in view change status
    let server = server_mutex.lock().unwrap();
    if server.status == constants::DO_VIEW_CHANGE {
        log::info!("receive client msg, but not is in view-change status");
        return false
    }
    // check signature
    result = result && cryptography::verify_sig(&constants::get_client_pub(client).unwrap(), &bincode::serialize(&msg_without_sig).unwrap(), signature);

    // check tampstemp
    // only 1. the msg's time_stamp > the client's last request timestamp, to maintain one semantics  2.  the client's last request complete
    result = result && msg_without_sig.time_stamp > server.client_request[client].0 && server.client_request[client].1 > constants::PREPARED;
    return result
}

pub fn check_pre_prepare(msg_without_sig: &message::PrePrepareMsg, signature: &[u8], server_mutex: &Arc<Mutex<model::Server>>) -> bool {
    let mut result = true;
    // 1. check wether in view change status
    let server = server_mutex.lock().unwrap();
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
    result = result && msg_without_sig.n < server.h + config::L as i32 && msg_without_sig.n >= server.h;
    // 5. if the log's slot is null or this pre-prepare msg is as same as precvious one
    let log_pointer = msg_without_sig.n - server.h;
    result = result && (server.log[log_pointer as usize].client_msg.is_none() || 
                        server.log[log_pointer as usize].client_msg_checksum == message::get_client_msg_sha256(&msg_without_sig.client_msg));
    result
}


pub fn check_prepare(msg_without_sig: &message::PrepareMsg, signature: &[u8], server_mutex: &Arc<Mutex<model::Server>>) -> bool {
    let mut result = true;
    // 1. check wether in view change status
    let server = server_mutex.lock().unwrap();
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
    result = result && msg_without_sig.n < server.h + config::L as i32 && msg_without_sig.n >= server.h;
    // 5. check checksum if I have already receive pre-prepare msg, if I didnt receive it, just store the msg into
    // log_entry.advanced_prepare
    let log_assigned = &server.log[(msg_without_sig.n - server.h) as usize];
    if !log_assigned.client_msg.is_none() {
        result = result && msg_without_sig.client_msg_checksum == log_assigned.client_msg_checksum;
    }
    // 6. check whether the sender has already vote for this n
    result = result && !log_assigned.cert_prepare_vote[msg_without_sig.who_send];
    return result
}

pub fn check_commit(msg_without_sig: &message::CommitMsg, signature: &[u8], server_mutex: &Arc<Mutex<model::Server>>) -> bool {
    let mut result = true;
    // 1. check wether in view change status
    let server = server_mutex.lock().unwrap();
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
    result = result && msg_without_sig.n < server.h + config::L as i32 && msg_without_sig.n >= server.h;


    // 5. check checksum if I have already receive pre-prepare msg, if I didnt receive it, just store the msg into
    // log_entry.advanced_prepare
    let log_assigned = &server.log[(msg_without_sig.n - server.h) as usize];
    if !log_assigned.client_msg.is_none() {
        result = result && msg_without_sig.client_msg_checksum == log_assigned.client_msg_checksum;
    }

    // 6. check whether the sender has already vote for this n
    result = result && !log_assigned.cert_commit_vote[msg_without_sig.who_send];
    return result
}
