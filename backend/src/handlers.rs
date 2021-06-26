use crate::sdk;
use crate::user::user::User;
use envconfig::Envconfig;
use log::info;
use rithub::api::api::{self, ReviewComment};
use rithub::app::app;
use rithub::error::errors::Error;
use rithub::webhook::webhook::WebhookRequest;
use rocket_contrib::database;
use rocket_contrib::databases::postgres;
use solana_client::rpc_client::RpcClient;
use solana_sdk::instruction::AccountMeta;
use solana_sdk::{
    commitment_config::CommitmentConfig,
    instruction::Instruction,
    message::Message,
    native_token::*,
    program_option::COption,
    program_pack::Pack,
    pubkey::Pubkey,
    signature::{Keypair, Signer},
    system_instruction, system_program,
    transaction::Transaction,
};
use std::borrow::Borrow;
use std::fs::File;
use std::io::{BufRead, BufReader};
#[database("my_db")]
pub struct MyPgDatabase(postgres::Connection);

#[derive(Envconfig)]
pub struct Config {
    #[envconfig(from = "GITHUB_OAUTH_CLIENT_ID")]
    pub oauth_client_id: String,
    #[envconfig(from = "GITHUB_OAUTH_SECRET")]
    pub oauth_client_secret: String,
    #[envconfig(from = "GITHUB_APP_CLIENT_ID")]
    pub app_client_id: String,
    #[envconfig(from = "GITHUB_APP_SECRET")]
    pub app_client_secret: String,
    #[envconfig(from = "DATABASE_URL")]
    pub database_url: String,
    #[envconfig(from = "SHARED_KEY")]
    pub shared_key: String,
    #[envconfig(from = "SECRET_TOKEN")]
    pub secret_token: String,
    #[envconfig(from = "CERT_PEM_PATH")]
    pub cert_pem_path: String,
}

pub struct Api {
    pub config: Config,
    pub github_app_client: app::Config,
}

fn scaled_sigmoid(scale_val: f64, val: f64) -> f64 {
    1.0 / (1.0 + (val / scale_val).exp())
}

fn calculate_pull_request_score(webhook_data: &WebhookRequest) -> String {
    let pr_score = 1.2 * webhook_data.pull_request.additions as f64
        + 0.8 * webhook_data.pull_request.deletions as f64
        + 1.1 * webhook_data.pull_request.changed_files as f64;
    let pr_score_normalized = scaled_sigmoid(100.0, pr_score);
    let pr_score_comment = format!(
                ":unicorn: **Total Reward** : {:} OCT (open contribution tokens). [Access your OCTs](http://localhost:5000/)",
                pr_score_normalized
            );
    pr_score_comment
}

// pull_request calculates the value of the pull request and posts the
// estimated value to the pull request as a comment
pub fn pull_request(webhook_data: &WebhookRequest, api: &Api) -> Result<(), Error> {
    info!("github_webhook.type.pull_request");
    let access_token = match api
        .github_app_client
        .authenticate_app(webhook_data.installation.id.to_string())
    {
        Ok(token) => token,
        Err(err) => {
            log::error!("github_webhook.type.pull_fail. Cause: {:}", err);
            return Err(Error::new(500, err.to_string()));
        }
    };

    // Calculate Pull request score
    let pr_score_comment = calculate_pull_request_score(&webhook_data);
    let github_client = api::Config::new(&access_token.token);

    match github_client.comment_issue(&webhook_data, &pr_score_comment) {
        Ok(res) => res,
        Err(err) => {
            log::error!("error: {:?}", err);
            return Err(Error::new(500, err.message));
        }
    };
    Ok(())
}

