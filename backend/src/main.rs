#![feature(proc_macro_hygiene, decl_macro)]

#[macro_use]
extern crate rocket;
use log::info;
use reqwest;
use reqwest::header::ACCEPT;
use rocket::http::Method;
use rocket::response::Redirect;
use rocket::Request;
use rocket::State;
use serde::{Deserialize, Serialize};

use envconfig::Envconfig;
use std::env;

#[derive(Envconfig)]
struct Config {
    #[envconfig(from = "GITHUB_OAUTH_CLIENT_ID")]
    pub client_id: String,
    #[envconfig(from = "GITHUB_OAUTH_SECRET")]
    pub client_secret: String,
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
    refresh_token: String,
}

#[get("/github/callback?<code>")]
fn github_callback(cfg: State<Config>, code: Option<String>) -> Redirect {
    let _gcode = match code {
        Some(res) => res,
        None => return Redirect::to(format!("http://localhost:5000/login/error")),
    };
    info!("ok let's check who we are dealing with");

    let github_post = Oauth {
        client_id: cfg.client_id.to_string(),
        client_secret: cfg.client_secret.to_string(),
        code: _gcode,
    };
    let res = match reqwest::blocking::Client::new()
        .post("https://github.com/login/oauth/access_token")
        .header(ACCEPT, "application/json")
        .json(&github_post)
        .send()
    {
        Ok(res) => res,
        Err(err) => {
            info!("error when retrieving access token: {}", err.to_string());
            return Redirect::to(format!("http://localhost:5000/login/error"));
        }
    };

    let resp_code = res.status();
    if resp_code != reqwest::StatusCode::OK {
        info!("access token response code: {}", resp_code.to_string());
        return Redirect::to(format!("http://localhost:5000/login/error"));
    }
    let access_token_resp = res.json::<AccessTokenResponse>();
    info!("bytes: {:?}", access_token_resp);

    Redirect::to(format!("http://localhost:5000/login/success"))
}

fn main() {
    log::info!("[root] info");
    let cfg = Config::init_from_env().unwrap();
    rocket::ignite()
        .manage(cfg)
        .mount("/", routes![github_callback])
        .launch();
}
