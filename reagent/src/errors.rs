quick_error! {
    #[derive(PartialEq)]
    #[derive(Debug)]
    pub enum ReagentError {
        BadProtocol {
            description("Protocol prefix not recognized")
        }
        BadRequest {
            description("Protocol request does not meet specification")
        }
    }
}
