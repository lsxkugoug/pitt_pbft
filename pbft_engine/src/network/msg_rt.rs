//! this file contains retransmission feature

use std::{sync::{Arc,Mutex}, cmp::max, collections::HashMap, hash::Hash};
use log::Log;
use model::{Server, LogEntry};
use tokio::time::{sleep, Duration};
use crate::{network::*, consensus::*, constants, config, cryptography::{self, sha256}, checkpoint_gc::{self, gc::try_gc, checkpoint::{read_states, install_state}}};
use std::time::{SystemTime, UNIX_EPOCH};

// This function periodically send rt msg, like a heartbeats
pub async  fn period_rt(server_mutex: Arc<Mutex<model::Server>>) {
    loop {
        let mut log_entry_status_set: Vec<(i32, i32, Vec<u8>)> = Vec::new();  // seq, status, digest
        let mut rt_msg = None;
        {
            let mut server = server_mutex.lock().unwrap();
            let mut idx = server.applied;
            while idx < config::L as i32 * config::L as i32 {
                let log_entry = server.log.get_seq(idx);
                if log_entry.is_none() {
                    server.log.set_seq(idx, LogEntry::default());
                }
                let log_entry = log_entry.unwrap();
                log_entry_status_set.push((log_entry.n, log_entry.entry_status, log_entry.client_msg_checksum.clone()));
                idx += 1;
            }
                    // broadcast to all servers
            rt_msg = Some(RtMsg {
                node_status: server.status,
                who_send: constants::get_i_am(),
                last_applied_seq: server.applied,
                log_entry_status_set: log_entry_status_set,
                low_water: server.get_h(),
            });
            // update server.rts which is used by view change
            if server.rts[constants::get_i_am()].is_some() && 
                rt_msg.as_ref().unwrap().last_applied_seq > server.rts[constants::get_i_am()].as_ref().unwrap().0.last_applied_seq {
                    server.rts[constants::get_i_am()] = Some((
                        rt_msg.as_ref().unwrap().clone(),
                        cryptography::sign_msg(&constants::get_my_prikey().unwrap(),&bincode::serialize(rt_msg.as_ref().unwrap()).unwrap()),
                    ));
                }
        }

        // update server.rts
    
        broadcast_servers(Msg::RtMsg(rt_msg.unwrap())).await;
        
        sleep(Duration::from_millis(config::RT_RCV as u64)).await;
    }

}

// when receive retransmission msg, check the status and add it into my vote, only restore prepared, commited 
pub async fn do_rt(rt_msg: RtMsg,  server_mutex: Arc<Mutex<model::Server>>, msg_sig: &Vec<u8>) {

    let mut is_lag = false;
    let mut rt_rpl = RtRplMsg {
        who_send: constants::get_i_am(),
        restore_state: None,
        restore_pp: Vec::<StateEntry>::new(),
    };
    let mut am_leader = false;
    let mut last_new_vc = None;
    let mut last_vc = None;
    {
        let mut server = server_mutex.lock().unwrap();
        if !check_rt(&rt_msg, msg_sig, &mut server) {
            return;
        }
        am_leader = server.am_leader();
        last_new_vc = Some(server.last_new_view.clone());
        last_vc = Some(server.last_vc.clone());
        // 1. update quorum_last_applied, try gc, wether do view change
        server.quorum_applied[rt_msg.who_send] = max(rt_msg.last_applied_seq, server.quorum_applied[rt_msg.who_send]);
        let mut tmp_apl = server.quorum_applied.clone();
        tmp_apl.sort_by(|a, b| b.cmp(a));
        let quorum_h = server.quorum_applied[config::F_NUM as usize] / config::K as i32 * config::K as i32;
        if quorum_h > server.get_h() + config::L as i32 {
            is_lag = true;
            server.status = constants::DO_VIEW_CHANGE;
        }
        let mut log_pp_set = Vec::new(); // used to help sender resotore pp msg

        // 2. restore prepare or commit msg
        for (seq, status, digest) in rt_msg.log_entry_status_set {
            if seq < server.get_h() || seq > server.get_h() + config::L as i32{
                continue;
            }
            if server.log.get_seq(seq).is_none() {
                server.log.set_seq(seq, LogEntry::default());
            }
            let mut cur_log_entry = server.log.get_seq(seq).unwrap();
            if status == constants::PREPARED &&
                !cur_log_entry.cert_prepare_vote[rt_msg.who_send] &&
                cur_log_entry.client_msg_checksum == digest {
                    cur_log_entry.cert_prepare_vote[rt_msg.who_send] = true;
                    cur_log_entry.cert_prepare_num += 1;
                    if cur_log_entry.cert_prepare_num > 2 * config::F_NUM {
                        cur_log_entry.entry_status = max(constants::PREPARED, cur_log_entry.entry_status);
                    }
            }

            if status == constants::COMMITED &&
            !cur_log_entry.cert_commit_vote[rt_msg.who_send] &&
            cur_log_entry.client_msg_checksum == digest {
                cur_log_entry.cert_commit_vote[rt_msg.who_send] = true;
                cur_log_entry.cert_commit_num += 1;
                if cur_log_entry.cert_commit_num > 2 * config::F_NUM {
                    cur_log_entry.entry_status = max(constants::COMMITED, cur_log_entry.entry_status);
                }
            }

            // means sender dont have pp msg in this slot
            if status == constants::LOG_ENTRY_INIT {
                if server.log.get_seq(seq).is_some() && 
                server.log.get_seq(seq).unwrap().entry_status >= constants::PREPARED{
                    log_pp_set.push(server.log.get_seq(seq).unwrap().generate_state());
                }
            }
        }

        // 2.2 construct rt_rpl to help sender restore its pp or state
        if (rt_msg.low_water + config::L as i32) < server.get_h() {
            // means sender need state transfer
            let sate_seq = rt_msg.last_applied_seq  / config::K as i32;
            rt_rpl.restore_state = Some((sate_seq,read_states(sate_seq).await));
        } else {
            // sender need restore pp
            rt_rpl.restore_pp = log_pp_set;
        }
    }
    // check wether sender need restore pp or state, if need, send rpl
    if rt_rpl.restore_pp.len() > 0 || rt_rpl.restore_state.is_some() {
        send_server(rt_msg.who_send, Msg::RtRplMsg(rt_rpl)).await;
    }
    try_gc(server_mutex).await;
    // if find lags make view change 
    if rt_msg.node_status == constants::DO_VIEW_CHANGE {
        if am_leader {
            send_server(rt_msg.who_send, Msg::NewViewMsg(last_new_vc.unwrap())).await;
        } else {
            send_server(rt_msg.who_send, Msg::VcMsg(last_vc.unwrap())).await;
        }
    }
    if is_lag {
        todo!("view change");
    }   


}

