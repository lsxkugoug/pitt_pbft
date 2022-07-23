// clients
    pub const CLIENT_NUM: usize = 3;

    // clients' public keys file name
    pub const CLIENT_PK: &'static [&'static str] = &[        
        "clien1_pub",
        "clien2_pub",
        "clien3_pub",
        ];

    // clients' ips
    pub const CLIENT_IP: &'static [&'static str] = &[
        "67.171.64.218:2220",
        "67.171.64.218:2221",
        "67.171.64.218:2222",
        "67.171.64.218:2223",
    ];


// server
    pub const SERVER_NUM: usize = 4;
    // servers' public keys file name 
    pub const SERVER_PK: &'static [&'static str] = &[
        "server1_pub",
        "server2_pub",    
        "server3_pub",    
    ];
    // clients' ips
    pub const SERVER_IP: &'static [&'static str] = &[
        "127.0.0.1:2020",
        "127.0.0.1:2021",
        "127.0.0.1:2022",
        "127.0.0.1:2023",
    ];
    


// log and checkpoint management
    pub const L:usize = 60;
    pub const K:usize = 20;

