pub mod user {
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
            INSERT INTO github_user (Username, Name)
            SELECT CAST($1 as varchar),$2
            WHERE NOT EXISTS (
                SELECT id FROM github_user where username=$1
            );
            ",
                &[&self.user_name, &self.name],
            );

            match query_result {
                Ok(_) => return Ok(()),
                Err(err) => return Err(Error::new(ErrorKind::Other, format!("{}", err))),
            }
        }

        pub fn add_address_to_user(
            &self,
            db: &postgres::Connection,
            address: &str,
        ) -> Result<(), Error> {
            let query_res = db.query(
                "
            UPDATE github_user(Eaddress) 
            VALUES ($1)
            WHERE Username=$2
            ",
                &[&address, &self.user_name],
            );

            match query_res {
                Ok(_) => return Ok(()),
                Err(err) => return Err(Error::new(ErrorKind::Other, format!("{}", err))),
            }
        }

        pub fn get_address_from_username(
            &self,
            db: &postgres::Connection,
        ) -> Result<String, Error> {
            let mut address = String::from("");
            for row in &db
                .query(
                    "
            SELECT Eaddress 
            FROM github_user
            WHERE Username=$1
            ",
                    &[&self.user_name],
                )
                .unwrap()
            {
                address = row.get("Eaddress");
            }

            if address == "" {
                return Err(Error::new(
                    ErrorKind::Other,
                    format!("could not retrieve address"),
                ));
            }
            Ok(address)
        }
    }
}
