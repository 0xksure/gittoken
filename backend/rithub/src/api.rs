pub mod api {
    use error::errors::Error;
    use reqwest;
    use reqwest::header::ACCEPT;
    use serde::{Deserialize, Serialize};
    #[derive(Debug, Clone)]
    pub struct Config {
        access_token: String,
    }

    fn empty_string() -> String {
        "".to_string()
    }
    #[derive(Debug, Deserialize, Serialize)]
    pub struct User {
        #[serde(rename(serialize = "username"))]
        pub login: String,
        pub name: String,
    }

    #[derive(Debug, Deserialize, Serialize)]
    pub struct ReviewComment {
        pub body: String,
        pub position: usize,
        pub original_position: usize,
        pub user: User,
    }

    #[derive(Debug, Serialize, Deserialize)]
    struct IssueComment {
        body: String,
    }

    impl Config {
        pub fn new(access_token: &str) -> Config {
            Config {
                access_token: access_token.to_string(),
            }
        }

        fn get_authorization_header(&self) -> String {
            format!("token {}", self.access_token.clone())
        }

        pub fn user(&self, client: reqwest::blocking::Client) -> Result<User, Error> {
            let authorization_header = self.get_authorization_header();
            let req = client
                .get("https://api.github.com/user")
                .header(reqwest::header::AUTHORIZATION, authorization_header)
                .header(reqwest::header::USER_AGENT, "request");

            let resp = match req.send() {
                Ok(r) => r,
                Err(err) => return Err(Error::new(501, err.to_string())),
            };
            if !(resp.status() == reqwest::StatusCode::OK) {
                return Err(Error::new(501, String::from("failed to get user. Not OK")));
            }
            let gh_user = resp.json::<User>().unwrap();

            Ok(gh_user)
        }

        pub fn authentiate(&self, client: reqwest::blocking::Client) -> Result<(), Error> {
            let authorization_header = self.get_authorization_header();
            let req = client
                .get("https://api.github.com/")
                .header(reqwest::header::AUTHORIZATION, authorization_header)
                .header(reqwest::header::USER_AGENT, "request");

            let resp = match req.send() {
                Ok(r) => r,
                Err(err) => return Err(Error::new(501, err.to_string())),
            };
            Ok(())
        }

        pub fn comment_issue(
            &self,
            webhook_data: &webhook::webhook::WebhookRequest,
            message: &str,
        ) -> Result<(), Error> {
            let authorization_header = self.get_authorization_header();
            let issue_path = format!(
                "repos/{}/{}/issues/{}/comments",
                webhook_data.repository.owner.login,
                webhook_data.repository.name,
                webhook_data.pull_request.number
            );
            let url = format!("https://api.github.com/{}", issue_path);
            let client = reqwest::blocking::Client::new();
            let issue_comment = IssueComment {
                body: message.to_string(),
            };
            let res = match client
                .post(url)
                .header(reqwest::header::AUTHORIZATION, authorization_header)
                .header(reqwest::header::USER_AGENT, "request")
                .header(ACCEPT, "application/json")
                .json(&issue_comment)
                .send()
            {
                Ok(res) => res,
                Err(err) => return Err(Error::new(501, err.to_string())),
            };

            if !res.status().is_success() {
                return Err(Error::new(501, String::from("failed to comment issue")));
            }
            return Ok(());
        }

        pub fn list_review_comments(
            &self,
            webhook_data: &webhook::webhook::WebhookRequest,
        ) -> Result<Vec<ReviewComment>, Error> {
            let authorization_header = self.get_authorization_header();
            let issue_path = format!(
                "repos/{}/{}/pulls/{}/reviews/{}/comments",
                webhook_data.repository.owner.login,
                webhook_data.repository.name,
                webhook_data.pull_request.number,
                webhook_data.review.id
            );
            let url = format!("https://api.github.com/{}", issue_path);
            let client = reqwest::blocking::Client::new();

            let res = match client
                .get(url)
                .header(reqwest::header::AUTHORIZATION, authorization_header)
                .header(reqwest::header::USER_AGENT, "request")
                .header(ACCEPT, "application/json")
                .send()
            {
                Ok(res) => res,
                Err(err) => return Err(Error::new(501, err.to_string())),
            };

            if !res.status().is_success() {
                return Err(Error::new(
                    501,
                    String::from("failed to list review comments issue"),
                ));
            }

            let review_comments = res.text().unwrap();
            let review_comments_vec: Vec<ReviewComment> =
                serde_json::from_str(&review_comments).unwrap();

            Ok(review_comments_vec)
        }
    }
}
