use std::collections::HashMap;
use std::fmt::format;

use std::sync::{Arc, Mutex, MutexGuard};
use std::{vec, string};
use crate::network::message::{MsgWithSignature, ClientMsg, ClientReplyMsg, VcMsg, sign_and_add_signature, broadcast_servers, Msg, NewViewMsg};
use crate::network::{message, check_vc};
use crate::cryptography::*;
use crate::{ consensus::model, consensus::view_change,constants, config};
use tokio::time::{sleep, Duration};


use super::Server;
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
    loop {
        let mut vc_msg = None;
        {   
            let mut server = server_mutex.lock().unwrap();   
            // 1. build VC msg
            vc_msg = Some(VcMsg {
                msg_type: constants::VIEW_CHANGE,
                v: server.my_view,
                who_send: constants::get_i_am(),
                last_stable_sq: server.get_h(),
                stable_certificates: server.rts.clone(),
                prepared_set: Vec::new(),
            });
            for offset in 0..config::L {
                if server.log.get_offset(offset as i32).is_some() && server.log.get_offset(offset as i32).unwrap().entry_status >= constants::PREPARED {
                    vc_msg.as_mut().unwrap().prepared_set.push((server.log.get_offset(offset as i32)).unwrap().generate_state());
                }
            }
            let sig = sign_msg(&constants::get_my_prikey().unwrap(), &bincode::serialize(&vc_msg).unwrap());
            server.vc_msgs[constants::get_i_am()] = Some((vc_msg.clone().unwrap(), sig));
        }


        let mut quorum_view = -1;
        // Liveness rule 1: wat vc msg whose view >= v + 1
        loop {
            let mut server = server_mutex.lock().unwrap();   
            vc_msg.as_mut().unwrap().v = server.my_view + 1;
            // liveness rule 2: participate view change when receive f + 1 vc msg.
            let mut live_2 = false;
            if server.vc_num > config::F_NUM + 1 {
                let mut tmp_views = Vec::new();
                for vc in &server.vc_msgs {
                    if vc.is_some() {
                        tmp_views.push(vc.as_ref().unwrap().0.v);
                    }
                }
                tmp_views.sort_by(|a, b| b.cmp(a));
                live_2 = true;
            }

            // open the lock
            drop(server);

            // if satisfy rule two, open the lock and participate in view change
            if live_2 {
                broadcast_servers(Msg::VcMsg(vc_msg.clone().unwrap())).await;
            }

            // wait other process collect vc msg
            sleep(Duration::from_millis(20)).await;

            // check wether can break
            let server = server_mutex.lock().unwrap();
            if server.vc_num > 2 * config::F_NUM + 1 {
                break;
            }
        }

        let mut server = server_mutex.lock().unwrap();
        // now get enough vc to calculate new_view
        let new_view = calculate_new_view(&mut server);

        // if I am leader, broadcast new view
        if constants::get_i_am() as i32 == new_view.v % config::SERVER_NUM as i32 {
            
            break;
        } else {
        // if I am normal node check the new view I calculate and the new leader send
        }
    }
}

fn leader_after_vc() {

}
fn calculate_new_view(server: &mut MutexGuard<Server>) -> NewViewMsg {

    let mut new_view = NewViewMsg {
        mst_type: constants::NEW_VIEW,
        v: server.my_view,
        vc_certificate: server.vc_msgs,
        state_set: vec![None; config::L],
    };
    let new_view_h = -1;
    for vc in server.vc_msgs {
        if vc.is_some() && vc.unwrap().0.last_stable_sq > new_view_h {
            new_view_h = vc.unwrap().0.last_stable_sq ;
        }
    }
    
    let new_view_map: HashMap<(i32, Vec<u8>), i32> = HashMap::new();
    for vc in server.vc_msgs {
        if vc.is_none() {
            continue;
        }
        for p in vc.unwrap().0.prepared_set {
            if p.n >= new_view_h && p.n <= new_view_h + config::L as i32 {
                let checksum = sha256(&bincode::serialize(&p).unwrap());
                if new_view_map.get(&(p.n, checksum)).is_none() {
                    new_view_map.insert((p.n, checksum), 0);
                }
                *new_view_map.get_mut(&(p.n, checksum)).unwrap() += 1;
                if *new_view_map.get(&(p.n, checksum)).unwrap() > config::F_NUM as i32 {
                    new_view.state_set[(p.n - new_view_h) as usize] = Some(p.clone());
                }
            }
        }
    };
    new_view
}


// when received viewchange msg
pub async fn do_vc(vc_msg: VcMsg, server_mutex: &Arc<Mutex<model::Server>>, msg_sig: &Vec<u8>){
    {
        let mut server = server_mutex.lock().unwrap();
        if !check_vc(&vc_msg, &mut server, msg_sig) {
            return;
        }

    }
 
    
}

// when received new-view msg
pub async fn do_newview(msg_without_sig: VcMsg, server_mutex: &Arc<Mutex<model::Server>>){

}