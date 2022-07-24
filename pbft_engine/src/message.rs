use std::sync::{Arc,Mutex};
use std::thread::panicking;
use p256::ecdsa::Signature;
use serde::{Deserialize, Serialize};
use serde_json::Result;
use crate::{cryptography, constants, server};

#[derive(Serialize, Deserialize, Clone)]
pub struct ClientMsg {
    pub msg_type: i32,
    pub who_send: usize,
    pub operation: String,
    pub time_stamp: String,
}


#[derive(Serialize, Deserialize, Clone)]
pub struct PrePrepareMsg {
    pub client_msg: ClientMsg,
    pub client_msg_sig: Vec<u8>, 
    pub who_send: usize,
    pub v: i32,
    pub n: i32,
}


#[derive(Serialize, Deserialize, Clone)]
pub struct PrepareMsg {
    pub client_msg: i32,

}

#[derive(Serialize, Deserialize, Clone)]
pub struct CommitMsg {
    pub client_msg: i32,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct VcMsg {
    pub who_send: usize,
}


#[derive(Serialize, Deserialize, Clone)]
pub enum Msg {
    ClientMsg(ClientMsg),
    PrePrepareMsg(PrePrepareMsg),
    PrepareMsg(PrepareMsg),
    CimmitMsg(CommitMsg),
    VcMsg(VcMsg),
}

#[derive(Serialize, Deserialize, Clone)]
pub struct  MsgWithSignature {
    pub msg_without_sig: Msg,
    pub signature: Vec<u8>,
}

// use my private key sign the msg, (note: sign objet, but not Msg enum) and generate Msg_with_signature object
pub fn sign_and_add_signature(msg_witout_signature: Msg) -> MsgWithSignature {
    let msg_bytes = match &msg_witout_signature {
        Msg::ClientMsg(msg_witout_signature) => bincode::serialize(msg_witout_signature).unwrap(),
        Msg::PrePrepareMsg(msg_witout_signature) => bincode::serialize(msg_witout_signature).unwrap(),
        Msg::PrepareMsg(msg_witout_signature) => bincode::serialize(msg_witout_signature).unwrap(),
        Msg::CimmitMsg(msg_witout_signature) => bincode::serialize(msg_witout_signature).unwrap(),
        Msg::VcMsg(msg_witout_signature) => bincode::serialize(msg_witout_signature).unwrap()
    };
    let signature = cryptography::sign_msg(&constants::get_my_prikey().unwrap(), &msg_bytes);
    MsgWithSignature{msg_without_sig: msg_witout_signature, signature: signature}
}



// later part is check functions related
pub fn check_msg(msg_with_sig: &MsgWithSignature, server_mutex: &Arc<Mutex<server::Server>>) ->bool {
    let signature = &msg_with_sig.signature;
    match &msg_with_sig.msg_without_sig {
        Msg::ClientMsg(msg_without_sig) => {
            let client = msg_without_sig.who_send;
            check_client_request(&msg_without_sig, client, signature, server_mutex)
        },
        Msg::PrePrepareMsg(msg_without_sig) => todo!(),
        Msg::PrepareMsg(msg_without_sig) => todo!(),
        Msg::CimmitMsg(msg_without_sig) => todo!(),
        Msg::VcMsg(msg_without_sig) => todo!()
    }
}


pub fn check_client_request(msg_without_sig: &ClientMsg, client:usize, signature: &[u8], server_mutex: &Arc<Mutex<server::Server>>) -> bool {
    let mut result = true;
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

