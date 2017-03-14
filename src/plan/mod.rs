extern crate libc;
use std::fs;
use std;
use std::io::Read;

const MAX_PLANSIZE: u64 = 128 * 1024;

pub fn get_userpath(username: &str) -> Result<String, String> {
    let c_username = match std::ffi::CString::new(username) {
        Ok(s)  => s,
        Err(_) => return Err(format!("Could not convert username {}?", username))
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
        0 => Err(format!("User {} not found.", username)),
        _ => Ok(unsafe {std::ffi::CStr::from_ptr(pwd.pw_dir)}.to_string_lossy().into_owned())
    }
}

pub fn get_userplan(userpath: &str) -> Result<String, String> {
    let planpath = format!("{}/{}", userpath, ".plan");
    let res = fs::metadata(&planpath);
    let metadata = match res {
        Ok(s) => s,
        Err(_) => return Err(format!("Could not access user plan {}.", planpath))
    };

    if !metadata.is_file() {
        return Err(format!("Plan {} is not a file, cannot be returned.", planpath));
    }
    
    if metadata.len() > MAX_PLANSIZE {
        return Err(format!("Plan {} is too large to return.", planpath));
    }

    let mut plan = match fs::File::open(&planpath) {
        Ok(s) => s,
        Err(_) => return Err(format!("Could not access {}", planpath))
    };

    let mut contents = String::new();
    let readres = plan.read_to_string(&mut contents);
    match readres {
        Ok(_) => Ok(contents),
        Err(_) => Err(format!("Plan {} could not be read", planpath))
    }
}
