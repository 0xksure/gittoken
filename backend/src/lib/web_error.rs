pub mod WebError {
    use std::fmt;
    use std::io;
    #[derive(Debug)]
    pub struct WebError {
        Status: usize,
        Message: String,
    }

    impl WebError {
        pub fn new(status: usize, message: String) -> WebError {
            WebError {
                Status: status,
                Message: message,
            }
        }
    }

    impl fmt::Display for WebError {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            let err_msg = match self.Status {
                404 => "404. Not found",
                _ => "undefined status code ",
            };
            write!(f, "{}", err_msg)
        }
    }
    impl From<io::Error> for WebError {
        fn from(error: io::Error) -> Self {
            WebError {
                Status: 500,
                Message: error.to_string(),
            }
        }
    }
}
