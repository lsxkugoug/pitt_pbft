# pitt_pbft

This project use `clap`to control command line.

Get help

`cargo run -- -h`

```
server 0.1.0
Simple program to greet a person

USAGE:
    server --ip <IP>

OPTIONS:
    -h, --help       Print help information
    -i, --ip <IP>    bind ip:port
    -V, --version    Print version information
```



Specify ip as same in config servers' ip



Before run the engine, generating public key and server keys is required

Run `cargo run --bin generate_kp` to generate key pairs.

Then open server

`cargo run --bin pbft_engine -- -i 127.0.0.1:2022`





TODO LIST:

repair the problem when a node receive pm cm before receiving pp from leader: √

garbage collection feature

checkpoint management feature

State transfer feature

Message Retransmission feature

VIEW-CHANGE logic



Test three normal phase

Test view change











Research part: proactive recovery



