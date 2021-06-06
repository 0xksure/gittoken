pub mod rocket_request_headers {
    use error::errors::Error;
    use rocket::http::Status;
    use rocket::request::{FromRequest, Outcome, Request};

    #[derive(Debug)]
    pub struct GithubWebhookHeaders {
        pub event: String,
    }

    impl<'r, 'a> FromRequest<'r, 'a> for GithubWebhookHeaders {
        type Error = Error;
        fn from_request(request: &'r Request<'a>) -> Outcome<Self, Self::Error> {
            let headers = request.headers();
            let mut event_iter = headers.get("X-GitHub-Event");
            let event = match event_iter.next() {
                Some(event) => event,
                None => {
                    return Outcome::Failure((
                        Status::Unauthorized,
                        Error::new(401, "unauthorized".to_string()),
                    ))
                }
            };
            Outcome::Success(GithubWebhookHeaders {
                event: String::from(event),
            })
        }
    }
}
