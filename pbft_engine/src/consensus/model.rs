//! Thie file contains model which used to support consensus alrithm. 
//! struct Server contains all vriables recording the consensus process.
use crate::{config, network::message::{self, VcMsg, RtMsg, NewViewMsg, ClientMsg}, constants};
use std::{time::{SystemTime, UNIX_EPOCH}, vec, collections::HashMap};
use log::Log;
use serde::{Deserialize, Serialize};

use serde_json::{json, Map};



#[derive(Debug, Clone)]
#[derive(Serialize, Deserialize)]
pub struct StateEntry {
    pub client_msg: Option<message::ClientMsg>,     // Initialy all slot's client msg == None
    pub v: i32,     // view number
    pub n: i32,     // sequence number
    pub client: usize,// client's number
}
impl StateEntry {
    pub fn generate_log_entry_commited(&self) ->LogEntry {
        let log_entry = LogEntry::default();
        log_entry.client_msg = self.client_msg.clone();
        log_entry.v = self.v;
        log_entry.n = self.n;
        log_entry.client = self.client;
        log_entry.entry_status = constants::COMMITED;
        log_entry
    }
    pub fn generate_log_entry_pp(&self) ->LogEntry {
        let log_entry = LogEntry::default();
        log_entry.client_msg = self.client_msg.clone();
        log_entry.v = self.v;
        log_entry.n = self.n;
        log_entry.client = self.client;
        log_entry.entry_status = constants::PRE_PREPARED;
        log_entry
    }
}
// note:Rust does not implement Default for all arrays
#[derive(Debug, Clone)]
#[derive(Serialize, Deserialize)]
pub struct LogEntry {
    pub client_msg: Option<message::ClientMsg>,     // Initialy all slot's client msg == None
    pub client_msg_checksum: Vec<u8>,
    pub entry_status: i32,  // PRE-PREPARED, PREPARED, COMMITED, APPLIED, etc...
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

impl Default for LogEntry {
    fn default() -> Self {
        Self {client_msg: None, 
            client_msg_checksum: Default::default(),  
            entry_status: constants::LOG_ENTRY_INIT, 
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

impl LogEntry {
    pub fn generate_state(&self) ->StateEntry {
        StateEntry {
            client_msg: self.client_msg.clone(),
            v: self.v,
            n: self.n,
            client: self.client,
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
    pub timeout: u64,
    pub vc_msgs: Vec<Option<VcMsg>>,
    pub rts: Vec<Option<(RtMsg, Vec<u8>)>>,                       // with sig used to verify largest stable checkpoint in VC msg
    pub new_view: Option<NewViewMsg>,
    
    // check point management
    pub quorum_applied: Vec<i32>,                             // record everyone's last applied seq number, when receive <Rt> msg, find the largest quorum checkpoint, and truncate logs, note, this may not stable checkpoint
    pub last_drop: i32,                                     // constains the sequence number which already be stored in disk. To avoid race condition
    
    // retransimission and state transfer
    pub restore_state: HashMap<i32, HashMap<Vec<u8>, (i32, Vec<StateEntry>)>>,    // key = state_seq, nested map<digest, (count,byte[])>
    pub restore_state_vote: HashMap<i32, Vec<bool>>,                      // restore_state_vote map<seq, list[bool] length:server_num>
    pub restore_pp:  HashMap<i32, HashMap<Vec<u8>, (i32, StateEntry)>>,    // key = seq, nested map<digest, (count,Log_Entry)>
    pub restore_pp_vote: HashMap<i32, Vec<bool>>,        // restore_state_vote map<seq, list[bool] length:server_num>
    pub last_new_view: NewViewMsg,               // with sig
    pub last_vc: VcMsg,                          // with sig
    
    // todo not used right now
    pub last_rcv: Vec<u128>,                             // for msg retransmission, record the last time I receive nodes' status
}
impl Default for Server {
    fn default() -> Self {
        Self {
            status: 0,
            client_request: vec![("0".to_string(), constants::PREPARED); config::CLIENT_NUM] ,
            my_view: 0,
            applied: -1,
            who_leader: 0,
            log: CirLog::new(config::L),
            log_assign: 0,   
            timeout: config::TIMEOUT,
            quorum_applied: vec![-1; config::SERVER_NUM],
            last_rcv: vec![0; config::SERVER_NUM],
            last_drop: -1,
            vc_msgs: vec![None; config::SERVER_NUM],
            rts: vec![None; config::SERVER_NUM],
            new_view: None,
            restore_state: HashMap::new(),
            restore_state_vote: HashMap::new(),
            restore_pp: HashMap::new(),
            restore_pp_vote: HashMap::new(),
            last_new_view: NewViewMsg{ mst_type: constants::NEW_VIEW, v: 0, vc_certificate: Vec::<(VcMsg, Vec<u8>)>::new()},
            last_vc: VcMsg{ msg_type: constants::VIEW_CHANGE, v: 0, who_send: constants::get_i_am(), last_stable_sq: -1, stable_certificates: Vec::<(RtMsg, Vec<u8>)>::new(), prepared_set: Vec::<LogEntry>::new()},
            }
    }

}
impl Server {
    // return low water mark sequence number
    pub fn get_h(&mut self) -> i32 {
        let head = self.log.get_offset(0);
        match head {
            Some(head) => head.n,
            None => (self.applied + 1) / config::K as i32 * config::K as i32, // if None == none, means window shift or initial state
        }
    }
    pub fn am_leader(&mut self) -> bool {
        self.my_view % config::SERVER_NUM as i32 == 0
    }

}
// circular queue implementation
// only for log
// when slid the window, the caller also should move server.h
pub struct CirLog
{
    capacity: usize,
    data: Vec<Option<LogEntry>>,
    head: usize,    // pointer to the start
    // tail: usize,
}

// log manager
impl CirLog{
    pub fn new(capacity: usize) -> CirLog {
        CirLog { capacity:capacity, data: vec![Default::default(); capacity], head: 0 ,}
    }
    
    // get from head + offset
    pub fn get_offset(&mut self, offset: i32)-> Option<LogEntry>  {
        return self.data[(self.head + offset as usize) % self.capacity].clone();
    }

    // get from specific sequence number
    pub fn get_seq(&mut self, seq: i32) -> Option<LogEntry> {
        if self.data[self.head].is_none() {
            None
        }else{
            let first = self.data[self.head].as_ref().unwrap().n;
            if seq < first || seq > first + config::L as i32 {
                None
            } else {
                self.get_offset(seq - first)
            }
        }
    }


    pub fn set_offset(&mut self, offset: i32, log_entry: LogEntry) -> bool {
        if offset as usize > config::L {
            false
        } else {
            self.data[(self.head + offset as usize ) % self.capacity] = Some(log_entry.clone());
            true
        }
    }

    pub fn set_seq(&mut self, seq: i32, log_entry: LogEntry) -> bool {
        if self.data[self.head].is_none() {
            return false;
        }else{
            let first = self.data[self.head].as_ref().unwrap().n;
            if seq < first || seq > first + config::L as i32 {
                false
            } else {
                self.set_offset(seq - first, log_entry)
            }
        }
    }

    fn remove(&mut self, offset: usize) {
        for i in 0..offset {
            self.data[i] = None;
        }
        self.head = (self.head + offset) % self.capacity
    }

    // pay attention! it doesn't store the state, since we need decrease the time of lock, use store
    // state to store the slided window log_entry.
    pub fn slid_window(&mut self) {
        self.remove(config::K);
    }

}



