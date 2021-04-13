#![feature(proc_macro_hygiene, decl_macro)]

#[macro_use]
extern crate rocket;
use log::info;
use reqwest;
use rocket::http::Method;
use rocket::response::Redirect;
use rocket::Request;

struct API {
    client_id: String,
    client_secret: String,
}

struct Oauth {
    client_id: String,
    client_secret: String,
    code: String,
}

#[get("/github/callback?<code>")]
fn github_callback(code: Option<String>) -> Redirect {
    let _gcode = match code {
        Some(res) => res,
        None => return Redirect::to(format!("http://localhost:5000/login/error")),
    };
    info!("ok let's check who we are dealing with");

    let client = reqwest::Client::new();
    let res = client
        .post("http://httpbin.org/post")
        .body("the exact body that is sent")
        .send()
        .await?;

    Redirect::to(format!("http://localhost:5000/login/success"))
}

fn main() {
    log::info!("[root] info");
    rocket::ignite()
        .mount("/", routes![github_callback])
        .launch();
}
