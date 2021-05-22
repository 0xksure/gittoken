pub mod github_api {
    use reqwest;
    use reqwest::header::ACCEPT;
    use serde::{Deserialize, Serialize};
    use serde_json;
    use std::io::{Error, ErrorKind};
    #[derive(Debug, Clone)]
    pub struct GithubConfig {
        access_token: String,
    }

    #[derive(Debug, Deserialize, Serialize)]
    pub struct User {
        #[serde(rename(serialize = "username"))]
        pub login: String,
        pub name: String,
        pub email: Option<String>,
    }

    #[derive(Debug, Serialize, Deserialize)]
    struct IssueComment {
        body: String,
    }

    impl GithubConfig {
        pub fn new(access_token: &str) -> GithubConfig {
            GithubConfig {
                access_token: access_token.to_string(),
            }
        }

        fn create_request(
            &self,
            path: String,
            client: reqwest::blocking::Client,
        ) -> reqwest::blocking::RequestBuilder {
            let authorization_header = format!("token {}", self.access_token.clone());
            let url = format!("https://api.github.com/{}", path);
            client
                .get(url)
                .header(reqwest::header::AUTHORIZATION, authorization_header)
                .header(reqwest::header::USER_AGENT, "request")
        }

        pub fn user(&self, client: reqwest::blocking::Client) -> Result<User, Error> {
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
            let gh_user = resp.json::<User>().unwrap();

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

        pub fn comment_issue(
            &self,
            owner: &str,
            repo: &str,
            issue_number: &str,
            message: &str,
        ) -> Result<(), Error> {
            let authorization_header = format!("token {}", self.access_token.clone());
            let issue_path = format!("repos/{}/{}/issues/{}/comments", owner, repo, issue_number);
            let url = format!("https://api.github.com/{}", issue_path);
            let client = reqwest::blocking::Client::new();
            let issue_comment = IssueComment {
                body: message.to_string(),
            };
            let res = match client
                .post(url)
                .header(reqwest::header::AUTHORIZATION, authorization_header)
                .header(reqwest::header::USER_AGENT, "request")
                .header(ACCEPT, "application/json")
                .json(&issue_comment)
                .send()
            {
                Ok(res) => res,
                Err(err) => return Err(Error::new(std::io::ErrorKind::Other, format!("{:}", err))),
            };

            if !res.status().is_success() {
                return Err(Error::new(
                    std::io::ErrorKind::Other,
                    format!(
                        "github_app.comment_issue.failure error_code: {}",
                        res.status()
                    ),
                ));
            }
            return Ok(());
        }
    }
}

pub mod github_app {
    use jsonwebtoken::Header;
    use log::info;
    use reqwest;
    use serde::{Deserialize, Serialize};
    use std::fmt;
    use std::io::{Error, ErrorKind};
    use std::ops::{Add, Sub};
    use std::time;

    #[derive(Debug, Serialize, Deserialize)]
    struct Claims {
        iat: u64,
        exp: u64,
        iss: String,
    }
    #[derive(Debug, Serialize, Deserialize)]
    pub struct InstallationAccessToken {
        pub token: String,
        pub expires_at: String,
    }

    impl fmt::Display for InstallationAccessToken {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            write!(
                f,
                "InstallationAccessToken(
                token: {},
                expires_at: {} 
                )",
                self.token, self.expires_at
            )
        }
    }

    pub struct Config {
        app_id: String,
        rsa_pem: Vec<u8>,
    }
    impl Config {
        pub fn new(app_id: String, rsa_pem: Vec<u8>) -> Config {
            Config {
                app_id: app_id.clone(),
                rsa_pem: rsa_pem,
            }
        }

        fn new_claim(&self) -> Result<Claims, Error> {
            let iat = match time::SystemTime::now()
                .sub(time::Duration::new(60, 0))
                .duration_since(time::SystemTime::UNIX_EPOCH)
            {
                Ok(res) => res.as_secs(),
                Err(err) => return Err(Error::new(ErrorKind::Other, err)),
            };

            let exp = match time::SystemTime::now()
                .add(time::Duration::new(60 * 10, 0))
                .duration_since(time::SystemTime::UNIX_EPOCH)
            {
                Ok(res) => res.as_secs(),
                Err(err) => return Err(Error::new(ErrorKind::Other, err)),
            };
            Ok(Claims {
                iat: iat,
                exp: exp,
                iss: self.app_id.clone(),
            })
        }

        pub fn authenticate_app(
            &self,
            installation_id: String,
        ) -> Result<InstallationAccessToken, Error> {
            let rsa_pem_u8: &[u8] = &self.rsa_pem;
            let pem_encoding_key = jsonwebtoken::EncodingKey::from_rsa_pem(rsa_pem_u8).unwrap();
            let new_auth_claim = match self.new_claim() {
                Ok(claims) => claims,
                Err(err) => return Err(Error::new(ErrorKind::Other, err)),
            };

            let rs256_header = jsonwebtoken::Header::new(jsonwebtoken::Algorithm::RS256);

            let jwt_string =
                match jsonwebtoken::encode(&rs256_header, &new_auth_claim, &pem_encoding_key) {
                    Ok(jwt) => jwt,
                    Err(err) => return Err(Error::new(ErrorKind::Other, err)),
                };
            let req_client = reqwest::blocking::Client::new();
            let res = match req_client
                .post(format!(
                    "https://api.github.com/app/installations/{}/access_tokens",
                    installation_id
                ))
                .header("Accept", "application/vnd.github.v3+json")
                .header("Authorization", format!("bearer {}", jwt_string))
                .header(reqwest::header::USER_AGENT, "request")
                .send()
            {
                Ok(res) => res,
                Err(err) => return Err(Error::new(ErrorKind::Other, err)),
            };

            if !res.status().is_success() {
                info!("authenticate_app.access_token.failure");
                return Err(Error::new(
                    ErrorKind::Other,
                    format!("expected status 200 but got {}", res.status()),
                ));
            }

            let installation_access_code = match res.json::<InstallationAccessToken>() {
                Ok(res) => res,
                Err(err) => {
                    info!("authenticate_app.access_token.decode");
                    return Err(Error::new(ErrorKind::Other, format!("got error {}", err)));
                }
            };
            Ok(installation_access_code)
        }
    }
}
