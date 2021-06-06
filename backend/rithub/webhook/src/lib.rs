pub mod webhook {
    use std::fmt::Display;

    use error::errors;
    use headers::rocket_request_headers::GithubWebhookHeaders;
    use serde::Deserialize;

    fn empty_string() -> String {
        "".to_string()
    }

    fn empty_usize() -> usize {
        0
    }

    #[derive(Deserialize, Debug)]
    pub struct Installation {
        pub id: usize,
        node_id: String,
    }

    #[derive(Deserialize, Debug)]
    pub struct PullRequest {
        #[serde(default = "empty_string")]
        url: String,
        #[serde(default = "empty_usize")]
        id: usize,
        #[serde(default = "empty_usize")]
        commits: usize,
        #[serde(default = "empty_usize")]
        pub additions: usize,
        #[serde(default = "empty_usize")]
        pub deletions: usize,
        #[serde(default = "empty_usize")]
        pub changed_files: usize,
        #[serde(default = "empty_usize")]
        pub number: usize,
        pub user: User,
        #[serde(default = "empty_string")]
        pub state: String,
        #[serde(default = "empty_string")]
        pub created_at: String,
        #[serde(default = "empty_string")]
        pub updated_at: String,
        #[serde(default = "empty_string")]
        pub closed_at: String,
        #[serde(default = "empty_string")]
        pub merged_at: String,
    }

    fn empty_pull_request() -> PullRequest {
        PullRequest {
            url: String::from(""),
            id: 0,
            commits: 0,
            additions: 0,
            deletions: 0,
            changed_files: 0,
            number: 0,
            user: empty_user(),
            state: String::from(""),
            created_at: String::from(""),
            updated_at: String::from(""),
            closed_at: String::from(""),
            merged_at: String::from(""),
        }
    }

    #[derive(Deserialize, Debug)]
    pub struct User {
        pub login: String,
        id: usize,
        node_id: String,
        url: String,
        #[serde(rename(deserialize = "type"))]
        repo_owner_type: String,
        site_admin: bool,
    }

    fn empty_user() -> User {
        User {
            login: String::from(""),
            id: 0,
            node_id: String::from(""),
            url: String::from(""),
            repo_owner_type: String::from(""),
            site_admin: false,
        }
    }

    #[derive(Deserialize, Debug)]
    pub struct Repository {
        id: usize,
        node_id: String,
        pub name: String,
        full_name: String,
        private: bool,
        pub owner: User,
    }

    #[derive(Deserialize, Debug)]
    pub struct Review {
        pub id: usize,
        node_id: String,
        pub user: User,
        body: String,
        submitted_at: String,
        pub state: String,
    }

    fn empty_review() -> Review {
        Review {
            id: 0,
            node_id: String::from(""),
            user: empty_user(),
            body: String::from(""),
            submitted_at: String::from(""),
            state: String::from(""),
        }
    }

    #[derive(Deserialize, Debug)]
    pub struct WebhookRequest {
        pub action: String,
        #[serde(default = "empty_pull_request")]
        pub pull_request: PullRequest,
        pub installation: Installation,
        pub repository: Repository,
        #[serde(default = "empty_review")]
        pub review: Review,
    }

    #[derive(Debug)]
    pub enum WebhookType {
        Open,
        Review,
        Approved,
        Closed,
        Merged,
        Unknown,
    }

    impl WebhookRequest {
        pub fn get_webhook_type(
            header: GithubWebhookHeaders,
            data: &WebhookRequest,
        ) -> WebhookType {
            match header.event.as_str() {
                "pull_request" => match data.pull_request.state.as_str() {
                    "open" => return WebhookType::Open,
                    "closed" => {
                        if data.pull_request.merged_at != "" {
                            return WebhookType::Merged;
                        }
                        return WebhookType::Closed;
                    }
                    _ => return WebhookType::Unknown,
                },
                "pull_request_review" => {
                    match data.review.state.as_str() {
                        "approved" => return WebhookType::Approved,
                        "changes_requested" => return WebhookType::Review,
                        _ => return WebhookType::Unknown,
                    };
                }
                _ => return WebhookType::Unknown,
            }
        }

        // pull_request handles cases of pull_request
        fn pull_request(&self, f: fn() -> Result<(), errors::Error>) {}
    }
}
