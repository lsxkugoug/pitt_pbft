use std::cmp::max;
use std::sync::{Arc, Mutex};
use std::vec;
use crate::network::message::{MsgWithSignature, ClientMsg, ClientReplyMsg};
use crate::{network::message, consensus::model, consensus::view_change,constants, config};
use tokio::time::{sleep, Duration};

// view-change msg: <VIEW-CHANGE, v + 1, last_scp, s, C, P, i>
// last_scp: last stable checkpoin
//  C is the stable certificate for last_scp? why
//  P is a set with a prepared certificate for each request that prepared at with a sequence number greater than 
pub async fn start_view_change(server_mutex: Arc<Mutex<model::Server>>){

}