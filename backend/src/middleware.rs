pub mod middleware {
    extern crate authorization;
    use rocket::fairing::{Fairing, Info, Kind};
    use rocket::http::hyper::header::Authorization;
    use rocket::{Data, Request, Response};
    pub struct Middleware {}

    impl Fairing for Middleware {
        fn info(&self) -> Info {
            Info {
                name: "GET/POST Counter",
                kind: Kind::Request | Kind::Response,
            }
        }
        fn on_request(&self, request: &mut Request, _: &Data) {
            log::info!("Request ");
            log::info!("uri: {}:", request.uri().path());
            request
                .headers()
                .get("Authorization")
                .for_each(|x| println!("ok {}", x));
        }
        fn on_response(&self, request: &Request, response: &mut Response) {
            log::info!("Response ");
            log::info!("uri: {}:", request.uri().path());
        }
    }

    impl Middleware {
        pub fn new() -> Middleware {
            Middleware {}
        }
    }
}
