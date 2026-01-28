use reqwest::Client;
use serde::{Deserialize, Serialize};

const ALLOWED_EXTENSIONS: &[&str] = &["pdf", "py", "cpp", "pptx", "ppt", "c", "h", "java", "js", "ts", "md", "tex"];
const IGNORED_PATHS: &[&str] = &[".git", "node_modules", "__pycache__", ".venv", "target", "dist", "build"];

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct GitHubTreeEntry {
    pub path: String,
    #[serde(rename = "type")]
    pub entry_type: String,
    pub sha: String,
}

#[derive(Debug, Deserialize)]
struct GitHubTreeResponse {
    tree: Vec<GitHubTreeEntry>,
}

#[derive(Debug, Deserialize)]
struct GitHubRepoInfo {
    default_branch: String,
}

#[derive(Debug, thiserror::Error)]
pub enum GitHubError {
    #[error("HTTP error: {0}")]
    Http(#[from] reqwest::Error),
    #[error("Repository not found or inaccessible")]
    NotFound,
    #[error("Rate limited")]
    RateLimited,
}

pub struct GitHubClient {
    client: Client,
}

impl GitHubClient {
    pub fn new() -> Self {
        Self {
            client: Client::new(),
        }
    }

    async fn request(&self, url: &str) -> Result<reqwest::Response, GitHubError> {
        let response = self
            .client
            .get(url)
            .header("User-Agent", "cwdb-crawler")
            .header("Accept", "application/vnd.github.v3+json")
            .send()
            .await?;

        match response.status().as_u16() {
            404 => Err(GitHubError::NotFound),
            403 => Err(GitHubError::RateLimited),
            _ => Ok(response.error_for_status()?),
        }
    }

    pub async fn get_default_branch(&self, owner: &str, repo: &str) -> Result<String, GitHubError> {
        let url = format!("https://api.github.com/repos/{}/{}", owner, repo);
        let info: GitHubRepoInfo = self.request(&url).await?.json().await?;
        Ok(info.default_branch)
    }

    pub async fn get_repo_tree(
        &self,
        owner: &str,
        repo: &str,
        branch: Option<&str>,
    ) -> Result<(String, Vec<GitHubTreeEntry>), GitHubError> {
        let branch = match branch {
            Some(b) => b.to_string(),
            None => self.get_default_branch(owner, repo).await?,
        };

        let url = format!(
            "https://api.github.com/repos/{}/{}/git/trees/{}?recursive=1",
            owner, repo, branch
        );

        let response: GitHubTreeResponse = self.request(&url).await?.json().await?;
        let filtered = filter_tree_entries(response.tree);

        Ok((branch, filtered))
    }
}

fn filter_tree_entries(entries: Vec<GitHubTreeEntry>) -> Vec<GitHubTreeEntry> {
    entries
        .into_iter()
        .filter(|e| e.entry_type == "blob")
        .filter(|e| !is_ignored_path(&e.path))
        .filter(|e| has_allowed_extension(&e.path))
        .collect()
}

fn is_ignored_path(path: &str) -> bool {
    IGNORED_PATHS.iter().any(|ignored| path.contains(ignored))
}

fn has_allowed_extension(path: &str) -> bool {
    path.rsplit('.')
        .next()
        .map(|ext| ALLOWED_EXTENSIONS.contains(&ext.to_lowercase().as_str()))
        .unwrap_or(false)
}
