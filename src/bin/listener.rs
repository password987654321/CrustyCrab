use std::net::{SocketAddr, UdpSocket, TcpListener};
use std::{thread, time};
use std::sync::{Arc, Mutex};
use crabby_patty_formula::*;
use std::mem::drop;
use std::io::{self, Write};

// Send packets over
//  - Bytes (UDP, TCP)
//  - HTTP
//  - DNS

// Defines essential functions

fn main(){
    let port: u16 = 2120;
    let id: u64 = 69;
    let protocol: &str = "udp";

    let mut lsn = new_lsn(id);

    let address = SocketAddr::from(([127, 0, 0, 1], port));

    let mut sb: Arc<Mutex<SharedBuffer>> = Arc::new(Mutex::new(SharedBuffer {
        cc: 0,
        buff: [0; 2048].to_vec(),
    }));

    let mut sb_arc = Arc::clone(&sb);

    // spawn the listnener
    let thr = thread::spawn(move ||
        {
            crabby_patty_formula::lsn_run(&mut lsn, protocol, address, &mut sb);
        }
    );

    thread::sleep(time::Duration::from_millis(5000));
    
    // test the module system by executing a module
    // commented out bc its not working at the moment
    let mut code: u8 = 6;
    if true {
        let mut buffer = sb_arc.lock().unwrap();
        buffer.cc = code;
    }
    
    let mut memo: String = "example".to_string();
    let mut swap = true;
    loop {
        if swap {
            io::stdout().flush().unwrap();
            memo = "example".to_string();
            // write to shared buffer
            let mut buffer = sb_arc.lock().unwrap();
            buffer.cc = 6;
            buffer.buff = memo.as_bytes().to_vec();
            swap = false;
        }
        else {
            let mut buffer = sb_arc.lock().unwrap();
            if !String::from_utf8_lossy(&buffer.buff[..]).contains(memo.as_str()) {
                println!("{}", String::from_utf8_lossy(&buffer.buff[..]));
                io::stdout().flush().unwrap();
                memo = String::new();
                break;
            }
        }

        // wait until shared buffer changes
        // print changed shared buffer
        thread::sleep(time::Duration::from_millis(10));
    }
    

    
    // shell time
    let mut code: u8 = 5;
    if true {
        let mut buffer = sb_arc.lock().unwrap();
        buffer.cc = code;
    }

    let mut swap = true;

    // now we interact
    print!("anchovy_shell $ ");
    io::stdout().flush().unwrap();
    let mut memo: String = String::new();
    loop {
        if swap {
            io::stdout().flush().unwrap();
            // read from stdin
            io::stdin().read_line(&mut memo);
            // check if we need to execute a module
            // write command to shared buffer
            let mut buffer = sb_arc.lock().unwrap();
            buffer.buff = memo.as_bytes().to_vec();
            swap = false;
        }
        else {
            let mut buffer = sb_arc.lock().unwrap();
            if !String::from_utf8_lossy(&buffer.buff[..]).contains(memo.as_str()) {
                print!("{}anchovy_shell $ ", String::from_utf8_lossy(&buffer.buff[..]));
                io::stdout().flush().unwrap();
                memo = String::new();
                swap = true;
            }
        }

        // wait until shared buffer changes
        // print changed shared buffer
        thread::sleep(time::Duration::from_millis(10));
    }
    
    thr.join().unwrap();
}