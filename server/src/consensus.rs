use std::sync::{Arc};
use tokio::sync::{Mutex};
use tokio::net::{TcpListener, TcpStream};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use serde_json::Value;
use crate::server;


pub fn do_client_request(msg: Value, server: Arc<Mutex<server::Server>>) {

}

pub fn do_pre_prepare() {

}

pub fn do_prepare() {

}

pub fn do_commit() {

}

pub fn do_vc() {

}