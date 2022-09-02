//! This file contains garbage collection feature. On paper p22

use crate::{network::message::*, consensus::model::{self, Log_entry}, config, constants};
use std::sync::{Arc,Mutex};
use std::cmp::max;
// todo no check_point msg now, we can use rt msg replaces it
// when receive check_point msg, do gc logic and 
// pub async fn do_check_point(check_point_msg: CheckPointMsg, server_mutex: Arc<Mutex<model::Server>>) {
//     let mut q_cp = -1;
//     let mut last_applied = -1;
//     let mut server_h = -1;

//     // 1. gc part
//     {
//         let mut server = server_mutex.lock().unwrap();
//         server.quorum_cp[check_point_msg.who_send] = max((check_point_msg.n / config::K as i32) * config::K as i32, server.quorum_cp[check_point_msg.who_send]);
//     }
//     gc_update_server(server_mutex).await;
    
//     // write logs to disk, log_entries_write
//     let check_num = log_entries_write.len() / config::K;
//     for i in 0..check_num {
//         // log_entries_write, log file name "my_No/check_num"

//     }

//     // todo if q_cp > my high water mark, state transfer
//     if q_cp > server_h + config::L as i32 {
//         todo!("start state transfer");
//     }
// }

// this function checks quorum's checkpoint, server's applied pointer,  slid winder & store logs into disk and update server'h
// There is only one situation1, which may update quorum_cp we should use this function:
// (1) when received RT msg. But applied function can change the quorum_cp
pub async fn gc_update_server(server_mutex: Arc<Mutex<model::Server>>) {
    let mut log_entries_write = Vec::new();
    {
        let mut server = server_mutex.lock().unwrap();
        let mut new_h = (server.h / config::K as i32) * config::K as i32; // find most recent check point
        let last_applied = server.applied;

        server.quorum_cp[constants::get_i_am()] = max(new_h, server.quorum_cp[constants::get_i_am()]);
        // 2.2 find the quorum's largest checkpoint
        let mut quorum_cp = server.quorum_cp.clone();
        quorum_cp.sort_by(|a, b| b.cmp(a));
        let q_cp = quorum_cp[(config::F_NUM * 2) as usize];
        // to avoid race condition
        if server.last_drop >= q_cp {
            return;
        }
        server.last_drop = q_cp;
        let mut server_h = server.h;
        // find logs which should be truncated
        while server_h + config::K as i32 <= q_cp && server_h + config::K as i32 <= last_applied{
            for _ in 0..config::K {
                // let server_h = server.h;
                log_entries_write.push(server.log.get(server_h as usize).clone());
                server_h += 1;
            }
            server.log.slid_window(); // todo we have to store the logs, after unlock, then write to the disk
        }
        server.h = server_h;
    }
    // write logs to disk, log_entries_write
    let cp_num = log_entries_write.len() / config::K;
    for i in 0..cp_num {
        let cp_num = log_entries_write[i].n / config::K as i32;
        let file_name = constants::get_i_am().to_string() + "/" + &cp_num.to_string();
        write_file(file_name, &log_entries_write[i.. i + config::K].to_vec()).await;
    }
}


async fn write_file(file_name: String, log_vec: &Vec<Log_entry>) {
    

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
        let mut tasks = Vec::new();
        for i in 0..5 {
            let i = i as i32;
            tasks.push(tokio::spawn(async move {
                write_file(i.to_string()).await; 
            }));
        };
        for t in tasks {
            t.await.unwrap(); 
        }
    }

   
}

