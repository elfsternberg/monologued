error_chain! {
    errors {
        ConnectionsExhausted {
            description("Tokens exhausted")
            display("The token pool has been exhausted -- Too many connections.")
        }
    }

    foreign_links {
        Io(::std::io::Error) #[cfg(unix)] #[doc = "A wrapper around `std::io::Error`"];
    }
}
