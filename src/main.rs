extern crate libc;

use std::io;
use libc::{getpwnam};
use std::mem;


fn get_username() -> String {
    let mut res = String::new();
    match io::stdin().read_line(&mut res) {
        Ok(_) => res,
        Err(_) => panic!("What the fuck?")
    }
}
         
fn get_userpath(username: &str) -> String {
    let c_username = match std::ffi::CString::new(username) {
        Ok(s)  => s,
        Err(_) => panic!("Could not convert username?")
    };

    unsafe {
        println!("YES 1");
        let c_passwd = getpwnam(c_username.as_ptr());
        if c_passwd == 0 {
            return String::from("");
        }
        let s = std::ffi::CStr::from_ptr((*c_passwd).pw_dir).to_string_lossy().into_owned();
        mem::drop(c_passwd);
        s
    }
}



fn main() {
    let i_username = get_username();
    let username = i_username.trim();
    let path = get_userpath(username);
    println!("HEY: {}", path);
}
