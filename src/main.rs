extern crate libc;

use std::io;
use std::str;


fn get_username() -> String {
    let mut res = String::new();
    match io::stdin().read_line(&mut res) {
        Ok(_) => res,
        Err(_) => panic!("What the fuck?")
    }
}
         
fn get_userpath(username: &str) -> Result<String, &'static str> {
    let c_username = match std::ffi::CString::new(username) {
        Ok(s)  => s,
        Err(_) => panic!("Could not convert username?")
    };
    
    let mut pwbuf = [0; 4096];
    let mut pwd: libc::passwd = unsafe { std::mem::zeroed() };
    let mut result: *mut libc::passwd = std::ptr::null_mut();
    
    unsafe {
        libc::getpwnam_r(c_username.as_ptr(),
                         &mut pwd as *mut _,
                         pwbuf.as_mut_ptr(),
                         pwbuf.len() as libc::size_t,
                         &mut result as *mut _)
    };
    match result as u32 {
        0 => Err("User not found."),
        _ => Ok(unsafe {std::ffi::CStr::from_ptr(pwd.pw_dir)}.to_string_lossy().into_owned())
    }
}

fn get_userplan(userdir: &String) -> Result<u32, &'static str> {
    let planpath = userdir + ".plan";
    let planpath_c = match std::ffi::CString::new(planpath) {
        Ok(s) => s,
        Err(_) => panic!("Could not build new plan path?")
    };

    let mut statbuf: libc::stat = unsafe { std::mem::zeroed() };

    let result = unsafe {
        libc::lstat(planpath_c.as_ptr(), &mut statbuf)
    };
    match result as u32 {
        0 => Ok(result),
        _ => panic!("Could not find plan on path.")
    }
    return statbuf.st_size;
}
    

fn main() {
    let i_username = get_username();
    let username = i_username.trim();
    match get_userpath(username) {
        Ok(path) => {
            println!("HEY: {}", path);
            let filesize = get_userplan(path);
            println!("SIZE: {}", filesize);
        },
        Err(s) => println!("{}", s)
    }
    
}
