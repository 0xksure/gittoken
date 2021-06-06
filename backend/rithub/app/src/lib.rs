pub mod app {
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

            let rs256_header = Header::new(jsonwebtoken::Algorithm::RS256);

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
