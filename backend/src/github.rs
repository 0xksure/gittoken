pub mod github_api {
    use reqwest;
    use serde::Deserialize;
    use serde_json;
    use std::io::{Error, ErrorKind};
    #[derive(Debug, Clone)]
    pub struct Gh {
        access_token: String,
    }

    #[derive(Debug, Deserialize)]
    pub struct GhUser {
        pub login: String,
        pub name: String,
        pub email: Option<String>,
    }
    impl Gh {
        pub fn new(access_token: &str) -> Gh {
            Gh {
                access_token: access_token.to_string(),
            }
        }

        pub fn user(&self, client: reqwest::blocking::Client) -> Result<GhUser, Error> {
            let authorization_header = format!("token {}", self.access_token.clone());
            let req = client
                .get("https://api.github.com/user")
                .header(reqwest::header::AUTHORIZATION, authorization_header)
                .header(reqwest::header::USER_AGENT, "request");

            let resp = match req.send() {
                Ok(r) => r,
                Err(err) => return Err(Error::new(std::io::ErrorKind::Other, format!("{:}", err))),
            };
            if !(resp.status() == reqwest::StatusCode::OK) {
                return Err(Error::new(
                    ErrorKind::Other,
                    format!("expected 200 returned {}", resp.status()),
                ));
            }
            let gh_user = resp.json::<GhUser>().unwrap();

            Ok(gh_user)
        }

        pub fn authentiate(&self, client: reqwest::blocking::Client) -> Result<(), Error> {
            let authorization_header = format!("token {}", self.access_token.clone());
            let req = client
                .get("https://api.github.com/")
                .header(reqwest::header::AUTHORIZATION, authorization_header)
                .header(reqwest::header::USER_AGENT, "request");

            let resp = match req.send() {
                Ok(r) => r,
                Err(err) => return Err(Error::new(std::io::ErrorKind::Other, format!("{:}", err))),
            };
            Ok(())
        }
    }
}
