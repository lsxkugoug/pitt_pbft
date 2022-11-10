//! This file contains garbage collection feature. On paper p22
use crate::{network::message::*, consensus::model::{self, LogEntry}, config, constants};
use std::{sync::{Arc,Mutex}, collections::HashMap};
use std::cmp::max;

use super::checkpoint::store_state;

// this function checks quorum's checkpoint, server's applied pointer,  slid winder & store logs into disk and update server'h
// There is only one situation1, which may update quorum_cp we should use this function:
// (1) when received RT msg. But applied function can change the quorum_cp
pub async fn try_gc(server_mutex: Arc<Mutex<model::Server>>) {
    let mut state_write = HashMap::new();
    {
        let mut server = server_mutex.lock().unwrap();
        server.quorum_applied[constants::get_i_am()] = max(server.applied, server.quorum_applied[constants::get_i_am()]);
        // 2.2 find the quorum's largest checkpoint
        let mut tmp_apl = server.quorum_applied.clone();
        tmp_apl.sort_by(|a, b| b.cmp(a));
        let quorum_cp = tmp_apl[(config::F_NUM * 2) as usize] / config::K as i32 * config::K as i32; 
        // to avoid race condition
        if server.last_drop >= quorum_cp {
            return;
        }
        // find logs which should be truncated

        while server.get_h() + config::K as i32 <= quorum_cp && server.get_h() + config::K as i32 <= server.applied{
            let state_seq = server.get_h() / config::K as i32;
            let mut one_K = Vec::new();
            let h = server.get_h(); // to avoid annoying rust complier....
            for i in 0..config::K {
                one_K.push(server.log.get_seq(h + i as i32).unwrap().generate_state().clone());
            }
            // log_entries_write.insert(, server.log.get(server_h as usize).clone());
            state_write.insert(state_seq ,one_K.clone());
            server.log.slid_window();  // clean the relative log slot 
        }
    }
    // write logs to disk, log_entries_write
    for (state_seq, state) in  state_write {
        store_state(state_seq, state);
    }

}


#[cfg(test)]
mod test{
    use std::{thread::{sleep_ms, sleep}, fs, vec};

    use futures::prelude::*;
    use serde_json::json;
    use tokio::net::TcpStream;
    use tokio_serde::formats::*;
    use tokio_util::codec::{FramedWrite, LengthDelimitedCodec};

    async fn write_file(i: String) {
        fs::write("./aaa", i).expect("Unable to write file");
    }
    // good way to use tokio process multiple 
    #[tokio::main]
    #[test]
    async fn test_write_file() {
        let mut tasks:Vec<i32> = vec![1, 2, 5,3, 4, 5];
        let filename = "./test/test1";
        let serialized = serde_json::to_string(&tasks).unwrap();
        fs::write(filename, serialized).expect("Unable to write file");

    }

    #[tokio::main]
    #[test]
    async fn test_read_from_file() {
        let serialized: String = fs::read_to_string("./aaa").expect("msg").parse().expect("msg");
        let deserialized: Vec<i32> = serde_json::from_str(&serialized).unwrap();
        println!("{:?}",deserialized);
    }   

   
}

