use crate::config;



// note:Rust does not implement Default for all arrays
#[derive(Debug, Clone)]
pub struct Log_entry {
    log_type: i32,  // PRE-PREPARE, PREPARE, COMMIT, APPLIED, etc...
    v: i32,     // view number
    n: i32,     // sequence number
    client: i32,// client's number
    who_send: i32 ,// who send this message

	cert_prepare_num: i32,
	cert_prepare_vote: Vec<bool>,

    cert_commit_num: i32,
	cert_commit_vote: Vec<bool>,
}

impl Default for Log_entry {
    fn default() -> Self {
        Self { log_type: Default::default(), v: Default::default(), n: -1, client: Default::default(), who_send: Default::default(), cert_prepare_num: Default::default(), cert_prepare_vote: vec![false; config::SERVER_NUM], cert_commit_num: Default::default(), cert_commit_vote: vec![false; config::SERVER_NUM]}
    }
}


// #[derive(Debug)]
pub struct Server {
    // normal variables
	I_am: i32,                                      // Identification of server
	client_request:  Vec<(String, i32)>,   // used to maintain one semantic
    my_view: i32,
    applied: i32,
    who_leader: i32,
    log: Vec<Log_entry>,

    // leader variable
    log_assign: i32,

    // view change variable
     
    
}

impl Default for Server {
    fn default() -> Self {
        Self { I_am: Default::default(),
            client_request: vec![("-1".to_string(), 0); config::CLIENT_NUM] ,
            my_view: -1,
            applied: -1,
            who_leader: -1,
            log: vec![Default::default(); config::L],
            log_assign: -1,

            }
    }
}