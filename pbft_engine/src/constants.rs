use p256::ecdsa::{VerifyingKey, SigningKey};
use crate::{config, cryptography::{self, load_private_key}};


// server
pub static mut I_AM: usize = usize::max_value();

// message type
pub const CLIENT_REQUEST: i32 = 1; 
pub const PRE_PREPARE: i32 = 2; 
pub const PREPARE: i32 = 3; 
pub const COMMIT: i32 = 4; 
pub const VIEW_CHANGE: i32 = 5; 

// log status
pub const PRE_PREPARED: i32= 1; 
pub const PREPARED: i32 = 2; 
pub const COMMITED: i32 = 3;
pub const APPLIED: i32 = 4;

// consensus
pub const TIMEOUT: u64 = 300;   // when timeout, go to view change

// public key, it initialized after program, little tricky
pub static mut CLIENT_PUB: Vec<VerifyingKey> = Vec::new();
pub static mut SERVER_PUB: Vec<VerifyingKey> = Vec::new();
pub static mut MY_PRIVATE_KEY: Option<SigningKey> = None;

// this function is used to init server constants. Scalable to future change.
pub unsafe fn init_constants(i_am: usize) {
    // 1. init server variable
    I_AM = i_am;
    // 2. init key pairs
    // 2.1 init client public key
    let client_root_path = "./key_pairs/client";
    for i in 0..config::CLIENT_NUM {
        let client_path = format!("{}/{}/{}", client_root_path, i, "pub_key");
        CLIENT_PUB.push(cryptography::load_public_key(client_path));
    }
    // 2.2 init server public key 
    let server_root_path = "./key_pairs/server";
    for i in 0..config::CLIENT_NUM {
        let client_path = format!("{}/{}/{}", server_root_path, i, "pub_key");
        SERVER_PUB.push(cryptography::load_public_key(client_path));
    }
    // 2.3 init my private key
    MY_PRIVATE_KEY = Some(load_private_key(format!("./key_pairs/server/{}/pri_key", i_am)));
}

pub fn get_i_am() -> usize {
    let mut ret = usize::max_value();
    unsafe{
        ret = I_AM;
    }
    return ret
}


pub fn get_client_pub(client: usize) -> Option<VerifyingKey>{
    let mut ret: Option<VerifyingKey> = None;
    unsafe {
        ret = Some(CLIENT_PUB[client]);
    }
    return ret
}
pub fn get_server_pub(server: usize) -> Option<VerifyingKey>{
    let mut ret: Option<VerifyingKey> = None;
    unsafe {
        ret = Some(SERVER_PUB[server]);
    }
    return ret
}
pub fn get_my_prikey() -> Option<SigningKey> {
    let mut ret: Option<SigningKey> = None;
    unsafe {
        ret = Some(load_private_key(format!("./key_pairs/server/{}/pri_key", get_i_am())))
    }
    ret
}

#[cfg(test)]
mod tests {
    #[test]
    fn initfun_test() {
        unsafe{
            super::init_constants(0);
        }

    }
}