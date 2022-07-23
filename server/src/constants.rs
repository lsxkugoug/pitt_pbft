use p256::ecdsa::{VerifyingKey, SigningKey};
use std::fmt::format;
use std::fs;
use std::fs::File;
use std::io::Read;
use crate::{config, cryptography::{self, load_private_key}};

// server
static mut I_AM: Option<&mut usize> = None;

// message type
pub const CLIENT_REQUEST: i32 = 1; 
pub const PRE_PREPARE: i32 = 2; 
pub const PREPARE: i32 = 3; 
pub const COMMIT: i32 = 4; 
pub const VIEW_CHANGE: i32 = 5; 

// log status
pub const PRE_PREPARED: i32 = 1; 
pub const PREPARED: i32 = 2; 
pub const COMMITED: i32 = 3;
pub const APPLIED: i32 = 4;

// consensus
pub const TIMEOUT: u64 = 300;   // when timeout, go to view change

// public key, it initialized after program, little tricky
static mut CLIENT_PUB: Option<&mut Vec<VerifyingKey>> = None;
static mut SERVER_PUB: Option<&mut Vec<VerifyingKey>> = None;
static mut MY_PRIVATE_KEY: Option<&mut SigningKey> = None;


pub fn init_constants(i_am: usize) {
    // 1. init public key part constant
    // 1.1 load client public keys
    let client_pub = Box::new(Vec::new());
    let client_root_path = "./key_pairs/client";
    for i in 0..config::CLIENT_NUM {
        let client_path = format!("{}/{}/{}", client_root_path, i, "pub_key");
        client_pub.push(cryptography::load_public_key(client_path));
    }
    // 1.2 load server public keys
    let server_pub = Box::new(Vec::new());
    let server_root_path = "./key_pairs/server";
    for i in 0..config::CLIENT_NUM {
        let client_path = format!("{}/{}/{}", server_root_path, i, "pub_key");
        server_pub.push(cryptography::load_public_key(client_path));
    }
    // 1.3 load my private key
    let my_private_key = Box::new(load_private_key(format!("./key_pairs/server/{}/pri_key", i_am)));
    // 1. init server 
    let i_am = Box::new(i_am);
    unsafe {
        CLIENT_PUB = Some(Box::leak(client_pub));
        SERVER_PUB = Some(Box::leak(server_pub));
        MY_PRIVATE_KEY = Some(Box::leak(my_private_key));
        I_AM = Some(Box::leak(i_am));
    }
}