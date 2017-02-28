extern crate libc;

use std::io;


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



fn main() {
    let i_username = get_username();
    let username = i_username.trim();
    match get_userpath(username) {
        Ok(path) => println!("HEY: {}", path),
        Err(s) => println!("{}", s)
    }
}
