use std::io;
use std::str;
mod plan;

fn get_username() -> String {
    let mut res = String::new();
    match io::stdin().read_line(&mut res) {
        Ok(_) => res,
        Err(_) => panic!("What the fuck?")
    }
}
         

fn main() {
    let i_username = get_username();
    let username = i_username.trim();
    let userpath = match plan::get_userpath(&username) {
        Ok(p) => p,
        Err(s) => panic!(s)
    };

    match plan::get_userplan(&userpath) {
        Ok(theplan) => {
            println!("PLAN: {}", theplan);
        },
        Err(s) => println!("{}", s)
    }
    
}
