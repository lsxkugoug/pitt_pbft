//! this file contains retransmission feature

use std::{sync::{Arc,Mutex}, cmp::max};
use model::Server;
use tokio::time::{sleep, Duration};
use crate::{network::message, consensus::model, constants, config, cryptography, checkpoint_gc};
use std::time::{SystemTime, UNIX_EPOCH};

// This function periodically send nodes status 
// msg format  <STATUS-ACTIVE, v, h, le, i, log_status> todo, for this time, only normal situation
pub async  fn period_rt(server_mutex: Arc<Mutex<model::Server>>) {
    loop {
        let mut node_status = constants::NORMAL;
        let mut v = -1;
        let mut h = 0;
        let mut le = -2;
        let who_send = constants::get_i_am();
        let mut log_status = Vec::new();
        {
            let mut server = server_mutex.lock().unwrap();
            node_status = server.status;
            v = server.my_view;
            for i in 0..config::L{
                let log_entry = server.log.get(i);
                if le == -2 && log_entry.entry_status != constants::APPLIED {
                    le = log_entry.n;
                }
                if le != -2 {
                    log_status.push(log_entry.entry_status);
                }
            }
            h = server.h;
        }
        // broadcast to all servers
        let rt_msg = message::RtMsg {
            node_status: node_status,
            v: v,
            h: h,
            le: le,
            who_send: who_send,
            log_status: log_status,
        };
        message::broadcast_servers(message::Msg::RtMsg(rt_msg)).await;
        
        sleep(Duration::from_millis(config::RT_RCV as u64)).await;
    }

}

// when receive retransmission msg, check the status and add it into my vote, only restore prepared, commited 
// <STATUS-ACTIVE, v, h, le, i, log_status>
pub async fn do_rt(rt_msg: message::RtMsg,  server_mutex: Arc<Mutex<model::Server>>) {
    // 1. update quorum cp
    // 1. gc part
    let mut ommit = false; // if server is now view change, only process gc
    {
        let mut server = server_mutex.lock().unwrap();
        server.quorum_cp[rt_msg.who_send] = max((rt_msg.le / config::K as i32) * config::K as i32, server.quorum_cp[rt_msg.who_send]);
        if server.status == constants::VIEW_CHANGE {
            ommit = true;
        }    
    }
    checkpoint_gc::gc::gc_update_server(server_mutex).await;
    
    if ommit {
        return;
    }
    if rt_msg.node_status == constants::NORMAL {

    } 
    if rt_msg.node_status == constants::VIEW_CHANGE {

    }

}