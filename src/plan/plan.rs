fn get_userpath(username: &str) -> Result<str, &'static str> {
    let c_username = match std::ffi::CString::new(username) {
        Ok(s)  => s,
        Err(_) => return Err("Could not convert username?")
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


