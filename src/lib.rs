use anyhow::Context;
use clap::Parser;
use serde::Deserialize;
use std::{
    fs,
    path::{Path, PathBuf},
};

#[derive(Debug, Deserialize)]
pub struct Branch(String);

impl Default for Branch {
    fn default() -> Self {
        Self("main".to_owned())
    }
}

#[derive(Debug, Default, Deserialize)]
pub struct Config {
    #[serde(default)]
    pub branch: Branch,

    #[serde(default)]
    pub pathspec: Vec<String>,

    #[serde(default)]
    pub repository_path: PathBuf,
}

#[derive(Debug, Parser)]
struct Args {
    repository_path: Option<PathBuf>,

    #[arg(long)]
    branch: Option<String>,

    #[arg(long)]
    pathspec: Vec<String>,
}

pub fn load_config() -> anyhow::Result<Config> {
    let args = Args::parse();

    let mut config = load_config_toml()?;

    if let Some(ref repository_path) = args.repository_path {
        config.repository_path = repository_path.clone();
    }

    if let Some(ref branch) = args.branch {
        config.branch = Branch(branch.clone());
    }

    if !args.pathspec.is_empty() {
        config.pathspec.extend(args.pathspec)
    }

    Ok(config)
}

fn load_config_toml() -> anyhow::Result<Config> {
    let config_path = Path::new("ikura.toml");

    if !config_path.exists() {
        println!("ikura.toml not found, using default configuration");
        return Ok(Config::default());
    }

    let toml_content = fs::read_to_string(config_path)?;

    toml::from_str::<Config>(&toml_content).with_context(|| "Failed to parse ikura.toml")
}

pub fn find_branch<'repo>(
    repo: &'repo git2::Repository,
    branch: &Branch,
) -> anyhow::Result<git2::Branch<'repo>> {
    repo.find_branch(&branch.0, git2::BranchType::Local)
        .with_context(|| format!("Failed to find the specified repository: {}", branch.0))
}
