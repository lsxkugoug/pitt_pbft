//! This file contains state management functions, and state transfer function
use crate::{network::message::*, consensus::model::{self, LogEntry, Server, StateEntry}, config, constants};
use std::{fs, sync::MutexGuard};
use std::cmp::max;

pub async fn read_states(state_seq: i32) -> Vec<StateEntry> {
    let file_name = constants::get_i_am().to_string() + "/" + &state_seq.to_string();
    let serialized: String = fs::read_to_string(file_name).expect("open state file failed").parse().expect("parse state file failed");
    let deserialized: Vec<StateEntry> = serde_json::from_str(&serialized).expect("deserize state failed");
    deserialized
}

pub async fn store_state(state_seq: i32, state: Vec<StateEntry>) {
    let file_name = constants::get_i_am().to_string() + "/" + &state_seq.to_string();
    let serialized = serde_json::to_string(&state).unwrap();
    fs::write(file_name, serialized).expect("Unable to write file");
}

// it only install the next state
pub fn install_state(state: Vec<StateEntry>, server: &mut MutexGuard<Server>) -> bool {

    if server.log.get_offset(0).is_some() &&
        server.applied == server.log.get_offset(0).unwrap().n + config::L as i32 {
        return false;
    }
    for state_entry in state {
        let seq = state_entry.n;
        server.log.set_seq(seq, state_entry.generate_log_entry_commited());
    }
    
    true
}