use postgres::{Client, NoTls};
use refinery::{Error, Report};
use std::io;
pub mod web_error;

mod embedded {
    use refinery::embed_migrations;
    embed_migrations!("src/migrations");
}

pub fn connect_to_postgres(database_url: &str) -> Result<Client, postgres::Error> {
    let new_connection = Client::connect(database_url, NoTls);

    let client = match new_connection {
        Ok(res) => res,
        Err(err) => return Err(err),
    };
    Ok(client)
}

pub fn migrate(database_url: &str) -> Result<(), io::Error> {
    let new_connection = Client::connect(database_url, NoTls);
    let mut client = match new_connection {
        Ok(res) => res,
        Err(err) => return Err(io::Error::new(io::ErrorKind::Other, format! {"{}",err})),
    };
    let res: Result<Report, Error> = embedded::migrations::runner().run(&mut client);
    let report = match res {
        Ok(res) => res,
        Err(err) => return Err(io::Error::new(io::ErrorKind::Other, format! {"{}",err})),
    };

    report
        .applied_migrations()
        .iter()
        .for_each(|migration| log::info!("{}", migration.to_string()));
    Ok(())
}
