pub mod util;

use search::query::IncompleteQuery;
use types::{Notification, Repository};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Event {
    RepositoryFetched(Repository),
    RepositoryForked(Repository),
    ForkDeleted(Repository),
    RepositoryFormatted(Repository),
    PRCreated(Repository),
    Notification(Notification),
    PRStatusChange(Repository),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum GithubRequest {
    Fetch(IncompleteQuery),
    Fork(Repository),
    DeleteFork(Repository),
    CreatePR {
        repo: Repository,
        branch: String,
        title: String,
        message: String,
    },
    FetchNotifications,
    CheckPRStatus(Repository),
}

pub mod topic {
    pub const GITHUB_REQUEST: &str = "rustyrobot.github.request";
    pub const EVENT: &str = "rustyrobot.event";
    pub const GITHUB_STATE: &str = "rustyrobot.github.state";
    pub const FETCHER_STATE: &str = "rustyrobot.fetcher.state";
}

pub mod group {
    pub const GITHUB: &str = "rustyrobot.github";
    pub const FETCHER: &str = "rustyrobot.fetcher";
    pub const FORKER: &str = "rustyrobot.forker";
    pub const FORMATTER: &str = "rustyrobot.formatter";
    pub const PR_ISSUER: &str = "rustyrobot.prissuer";
}