// when receive rt_rpl msg, receiver should restore its state and its pp
pub async fn do_rt_rpl(rt_rpl_msg: RtRplMsg,  server_mutex: Arc<Mutex<model::Server>>, msg_sig: &Vec<u8>) {
    {
        let mut server = server_mutex.lock().unwrap();
        if !check_rt_rpl(&rt_rpl_msg, msg_sig, &mut server) {
            return;
        }
        if rt_rpl_msg.restore_state.is_some() {
            // need to restore state
            // 1. install the state, example, K = 4, last applied = 3, so I need state_seq = 1. if Last applied = 4, so I need 1 too.
            let mut state_seq = server.applied / config::K as i32;
            if server.applied / config::K as i32 == 0 {
                state_seq += 1;
            }
            // 1.1 check wether I need this state
            if rt_rpl_msg.restore_state.as_ref().unwrap().0 != state_seq
            {
                return;
            }
            // 1.2 if it doesn't contain this key, insert them, and intialize the nested data structure
            let state_digest = sha256(&bincode::serialize(rt_rpl_msg.restore_state.as_ref().unwrap()).unwrap()); // use client msg as digest indicator
            if !server.restore_state_vote.contains_key(&state_seq) {
                server.restore_state.insert(state_seq, HashMap::new());
                server.restore_state.get_mut(&state_seq).unwrap().
                    insert(state_digest.clone(), (0, rt_rpl_msg.restore_state.as_ref().unwrap().1.clone()));
                server.restore_state_vote.insert(state_seq, vec![false; config::SERVER_NUM]);
            }

            // 1.2 store the state into the data structure, and if count > f, install it. 
            if !server.restore_state_vote.get(&state_seq).unwrap()[rt_rpl_msg.who_send] {
                server.restore_state.get_mut(&state_seq).unwrap().
                get_mut(&state_digest).unwrap().0 += 1;
            }

            let mut clean = false;
            if server.restore_state.get_mut(&state_seq).unwrap().
                get_mut(&state_digest).unwrap().0 > 2 * config::F_NUM {
                    // install the data, if the log is full, wait gc.
                    // if install is successful. clean the data structure
                    if install_state(rt_rpl_msg.restore_state.unwrap().1.clone(), &mut server) {
                        clean = true;
                    }
                }
            // 2. clear stored state whose state_seq < current applied
            if clean {
                server.restore_state = HashMap::new();
                server.restore_state_vote = HashMap::new();
            }
        } else {
            // need to restore pp
            for  state_entry in rt_rpl_msg.restore_pp {
                // check wether it already be occupied(is_some and > not LOG_ENTRY_INIT). LOG_ENTRY_INIT may receive prepare or commit before get pp  wether it is out of boud
                let seq = state_entry.n;
        
                if (server.log.get_seq(seq).is_some() && server.log.get_seq(seq).unwrap().entry_status > constants::LOG_ENTRY_INIT) ||
                 (seq < server.get_h() || seq > server.get_h() + config::L as i32){
                    continue;
                }

                if state_entry.client_msg.is_none() {
                    return;
                }
                let pp_digest = sha256(&bincode::serialize(&state_entry).unwrap()); 
                // if dont have it, add the key into corresponding data structure
                if !server.restore_pp_vote.contains_key(&seq) {
                    server.restore_pp_vote.insert(seq, vec![false; config::SERVER_NUM]);
                    server.restore_pp.insert(seq, HashMap::new());
                    if !server.restore_pp.get_mut(&seq).unwrap().contains_key(&pp_digest) {
                        server.restore_pp.get_mut(&seq).unwrap().insert(pp_digest.clone(), (0, state_entry.clone()));
                    }
                }
                // store the state into the data structure
                if !server.restore_state_vote.get(&seq).unwrap()[rt_rpl_msg.who_send] {
                    server.restore_pp.get_mut(&seq).unwrap().
                    get_mut(&pp_digest).unwrap().0 += 1;
                }
                // if count > 2f, install it
                let mut success = false;
                if server.restore_pp.get_mut(&seq).unwrap().get_mut(&pp_digest).unwrap().0 > 2 * config::F_NUM {
                    success = true;
                    server.log.set_seq(seq, state_entry.generate_log_entry_pp());
                }

                // if success, clear the data structure
                if success {
                    server.restore_pp.remove(&seq);
                    server.restore_pp_vote.remove(&seq);
                }
            }
        }
    }
    try_reply_apply(server_mutex);
}