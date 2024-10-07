use std::fs::{self, File};
use std::io::{Write, Read};
use std::path::PathBuf;
use std::process::Command;
use tempfile::TempDir;
use url::Url;

pub struct Args {
    pub url: String,
    pub timeout: u32,
    pub output_dir: PathBuf,
}

pub fn main(args: Args) -> Result<String, Box<dyn std::error::Error>> {
    let (repo_url, folder) = parse_github_url(&args.url)?;

    let temp_dir = TempDir::new()?;
    let temp_path = temp_dir.path();

    // Clone the repository with the specified timeout
    let status = Command::new("timeout")
        .arg(args.timeout.to_string())
        .arg("git")
        .arg("clone")
        .arg("--depth")
        .arg("1")
        .arg("--single-branch")
        .arg("--branch")
        .arg("main")
        .arg(&repo_url)
        .arg(temp_path)
        .status()?;

    if !status.success() {
        return Err(format!("Error: Git clone timed out after {} seconds or failed.", args.timeout).into());
    }

    let target_dir = temp_path.join(&folder);
    if !target_dir.exists() {
        return Err(format!("Error: Folder {} not found in the repository.", folder).into());
    }

    // Create the output directory if it doesn't exist
    fs::create_dir_all(&args.output_dir)?;

    let repo_name = repo_url.split('/').last().unwrap_or("repo").trim_end_matches(".git");
    let output_file_name = format!("{}_{}.txt", repo_name, folder.replace('/', "_"));
    let output_file_path = args.output_dir.join(output_file_name);
    let mut output_file = File::create(&output_file_path)?;

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

    // Write the contents to the output file
    write!(output_file, "{}", contents)?;

    Ok(contents)
}

fn parse_github_url(url: &str) -> Result<(String, String), Box<dyn std::error::Error>> {
    let parsed_url = Url::parse(url)?;
    let path_segments: Vec<&str> = parsed_url.path_segments().ok_or("Invalid URL")?.collect();

    if path_segments.len() < 5 {
        return Err("Invalid GitHub URL format".into());
    }

    let repo_url = format!("https://github.com/{}/{}.git", path_segments[0], path_segments[1]);
    let folder = path_segments[4..].join("/");

    Ok((repo_url, folder))
}