use clap::Parser;
use git2::{BranchType, DiffOptions, Repository, Sort};
use indicatif::ProgressBar;
use std::{collections::HashMap, path::PathBuf};

#[derive(Parser)]
struct Args {
    repo_path: PathBuf,

    #[arg(long, default_value = "main")]
    branch: String,
}

#[derive(Debug)]
struct LineDelta {
    added: usize,
    deleted: usize,
}

fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    let repo = Repository::open(args.repo_path)?;

    let branch = repo.find_branch(&args.branch, BranchType::Local)?;
    let commit = branch.get().peel_to_commit()?;
    let commit_id = commit.id();

    let mut revwalk = repo.revwalk()?;
    revwalk.push(commit_id)?;
    revwalk.set_sorting(Sort::TIME | Sort::REVERSE)?;

    let commits: Vec<_> = revwalk.collect();
    let total_commits = commits.len();
    let progress_bar = ProgressBar::new(total_commits as u64);

    let mut stats: HashMap<PathBuf, LineDelta> = HashMap::new();

    for commit_id in commits {
        let commit_id = commit_id?;
        let commit = repo.find_commit(commit_id)?;

        progress_bar.inc(1);

        if commit.parent_count() != 1 {
            continue;
        }

        let parent = commit.parent(0)?;

        let mut diff_options = DiffOptions::new();
        diff_options.context_lines(0);

        let diff = repo.diff_tree_to_tree(
            Some(&parent.tree()?),
            Some(&commit.tree()?),
            Some(&mut diff_options),
        )?;

        diff.foreach(
            &mut |_delta, _progress| true,
            None,
            None,
            Some(&mut |delta, _hunk, line| {
                let path = delta
                    .new_file()
                    .path()
                    .or_else(|| delta.old_file().path())
                    .unwrap()
                    .to_path_buf();

                let line_delta = stats.entry(path).or_insert(LineDelta {
                    added: 0,
                    deleted: 0,
                });

                match line.origin() {
                    '+' => line_delta.added += 1,
                    '-' => line_delta.deleted += 1,
                    _ => {}
                }

                true
            }),
        )?;
    }

    progress_bar.finish();

    let mut sorted_stats: Vec<_> = stats.into_iter().collect();

    sorted_stats.sort_by(|(a_path, a_delta), (b_path, b_delta)| {
        let churn_a = a_delta.added + a_delta.deleted;
        let churn_b = b_delta.added + b_delta.deleted;
        if churn_b == churn_a {
            a_path.cmp(b_path)
        } else {
            churn_b.cmp(&churn_a)
        }
    });

    for (path, delta) in sorted_stats {
        println!("{}: +{} -{}", path.display(), delta.added, delta.deleted);
    }

    Ok(())
}
