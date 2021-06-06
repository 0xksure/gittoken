pub mod errors {
    #[derive(Debug)]
    pub struct Error {
        status: usize,
        pub message: String,
    }

    impl Error {
        pub fn new(status: usize, message: String) -> Error {
            Error {
                status: status,
                message: message,
            }
        }
    }
}