fn review_score_for_user(comments: &Vec<ReviewComment>, user: &str) -> String {
    let num_comments = comments.len();
    let comment_lengths: usize = comments
        .into_iter()
        .filter(|x| x.user.login == user)
        .map(|c| c.body.chars().count())
        .sum();
    let abs_score = (num_comments + comment_lengths / 30) as f64;
    let scaled_score = scaled_sigmoid(100.0, abs_score);
    let review_score_comment =  format!(
        ":unicorn: **Pull Request Value** : The minimal value of your PR is {:} OCT (open contribution tokens). If approved your OCTs will be accessible in your wallet. [Access your OCTs](http://localhost:5000/)",
        scaled_score
    );
    review_score_comment
}
// pull_request_review is called when a user submits a pull request review
// is either submitted, edited or dismissed
pub fn pull_request_review(webhook_data: &WebhookRequest, api: &Api) -> Result<(), Error> {
    info!("github_webhook.type.pull_request_review");
    let access_token = match api
        .github_app_client
        .authenticate_app(webhook_data.installation.id.to_string())
    {
        Ok(token) => token,
        Err(err) => {
            log::error!(
                "github_webhook.type.pull_request_review.fail. Cause: {:}",
                err
            );
            return Err(Error::new(500, err.to_string()));
        }
    };

    // Get review comments
    let github_client = api::Config::new(&access_token.token);
    let review_comments = match github_client.list_review_comments(&webhook_data) {
        Ok(res) => res,
        Err(err) => {
            log::error!(
                "github_webhook.type.pull_request_review.fail. Cause: {:?}",
                err
            );
            return Err(Error::new(500, err.message));
        }
    };
    info!("review_comments {:?}", review_comments);

    // Calculate review score
    let username = webhook_data.review.user.login.as_str();
    let review_score_comment = review_score_for_user(&review_comments, username);
    let review_score_comment_atuser = format!("@{}: {}", username, review_score_comment);

    // comment review score
    match github_client.comment_issue(&webhook_data, &review_score_comment_atuser) {
        Ok(res) => res,
        Err(err) => {
            log::error!("error: {:?}", err);
            return Err(Error::new(500, err.message));
        }
    };

    Ok(())
}

// approved_pull_request recalculates the pull request value of
// all requests and comments made by the user
// This is posted on the pull request and on merge commit the
// OCT is transferred to the users wallet
pub fn approved_pull_request(webhook_data: &WebhookRequest, api: &Api) -> Result<(), Error> {
    Ok(())
}

// merge_pull_request makes sure the reviewers receieves their tokens
pub fn merge_pull_request(
    webhook_data: &WebhookRequest,
    api: &Api,
    db: &MyPgDatabase,
) -> Result<(), Error> {
    info!("github_webhook.type.pull_request_review");
    let access_token = match api
        .github_app_client
        .authenticate_app(webhook_data.installation.id.to_string())
    {
        Ok(token) => token,
        Err(err) => {
            log::error!(
                "github_webhook.type.pull_request_review.fail. Cause: {:}",
                err
            );
            return Err(Error::new(500, err.to_string()));
        }
    };

    // get all reviews comments
    let github_client = api::Config::new(&access_token.token);
    let review_comments = match github_client.list_review_comments(&webhook_data) {
        Ok(res) => res,
        Err(err) => {
            log::error!(
                "github_webhook.type.pull_request_review.fail. Cause: {:?}",
                err
            );
            return Err(Error::new(500, err.message));
        }
    };
    info!("review_comments {:?}", review_comments);
    let review_comments_ = &review_comments;
    // get all reviewers
    // Vec
    let mut users: Vec<&str> = review_comments_
        .into_iter()
        .filter(|comment| comment.user.login != webhook_data.pull_request.user.login)
        .map(|comment| comment.user.login.as_str())
        .collect();
    users.sort();
    users.dedup();

    let owner = &webhook_data.pull_request.user.login;
    let owner_user = User::new(&owner, &owner);
    let owner_addr = match owner_user.get_address_from_username(&db) {
        Ok(addr) => addr,
        Err(err) => {
            log::error!("error: {:?}", err);
            return Err(Error::new(500, String::from("oh no")));
        }
    };

    for username in users {
        let review_score_comment = review_score_for_user(review_comments_, &username);
        let review_score_comment_atuser = format!("@{}: {}", username, review_score_comment);

        match github_client.comment_issue(&webhook_data, &review_score_comment_atuser) {
            Ok(res) => res,
            Err(err) => {
                log::error!("error: {:?}", err);
                return Err(Error::new(500, err.message));
            }
        };

        // Transfer money to wallets from PR owner
        // if not present then post fail message
        let user = User::new(&username, &username);
        let addr = match user.get_address_from_username(&db) {
            Ok(addr) => addr,
            Err(err) => {
                log::error!("error: {:?}", err);
                return Err(Error::new(500, String::from("oh no")));
            }
        };

        match sdk::transfer_token(&owner_addr, &addr, 10) {
            Ok(res) => res,
            Err(err) => return Err(err),
        };
    }
    Ok(())
}
