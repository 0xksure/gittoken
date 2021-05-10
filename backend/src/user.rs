pub mod user {
    use futures::executor::block_on;
    use rocket_contrib::databases::postgres;
    use std::io::{Error, ErrorKind};

    pub struct User {
        user_name: String,
        name: String,
    }

    impl User {
        pub fn new(user_name: &str, name: &str) -> User {
            User {
                user_name: user_name.to_string(),
                name: name.to_string(),
            }
        }

        pub fn create_user_if_not_exist(
            &self,
            db: &postgres::Connection,
            token: &str,
        ) -> Result<(), Error> {
            let query_result = db.query(
                "
            INSERT INTO github_user (Username, Name, Token)
            SELECT CAST($1 as varchar),$2,$3
            WHERE NOT EXISTS (
                SELECT id FROM github_user where username=$1
            );
            ",
                &[&self.user_name, &self.name, &token],
            );

            match query_result {
                Ok(_) => return Ok(()),
                Err(err) => return Err(Error::new(ErrorKind::Other, format!("{}", err))),
            }
        }
    }
}
