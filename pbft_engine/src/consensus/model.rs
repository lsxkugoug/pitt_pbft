//! Thie file contains model which used to support consensus alrithm. 
//! struct Server contains all vriables recording the consensus process.
use crate::{config, network::message};
use std::{time::{SystemTime, UNIX_EPOCH}, vec};
// note:Rust does not implement Default for all arrays
#[derive(Debug, Clone)]
pub struct Log_entry {
    pub client_msg: Option<message::ClientMsg>,     // if client_msg == None, means this slot is empty.
    pub client_msg_checksum: Vec<u8>,
    pub entry_status: i32,  // PRE-PREPARE, PREPARE, COMMIT, APPLIED, etc...
    pub v: i32,     // view number
    pub n: i32,     // sequence number
    pub client: usize,// client's number
    pub who_send: usize ,// who send this message

	pub cert_prepare_num: i32,
	pub cert_prepare_vote: Vec<bool>,

    pub cert_commit_num: i32,
	pub cert_commit_vote: Vec<bool>,

    // This two variable is try to solve the problem receive prepare or commit msg before receive pre-prepare msg from leader
    pub advanced_prepare:Vec<Vec<u8>>,
    pub advanced_commit:Vec<Vec<u8>>,

}

impl Default for Log_entry {
    fn default() -> Self {
        Self {client_msg: None, 
            client_msg_checksum: Default::default(),  
            entry_status: Default::default(), 
            v: Default::default(), 
            n: -1, client: Default::default(), 
            who_send: Default::default(), 
            cert_prepare_num: Default::default(), 
            cert_prepare_vote: vec![false; config::SERVER_NUM], 
            cert_commit_num: Default::default(), 
            cert_commit_vote: vec![false; config::SERVER_NUM],
            advanced_prepare: vec![vec![]; config::SERVER_NUM],
            advanced_commit: vec![vec![]; config::SERVER_NUM],
        }
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
    pub applied: i32,                                   // applied sequence number
    pub who_leader: i32,
    pub log: CirLog,

    // leader variable
    pub log_assign: i32,                               // the pointer point the next slot of log should be assgined

    // view change variable
    
    // check point management
    pub h: i32,                                         // current sequence number of log[0]
    pub quorum_cp: Vec<i32>,                             // when receive <CHECK-POINT> msg, find the largest quorum checkpoint, and truncate logs, note, this may not stable checkpoint

    
    // retransimission and gc
    pub last_rcv: Vec<u128>,                             // for msg retransmission, record the last time I receive nodes' status
    pub last_drop: i32,                                  // constains the sequence number which already be stored in disk. To avoid race condition
}

impl Default for Server {
    fn default() -> Self {
        Self {
            status: 0,
            client_request: vec![("0".to_string(), 0); config::CLIENT_NUM] ,
            my_view: 0,
            applied: -1,
            who_leader: 0,
            log: CirLog::new(config::L),
            log_assign: 0,      
            h: 0,
            quorum_cp: vec![0; config::SERVER_NUM],
            last_rcv: vec![0; config::SERVER_NUM],
            last_drop: -1,
            }
    }
}


// circular queue implementation
// only for log
// when slid the window, the caller also should move server.h
pub struct CirLog
{
    capacity: usize,
    data: Vec<Log_entry>,
    head: usize,    // pointer to the start
    // tail: usize,
}

impl CirLog{
    pub fn new(capacity: usize) -> CirLog {
        CirLog { capacity:capacity, data: vec![Default::default(); capacity], head: 0 ,}
    }
    
    pub fn get(&mut self, index: usize) -> &mut Log_entry {
        return &mut self.data[(self.head + index) % self.capacity];
    }

    pub fn set(&mut self, index: usize, log_entry: Log_entry) {
        self.data[(self.head + index) % self.capacity] = log_entry;
    }

    fn remove(&mut self, slot_num: usize) {
        for i in 0..slot_num {
            self.get(self.head + i).client_msg = None;
        }
        self.head = (self.head + slot_num) % self.capacity
    }

    pub fn slid_window(&mut self) {
        self.remove(config::K);
        //  store the state into disk, use gc write disk, cant lock mutex
    }

}



