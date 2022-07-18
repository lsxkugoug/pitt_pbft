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

`cargo run -- -i 127.0.0.1:2022`