
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

// log and checkpoint management
    pub const L:usize = 60;
    pub const K:usize = 20;



