pub mod jwt_authentication {
    use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
    use serde::{Deserialize, Serialize};
    use std::time::{Duration, SystemTime};
    use std::{io, ops::Add};

    pub struct Jwt {
        shared_secret: String,
    }
    #[derive(Debug, Serialize, Deserialize)]
    struct Claims {
        sub: String,
        company: String,
        exp: usize,
    }

    impl Jwt {
        pub fn new(shared_secret: &str) -> Jwt {
            Jwt {
                shared_secret: shared_secret.to_string(),
            }
        }

        pub fn new_token(&self) -> Result<String, io::Error> {
            let expiry_interval = Duration::new(60 * 60, 0);

            let expiration_time_res = SystemTime::now()
                .add(expiry_interval)
                .duration_since(SystemTime::UNIX_EPOCH);

            let expiration_time = match expiration_time_res {
                Ok(res) => res.as_secs(),
                Err(err) => return Err(io::Error::new(io::ErrorKind::Other, err.to_string())),
            };

            let claims = Claims {
                sub: String::from("ok"),
                company: String::from("ok"),
                exp: expiration_time as usize,
            };
            let jwt = match encode(
                &Header::default(),
                &claims,
                &EncodingKey::from_secret(self.shared_secret.as_ref()),
            ) {
                Ok(res) => res,
                Err(err) => return Err(io::Error::new(io::ErrorKind::Other, err.to_string())),
            };

            Ok(jwt)
        }

        fn validate_token(&self, token: &str) -> Result<(), io::Error> {
            let decoded_jwt = decode::<Claims>(
                token,
                &DecodingKey::from_secret(self.shared_secret.as_ref()),
                &Validation::default(),
            );
            let _ = match decoded_jwt {
                Ok(res) => res,
                Err(err) => return Err(io::Error::new(io::ErrorKind::Other, err)),
            };
            Ok(())
        }
    }
}
