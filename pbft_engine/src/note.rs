//! This file contains some test for practice, useless


#[cfg(test)]
mod test{
    use std::{thread::{sleep_ms, sleep}, fs, vec, collections::HashMap};

    use futures::prelude::*;
    use serde_json::json;
    use tokio::net::TcpStream;
    use tokio_serde::formats::*;
    use tokio_util::codec::{FramedWrite, LengthDelimitedCodec};

    async fn print_i(i: String) {
            print!("r {}\n", i);
            // fs::write("./aaa", i).expect("Unable to write file");
    }

    // good way to use tokio process multiple 
    #[tokio::main]
    #[test]
    async fn test_write_file() {
        let mut tasks = Vec::new();
        for i in 0..5 {
            let i = i as i32;
            tasks.push(tokio::spawn(async move {
                print_i(i.to_string()).await; 
            }));
        };
        for t in tasks {
            t.await.unwrap(); 
        }
    }
    
    #[tokio::main]
    #[test]
    async fn test_map() {
        let mut process_map: HashMap<i32, (i32, i32)> = HashMap::new();
        process_map.insert(1, (1,2));
        let x = process_map.entry(1).or_insert((1,1));
        x.1 = 6;
        print!("{}", process_map.get(&1).unwrap().1);
    }
}

