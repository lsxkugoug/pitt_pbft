
// clients
    pub const CLIENT_NUM: usize = CLIENT_IP.len();

    // clients' ips
    pub const CLIENT_IP: &'static [&'static str] = &[
        "67.171.64.218:2220",
        "67.171.64.218:2221",
        "67.171.64.218:2222",
        "67.171.64.218:2223",
    ];


// server
    pub const F_NUM :i32 = 1;
    pub const SERVER_NUM: usize = (F_NUM * 3 + 1) as usize; // DONT CHANGE IT
    pub const SERVER_IP: &'static [&'static str] = &[
        "127.0.0.1:2020",
        "127.0.0.1:2021",
        "127.0.0.1:2022",
        "127.0.0.1:2023",
    ];


// consensus
    pub const TIMEOUT: u64 = 300;   // when timeout, go to view change

// log and checkpoint management
    pub const L:usize = 60;
    pub const K:usize = 20;


// retransmission
    pub const RT_INTERV: u128 = 50;  // nodes periodically broadcast their status, receiver nodes periodically accept their status msg RT_INTERV * 2 / 3
    pub const RT_RCV: u128 = (RT_INTERV * 2) / 3;




