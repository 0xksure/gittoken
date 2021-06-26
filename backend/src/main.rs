#![feature(proc_macro_hygiene, decl_macro)]

#[macro_use]
extern crate rocket_contrib;

#[macro_use]
extern crate rocket;

use envconfig::Envconfig;
use log::info;
use reqwest::header::ACCEPT;
use reqwest::{self, redirect};
use rithub::api::api;
use rithub::app::app;
use rithub::headers::rocket_request_headers;
use rithub::webhook::webhook;
use rocket::http::{ContentType, Cookie, Cookies, SameSite, Status};
use rocket::request::{FromRequest, Outcome, Request};
use rocket::response::{self, Responder, Response};
use rocket::State;
use serde::{Deserialize, Serialize};
use std::{borrow::Borrow, io::prelude::*};
mod handlers;
pub mod sdk;
use openssl::pkey::PKey;
use rocket_contrib::json::{Json, JsonValue};
mod lib;
use lib::web_error::WebError::WebError;
mod middleware;
mod user;
use rocket_cors::{AllowedHeaders, AllowedOrigins};
use std::fs::File;
extern crate authorization;
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
fn empty_string() -> String {
    "".to_string()
}

fn empty_usize() -> usize {
    0
}

#[derive(Debug, Deserialize)]
struct AccessTokenResponse {
    #[serde(default = "empty_string")]
    access_token: String,
    #[serde(default = "empty_usize")]
    expires_in: usize,
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
    api: State<handlers::Api>,
    conn: handlers::MyPgDatabase,
    authorization_header: AuthorzationHeader,
) -> Result<Json<api::User>, ResponseBodyError> {
    let access_token = authorization_header.0;
    info!("Authorization header: {:?}", access_token);
    let github_client = api::Config::new(&access_token);
    let req_client = reqwest::blocking::Client::new();
    let user = match github_client.user(req_client) {
        Ok(res) => res,
        Err(err) => {
            return Err(ResponseBodyError {
                status: Status::Unauthorized,
                message: json!({ "message": format!("{:?}", err) }),
            })
        }
    };

    Ok(Json(user))
}

#[derive(Deserialize, Debug)]
struct TransferData {
    from: String,
    to: String,
    amount: u64,
}
#[post("/github/app/send_token", data = "<transfer_data>")]
fn github_app_send_tokens(
    api: State<handlers::Api>,
    transfer_data: Json<TransferData>,
) -> Result<rocket::Response, ResponseBodyError> {
    match sdk::transfer_token(&transfer_data.from, &transfer_data.to, transfer_data.amount) {
        Ok(res) => Ok(Response::build().status(Status::Ok).finalize()),
        Err(err) => Err(ResponseBodyError {
            status: Status::InternalServerError,
            message: json!({ "message": "not able to send tokens" }),
        }),
    }
}

#[get("/github/app/post/status")]
fn github_app_post_status(
    api: State<handlers::Api>,
) -> Result<rocket::Response, ResponseBodyError> {
    info!("github_app_post_status");
    let installation_id = "31";
    let access_token = match api
        .github_app_client
        .authenticate_app(installation_id.to_string())
    {
        Ok(token) => token,
        Err(err) => {
            return Err(ResponseBodyError {
                status: Status::InternalServerError,
                message: json!({ "message": format!("{}", err) }),
            })
        }
    };

    info!("Access token: {}", access_token);
    Ok(Response::build().status(Status::Ok).finalize())
}
// Webhook responsible for
// - If PR, calculate potential cost
#[post("/github/webhook", data = "<webhook_data>")]
fn github_webhook(
    api: State<handlers::Api>,
    db: handlers::MyPgDatabase,
    webhook_data: Json<webhook::WebhookRequest>,
    github_headers: rocket_request_headers::GithubWebhookHeaders,
) {
    info!(
        "github_webhook.request: {}. Body: {:?}",
        "github webhook request", webhook_data
    );
    let data = webhook_data.into_inner();
    // Created pull request
    let request_type = webhook::WebhookRequest::get_webhook_type(github_headers, &data);
    info!("github.webhook.request_type {:?} ", request_type);
    match request_type {
        webhook::WebhookType::Open => {
            info!("github.webhook.pull_request.open");
            match handlers::pull_request(&data, api.inner()) {
                Ok(_) => info!("github.webhook.pull_request.success"),
                Err(_) => info!("github.webhook.pull_request.fail"),
            }
        }
        webhook::WebhookType::Review => {
            info!("github.webhook.pull_request_review.review");
            match handlers::pull_request_review(&data, api.inner()) {
                Ok(_) => info!("github.webhook.pull_request_review.success"),
                Err(_) => info!("github.webhook.pull_request_review.fail"),
            }
        }
        webhook::WebhookType::Approved => {
            info!("github.webhook.pull_request_review");
        }
        webhook::WebhookType::Merged => {
            info!("github.webhook.pull_request_review.merged");
            match handlers::merge_pull_request(&data, api.inner(), &db) {
                Ok(_) => info!("github.webhook.pull_request.success"),
                Err(_) => info!("github.webhook.pull_request.fail"),
            }
        }
        webhook::WebhookType::Closed => return,
        webhook::WebhookType::Unknown => return,
    }

    info!("github_webhook.finished");
}

#[get("/github/login?<code>")]
fn github_login<'a>(
    api: State<handlers::Api>,
    code: Option<String>,
    conn: handlers::MyPgDatabase,
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
        client_id: api.config.oauth_client_id.clone(),
        client_secret: api.config.oauth_client_secret.clone(),
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
    info!("response code: {}", resp_code);
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
    let gh = api::Config::new(&access_token.access_token);
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
    let cfg = handlers::Config::init_from_env().unwrap();

    let migration_result = lib::migrate(&cfg.database_url);
    match migration_result {
        Ok(_) => (),
        Err(err) => return log::error!("{}", err),
    }

    let mut pem_file = File::open(&cfg.cert_pem_path).unwrap();
    let mut buffer = Vec::new();
    match pem_file.read_to_end(&mut buffer) {
        Ok(_) => (),
        Err(err) => return log::error!("{}", err),
    }
    let buffer: Vec<u8> = buffer;
    let github_app_client = app::Config::new("114926".to_string(), buffer);
    let api = handlers::Api {
        config: cfg,
        github_app_client: github_app_client,
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
        .attach(handlers::MyPgDatabase::fairing())
        .attach(middleware::middleware::Middleware::new())
        .mount(
            "/v0",
            routes![
                github_login,
                get_user,
                github_app_post_status,
                github_webhook,
            ],
        )
        .launch();
}
