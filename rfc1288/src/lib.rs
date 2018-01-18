extern crate bytes;

#[macro_use]
extern crate error_chain;

error_chain! {
    errors {
        BadProtocol {
            description("Protocol prefix not recognized")
        }
        BadRequest {
            description("Protocol request does not meet specification")
        }
    }
}

// RFC1288, Section 2.3: Query Specification
// The Finger query specification is defined:
//      {Q1}    ::= [{W}|{W}{S}{U}]{C}
//      {Q2}    ::= [{W}{S}][{U}]{H}{C}
//      {U}     ::= username
//      {H}     ::= @hostname | @hostname{H}
//      {W}     ::= /W
//      {S}     ::= <SP> | <SP>{S}
//      {C}     ::= <CRLF>
//
// The username specification is taken from:
// [TK look up the slideshow on Unix username conventions]
//
// Almost no checking is done on the hostname.

#[derive(Debug)]
#[derive(PartialEq)]
pub enum Request {
    UserList,
    User(Vec<u8>),
    Remote(Vec<u8>, Vec<u8>),
}

#[inline]
fn is_unix_conventional(i: Option<&u8>) -> bool {
    match i {
        Some(i) => (*i >= b'0' && *i <= b'9') || (*i >= b'A' && *i <= b'Z') || (*i >= b'a' && *i <= b'z'),
        None => false,
    }
}

pub fn parse_rfc1288_request(buffer: &bytes::Bytes) -> Result<Request> {
    if buffer.len() < 2 {
        return ErrorKind::BadProtocol);
}

    let mut req = buffer.into_iter().peekable();
    if req.next() != Some(b'/') { return Err(ErrorKind::BadProtocol); }

    {
        let n = req.next();
        if n != Some(b'W') && n != Some(b'w') { return Err(ErrorKind::BadProtocol); }
    }

    if req.peek() == None {
        return Ok(Request::UserList);
    }

    if req.peek() != Some(&b' ') {
        return Err(ErrorKind::BadProtocol);
    }

    loop {
        if req.peek() != Some(&b' ') {
            break;
        }
        req.next();
    }

    if req.peek() == None {
        return Ok(Request::UserList);
    }

    let mut user = Vec::with_capacity(512);
    let mut host = Vec::with_capacity(512);

    while is_unix_conventional(req.peek()) {
        user.push(req.next().unwrap())
    }

    if req.peek() == Some(&b' ') || req.peek() == None {
        user.shrink_to_fit();
        return Ok(Request::User(user));
    }

    if req.next() != Some(b'@') {
        return Err(ErrorKind::BadRequest);
    }

    while req.peek() != Some(&b' ') && req.peek() != None {
        host.push(req.next().unwrap());
    }

    Ok(Request::Remote(user, host))
}

#[cfg(test)]
mod tests {
    use super::*;
    use bytes::Bytes;

    #[test]
    fn good_list() {
        let res = parse_rfc1288_request(&Bytes::from("/W"));
        match res {
            Ok(c) => assert_eq!(c, Request::UserList),
            Err(_) => assert!(false),
        }
    }

    #[test]
    fn good_list_w_spaces() {
        let res = parse_rfc1288_request(&Bytes::from("/W                           "));
        match res {
            Ok(c) => assert_eq!(c, Request::UserList),
            Err(_) => assert!(false),
        }
    }

    
    #[test]
    fn bad_start0() {
        let res = parse_rfc1288_request(&Bytes::from(""));
        match res {
            Ok(_) => assert!(false),
            Err(e) => assert_eq!(e, ErrorKind::BadProtocol),
        }
    }

    #[test]
    fn bad_start1() {
        let res = parse_rfc1288_request(&Bytes::from("/"));
        match res {
            Ok(_) => assert!(false),
            Err(e) => assert_eq!(e, ErrorKind::BadProtocol),
        }
    }

    #[test]
    fn bad_start2() {
        let res = parse_rfc1288_request(&Bytes::from("/X"));
        match res {
            Ok(_) => assert!(false),
            Err(e) => assert_eq!(e, ErrorKind::BadProtocol),
        }
    }

    #[test]
    fn good_name() {
        let res = parse_rfc1288_request(&Bytes::from("/W foozle"));
        match res {
            Ok(e) => if let Request::User(v) = e { assert_eq!(b"foozle", v.as_slice()) } else { assert!(false); }, 
            Err(_) => assert!(false)
        }
    }

    #[test]
    fn good_name_extra_space() {
        let res = parse_rfc1288_request(&Bytes::from("/W foozle   "));
        match res {
            Ok(e) => if let Request::User(v) = e { assert_eq!(b"foozle", v.as_slice()) } else { assert!(false); }, 
            Err(_) => assert!(false)
        }
    }

    #[test]
    fn good_name_w_host() {
        let res = parse_rfc1288_request(&Bytes::from("/W foozle@localhost"));
        match res {
            Ok(e) => if let Request::Remote(u, h) = e {
                assert_eq!(b"foozle", u.as_slice());
                assert_eq!(b"localhost", h.as_slice()); }
            else {
                assert!(false);
            }, 
            Err(_) => assert!(false)
        }
    }

    #[test]
    fn good_name_w_host_and_spaces() {
        let res = parse_rfc1288_request(&Bytes::from("/W   foozle@localhost   "));
        match res {
            Ok(e) => if let Request::Remote(u, h) = e {
                assert_eq!(b"foozle", u.as_slice());
                assert_eq!(b"localhost", h.as_slice()); }
            else {
                assert!(false);
            }, 
            Err(_) => assert!(false)
        }
    }

    #[test]
    fn good_name_w_host_and_spaces_and_lowercase_w() {
        let res = parse_rfc1288_request(&Bytes::from("/w   foozle@localhost   "));
        match res {
            Ok(e) => if let Request::Remote(u, h) = e {
                assert_eq!(b"foozle", u.as_slice());
                assert_eq!(b"localhost", h.as_slice()); }
            else {
                assert!(false);
            }, 
            Err(_) => assert!(false)
        }
    }
    
    #[test]
    fn bad_name() {
        let res = parse_rfc1288_request(&Bytes::from("/W   foozle..   "));
        match res {
            Ok(_) => assert!(false),
            Err(e) => assert_eq!(e, ErrorKind::BadRequest),
        }
    }
    
}
