use std::fs::{self, File};
use std::io::{Read};
use std::process::Command;
use tempfile::TempDir;
use url::Url;
use std::error::Error;
use std::fmt;

#[derive(Debug)]
pub enum GitHubFetchError {
    IoError(std::io::Error),
    UrlParseError(url::ParseError),
    GitCloneError(String),
    InvalidUrlError(String),
}

impl fmt::Display for GitHubFetchError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            GitHubFetchError::IoError(e) => write!(f, "IO error: {}", e),
            GitHubFetchError::UrlParseError(e) => write!(f, "URL parse error: {}", e),
            GitHubFetchError::GitCloneError(s) => write!(f, "Git clone error: {}", s),
            GitHubFetchError::InvalidUrlError(s) => write!(f, "Invalid URL error: {}", s),
        }
    }
}

impl Error for GitHubFetchError {}

impl From<std::io::Error> for GitHubFetchError {
    fn from(error: std::io::Error) -> Self {
        GitHubFetchError::IoError(error)
    }
}

impl From<url::ParseError> for GitHubFetchError {
    fn from(error: url::ParseError) -> Self {
        GitHubFetchError::UrlParseError(error)
    }
}

pub struct Args {
    pub url: String,
    pub timeout: u32,
}

pub fn main(args: Args) -> Result<String, GitHubFetchError> {
    let (repo_url, folder) = parse_github_url(&args.url)?;

    let temp_dir = TempDir::new()?;
    let temp_path = temp_dir.path();
    dbg!(&repo_url);

    // Check for available branches
    let branches = get_available_branches(&repo_url)?;
    let branch = if branches.contains(&"main".to_string()) {
        "main"
    } else if branches.contains(&"master".to_string()) {
        "master"
    } else {
        return Err(GitHubFetchError::GitCloneError(
            "Neither 'main' nor 'master' branch found".to_string()
        ));
    };

    // Clone the repository with the specified timeout and the detected branch
    let status = Command::new("timeout")
        .arg(args.timeout.to_string())
        .arg("git")
        .arg("clone")
        .arg("--depth")
        .arg("1")
        .arg("--single-branch")
        .arg("--branch")
        .arg(branch)
        .arg(&repo_url)
        .arg(temp_path)
        .status()?;

    if !status.success() {
        dbg!(&status);
        return Err(GitHubFetchError::GitCloneError(
            format!("Git clone timed out after {} seconds or failed.", args.timeout)
        ));
    }

    let target_dir = temp_path.join(&folder);
    if !target_dir.exists() {
        return Err(GitHubFetchError::InvalidUrlError(
            format!("Folder {} not found in the repository.", folder)
        ));
    }

    let mut contents = String::new();

    for entry in fs::read_dir(target_dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_file() {
            let file_name = path.file_name().unwrap().to_string_lossy();
            contents.push_str(&format!("File: {}/{}\n", folder, file_name));
            let mut file_content = String::new();
            File::open(&path)?.read_to_string(&mut file_content)?;
            contents.push_str(&file_content);
            contents.push_str("\n\n");
        }
    }

    Ok(contents)
}

fn get_available_branches(repo_url: &str) -> Result<Vec<String>, GitHubFetchError> {
    let output = Command::new("git")
        .arg("ls-remote")
        .arg("--heads")
        .arg(repo_url)
        .output()?;

    if !output.status.success() {
        return Err(GitHubFetchError::GitCloneError(
            "Failed to fetch remote branches".to_string()
        ));
    }

    let output_str = String::from_utf8_lossy(&output.stdout);
    let branches: Vec<String> = output_str
        .lines()
        .filter_map(|line| {
            line.split_whitespace().nth(1).and_then(|ref_path| {
                ref_path.strip_prefix("refs/heads/").map(|s| s.to_string())
            })
        })
        .collect();

    Ok(branches)
}

fn parse_github_url(url: &str) -> Result<(String, String), GitHubFetchError> {
    let parsed_url = Url::parse(url)?;
    let path_segments: Vec<&str> = parsed_url.path_segments().ok_or_else(||
    GitHubFetchError::InvalidUrlError("Invalid URL".to_string())
    )?.collect();
    dbg!(&path_segments);

    let repo_url = format!("https://github.com/{}/{}.git", path_segments[0], path_segments[1]);
    let folder = path_segments[2..].join("/");

    Ok((repo_url, folder))
}