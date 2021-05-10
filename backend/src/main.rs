#![feature(proc_macro_hygiene, decl_macro)]

#[macro_use]
extern crate rocket_contrib;

#[macro_use]
extern crate rocket;
use envconfig::Envconfig;
use hyper::StatusCode;
use log::info;
use reqwest::header::ACCEPT;
use reqwest::{self, redirect};
use rocket::http::{ContentType, Cookie, Cookies, SameSite, Status};
use rocket::request::{FromRequest, Outcome, Request};
use rocket::response::{self, Responder, Response};
use rocket::State;
use serde::{Deserialize, Serialize};
use std::io::Cursor;
mod github;
use rocket_contrib::json::{Json, JsonValue};
mod lib;
use lib::web_error::WebError::WebError;
mod middleware;
mod user;
use rocket_contrib::database;
use rocket_contrib::databases::postgres;
use rocket_cors::{AllowedHeaders, AllowedOrigins};
extern crate authorization;
#[database("my_db")]
struct MyPgDatabase(postgres::Connection);

#[derive(Envconfig)]
struct Config {
    #[envconfig(from = "GITHUB_OAUTH_CLIENT_ID")]
    pub client_id: String,
    #[envconfig(from = "GITHUB_OAUTH_SECRET")]
    pub client_secret: String,
    #[envconfig(from = "DATABASE_URL")]
    pub database_url: String,
    #[envconfig(from = "SHARED_KEY")]
    pub shared_key: String,
}

struct Api {
    client_id: String,
    client_secret: String,
    shared_key: String,
}

#[derive(Debug)]
struct ResponseBodyError {
    status: Status,
    message: JsonValue,
}

impl<'a> Responder<'a> for ResponseBodyError {
    fn respond_to(self, req: &Request) -> response::Result<'a> {
        Response::build()
            .header(ContentType::JSON)
            .status(self.status)
            .ok()
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct Oauth {
    client_id: String,
    client_secret: String,
    code: String,
}

#[derive(Debug, Deserialize)]
struct AccessTokenResponse {
    access_token: String,
}

fn authenticate_user() -> Result<(), WebError> {
    Ok(())
}

#[derive(Debug)]
struct AuthorzationHeader(String);

impl<'r, 'a> FromRequest<'r, 'a> for AuthorzationHeader {
    type Error = WebError;
    fn from_request(req: &'r Request<'a>) -> Outcome<Self, Self::Error> {
        let authorization_cookie = req.cookies().get_private("authorization");
        match authorization_cookie {
            Some(authorization_header) => {
                Outcome::Success(AuthorzationHeader(authorization_header.value().to_string()))
            }
            None => Outcome::Failure((
                Status::Unauthorized,
                WebError::new(401, "unauthorized".to_string()),
            )),
        }
    }
}

#[get("/user")]
fn get_user(
    api: State<Api>,
    conn: MyPgDatabase,
    authorization_header: AuthorzationHeader,
) -> Result<(), ResponseBodyError> {
    let access_token = authorization_header.0;
    info!("Authorization header: {:?}", access_token);
    Ok(())
}

#[get("/github/login?<code>")]
fn github_login<'a>(
    api: State<Api>,
    code: Option<String>,
    conn: MyPgDatabase,
    mut cookies: Cookies,
) -> Result<rocket::Response<'a>, ResponseBodyError> {
    info!("github_login");
    let _gcode = match code {
        Some(res) => res,
        None => {
            return Err(ResponseBodyError {
                status: Status::Unauthorized,
                message: json!({"message":"unautorized"}),
            })
        }
    };
    info!("code: {}", _gcode);
    let github_post = Oauth {
        client_id: api.client_id.clone(),
        client_secret: api.client_secret.clone(),
        code: _gcode.clone(),
    };
    let req_client = reqwest::blocking::Client::new();
    let res = match req_client
        .post("https://github.com/login/oauth/access_token")
        .header(ACCEPT, "application/json")
        .json(&github_post)
        .send()
    {
        Ok(res) => res,
        Err(err) => {
            info!("error when retrieving access token: {}", err.to_string());
            return Err(ResponseBodyError {
                status: Status::Unauthorized,
                message: json!({"message":"unautorized"}),
            });
        }
    };

    let resp_code = res.status();
    if resp_code != reqwest::StatusCode::OK {
        info!("access token response code: {}", resp_code.to_string());
        return Err(ResponseBodyError {
            status: Status::Unauthorized,
            message: json!({"message":"unautorized"}),
        });
    }
    let access_token_response_res = res.json::<AccessTokenResponse>();
    info!("bytes: {:?}", access_token_response_res);
    let access_token = match access_token_response_res {
        Ok(res) => res,
        Err(err) => {
            return Err(ResponseBodyError {
                status: Status::Unauthorized,
                message: json!({"message":"unautorized"}),
            })
        }
    };

    //test request
    let gh = github::github_api::Gh::new(&access_token.access_token);
    let gh_user = gh.user(req_client).unwrap();
    info!("github user: {:?}", gh_user);

    let user = user::user::User::new(&gh_user.login, &gh_user.name);
    let create_res = user.create_user_if_not_exist(&conn, &access_token.access_token);
    match create_res {
        Ok(_) => (),
        Err(err) => {
            log::info!("{}", err);
            return Err(ResponseBodyError {
                status: Status::Unauthorized,
                message: json!({"message":"unautorized"}),
            });
        }
    };
    let auth_cookie = Cookie::build("authorization", access_token.access_token)
        .same_site(SameSite::None)
        .path("/v0")
        .secure(true)
        .http_only(true)
        .finish();
    cookies.add_private(auth_cookie);
    Ok(Response::build().status(Status::Ok).finalize())
}
fn main() {
    env_logger::init();
    log::info!("[root]");
    let cfg = Config::init_from_env().unwrap();

    let migration_result = lib::migrate(&cfg.database_url);
    match migration_result {
        Ok(_) => (),
        Err(err) => return log::error!("{}", err),
    }

    let api = Api {
        client_id: cfg.client_id.clone(),
        client_secret: cfg.client_secret.clone(),
        shared_key: cfg.shared_key.clone(),
    };

    let cors = rocket_cors::CorsOptions {
        allowed_origins: AllowedOrigins::some_exact(&["http://localhost:5000"]),
        allowed_headers: AllowedHeaders::some(&[
            "Authorization",
            "Accept",
            "Set-Cookie",
            "Content-Type",
        ]),
        allow_credentials: true,
        ..Default::default()
    }
    .to_cors()
    .expect("error creating CORS fairing");
    rocket::ignite()
        .manage(api)
        .attach(cors)
        .attach(MyPgDatabase::fairing())
        .attach(middleware::middleware::Middleware::new())
        .mount("/v0", routes![github_login, get_user])
        .launch();
}
