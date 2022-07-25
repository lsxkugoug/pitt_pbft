//! Thie file contains model which used to support consensus alrithm. 
//! struct Server contains all vriables recording the consensus process.
use crate::{config, network::message};

// note:Rust does not implement Default for all arrays
#[derive(Debug, Clone)]
pub struct Log_entry {
    pub client_msg: Option<message::ClientMsg>,
    pub client_msg_checksum: Vec<u8>,
    pub log_type: i32,  // PRE-PREPARE, PREPARE, COMMIT, APPLIED, etc...
    pub v: i32,     // view number
    pub n: i32,     // sequence number
    pub client: usize,// client's number
    pub who_send: usize ,// who send this message

	pub cert_prepare_num: i32,
	pub cert_prepare_vote: Vec<bool>,

    pub cert_commit_num: i32,
	pub cert_commit_vote: Vec<bool>,
}

impl Default for Log_entry {
    fn default() -> Self {
        Self {client_msg: None, client_msg_checksum: Default::default(),  log_type: Default::default(), v: Default::default(), n: -1, client: Default::default(), who_send: Default::default(), cert_prepare_num: Default::default(), cert_prepare_vote: vec![false; config::SERVER_NUM], cert_commit_num: Default::default(), cert_commit_vote: vec![false; config::SERVER_NUM]}
    }
}


// #[derive(Debug)]
// the constants veriable would stored in constants
pub struct Server {
    // normal variables
	// pub I_am: i32,                                     // Identification of server
    pub status: i32,                                    // normal, view-change, if view-change, only receive vc related msg                                     
	pub client_request:  Vec<(String, i32)>,            // (timestemp, status) used to maintain one semantic
    pub my_view: i32,
    pub applied: i32,
    pub who_leader: i32,
    pub log: Vec<Log_entry>,

    // leader variable
    pub log_assign: i32,                               // the pointer point the next slot of log should be assgined

    // view change variable
    
    // change point management
    pub h: i32,                                         // current sequence number of log[0]
    
}

impl Default for Server {
    fn default() -> Self {
        Self {
            status: 0,
            client_request: vec![("0".to_string(), 0); config::CLIENT_NUM] ,
            my_view: 0,
            applied: -1,
            who_leader: 0,
            log: vec![Default::default(); config::L],
            log_assign: 0,      
            h: 0
            }
    }
}