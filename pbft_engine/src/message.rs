use std::thread::panicking;
use p256::ecdsa::Signature;
use serde::{Deserialize, Serialize};
use serde_json::Result;
use crate::cryptography;
use crate::constants;

#[derive(Serialize, Deserialize)]
pub struct ClientMsg {
    pub msg_type: i32,
    pub who_send: usize,
    pub operation: String,
    pub time_stamp: String,
}

pub struct  ClientSignatureMsg {
    pub msg: ClientMsg,
    pub signature: Vec<u8>
}


#[derive(Serialize, Deserialize)]
pub struct PrePrepareMsg {
    pub client_msg: ClientMsg,
    pub who_send: usize,
    pub v: i32,
    pub n: i32,
    pub signature: Vec<u8>    
}


#[derive(Serialize, Deserialize)]
pub struct PrepareMsg {
    pub client_msg: i32,

}

#[derive(Serialize, Deserialize)]
pub struct CommitMsg {
    pub client_msg: i32,
}

pub struct VcMsg {
    
}

// // sign and wrap msg before send
// pub fn send_msg(&Str:msg) {
    
// }
#[derive(Serialize, Deserialize)]
pub enum Msg {
    ClientMsg(ClientMsg),
    PrePrepareMsg (PrePrepareMsg),
    CimmitMsg(CommitMsg)
}

#[derive(Serialize, Deserialize)]
pub struct  MsgWithSignature {
    pub msg_without_sig: Msg,
    pub signature: Vec<u8>    
}

// use my private key sign the msg and generate Msg_with_signature object
pub fn construct_msg_with_signature(msg_witout_signature: Msg) -> MsgWithSignature {
    let msg_bytes = bincode::serialize(&msg_witout_signature).unwrap();
    let signature = cryptography::sign_msg(&constants::get_my_prikey().unwrap(), &msg_bytes);
    MsgWithSignature{msg_without_sig: msg_witout_signature, signature: signature}
}

pub fn check_msg(msg_with_sig: &MsgWithSignature) ->bool {
    let signature = &msg_with_sig.signature;
    match &msg_with_sig.msg_without_sig {
        Msg::ClientMsg(msg_without_sig) => {
            let client = msg_without_sig.who_send;
            check_client_request(&msg_without_sig, client, signature)
        },
        Msg::PrePrepareMsg(msg_without_sig) => todo!(),
        Msg::CimmitMsg(msg_without_sig) => todo!(),
    }
}


// check function todo
fn check_client_request(msg_with_sig: &ClientMsg, client:usize, signature: &[u8]) -> bool {
    let mut result = true;
    // check signature
    result = result && cryptography::verify_sig(&constants::get_client_pub(client).unwrap(), &bincode::serialize(&msg_with_sig).unwrap(), signature);


    // check tampstemp

    // check signature


    return result
}

