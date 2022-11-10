use std::fmt::format;

use std::sync::{Arc, Mutex};
use std::{vec, string};
use crate::network::message::{MsgWithSignature, ClientMsg, ClientReplyMsg, VcMsg, sign_and_add_signature};
use crate::network::{message};
use crate::cryptography;
use crate::{ consensus::model, consensus::view_change,constants, config};
use tokio::time::{sleep, Duration};
use std::collections::HashMap;

// view-change msg: <VIEW-CHANGE, v + 1, last_scp, s, C, P, i> why？<VIEW-CHANGE, v + 1, P, i> I think?
//  last_scp: sequence number of last stable checkpoin
//  s: operation?
//  C is the stable certificate for last_scp? why
//  P is a set with a prepared certificate for each request that prepared at with a sequence number greater than 
pub async fn start_view_change(server_mutex: &Arc<Mutex<model::Server>>){
    // // start view change until elect a new leader
    // let mut cur_vc_timeout = config::VC_TIMEOUT;
    // // 1. we should get 2f + 1 valid v + 1 before double TIMEOUT
    
    // loop{
    //     let mut VcMsg = None;
    //     {
    //         let mut server = server_mutex.lock().unwrap();
    //         // 1. reset related variables
    //         server.new_view += 1;
    //         server.cur_prepared_set = Vec::new();
    //         server.vc_num = 1;
    //         server.vc_vote = vec![false; config::SERVER_NUM];
    //         server.vc_vote[constants::get_i_am()] = true;
    //         server.vc_num = 1;
    //         server.vc_vote[constants::get_i_am()] = true;
    //         // 2. put my prepared log into it firstly
    //         let mut prepared_set = Vec::new();
    //         for i in 0..config::K {
    //             let log_entry = server.log.get(i);
    //             if !log_entry.client_msg.is_none() && log_entry.entry_status >= constants::PREPARED {
    //                 prepared_set.push((log_entry.n, log_entry.client_msg.as_ref().unwrap().clone()));
    //             }
    //         }
    //         VcMsg = Some(VcMsg {
    //             msg_type: constants::VIEW_CHANGE,
    //             v: server.new_view,
    //             prepared_set: prepared_set,
    //             who_send: constants::get_i_am(),
    //         });
    //     }

    //     message::broadcast_servers(message::Msg::VcMsg(VcMsg.unwrap())).await;
        
    //     sleep(Duration::from_millis(cur_vc_timeout)).await;
    //     // judge wether view_change finished
    //     {
    //         let mut server = server_mutex.lock().unwrap();
    //         // if server completed view change, quit from this loop
    //         if server.status == constants::NORMAL{
                
    //             break;
    //         }
    //     }
    //     cur_vc_timeout *= 2;
    // }
}

// when received viewchange msg, every time
pub async fn do_vc(msg_without_sig: VcMsg, server_mutex: &Arc<Mutex<model::Server>>){
    // let mut can_process = false;
    // let mut cur_view = -1;
    // let mut be_processed = Vec::new();
    // {
    //     let mut server = server_mutex.lock().unwrap();
    //     if msg_without_sig.v != server.new_view {
    //         return;
    //     }
    //     server.vc_num += 1;
    //     server.vc_vote[msg_without_sig.who_send] = true;
    //     server.cur_prepared_set.extend(msg_without_sig.prepared_set);
    //     if server.vc_num >= config::F_NUM * 2 + 1 {
    //         can_process = true;
    //         be_processed = server.cur_prepared_set.clone();
    //     }
    //     cur_view = server.new_view;
    // }
    // if !can_process {
    //     return;
    // }
    // let mut compelted_prepare_set:Vec<(i32, ClientMsg)> = Vec::new();
    // let mut process_map: HashMap<(i32,Vec<u8>), (i32, ClientMsg)> = HashMap::new(); // HashMap<(n, digest), number>

    // for pp_entry in be_processed {
    //     let key1 = pp_entry.0;
    //     let key2 = cryptography::sha256(&bincode::serialize(&pp_entry.1).unwrap());
    //     let count = process_map.entry((key1, key2)).or_insert((0, pp_entry.1.clone()));
    //     count.0 += 1;
    // }

    // for (k, v) in process_map.iter() {
    //     let count = v.0;
    //     let n = k.0;
    //     if count  > (config::F_NUM + 1) as i32 {
    //         compelted_prepare_set.push((k.0, v.1.clone()));
    //     }
    // }
    // // sort by sequence number
    // compelted_prepare_set.sort_by(|a, b| a.0.cmp(&b.0));

    // // if I am current leader, send new_view to others
    // if cur_view % config::SERVER_NUM as i32 == 0 {
    //     todo!()
    // }
    
}

pub async fn do_newview(msg_without_sig: VcMsg, server_mutex: &Arc<Mutex<model::Server>>){

}