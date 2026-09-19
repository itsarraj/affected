use std::path::PathBuf;

use affected::{discover, git, matcher};
use clap::Parser;
use serde::Serialize;

#[derive(Parser)]
#[command(
    name = "affected",
    about = "Given a git diff, which projects in this monorepo actually changed"
)]
struct Cli {
    /// Diff range, e.g. `main..HEAD` or `main...HEAD` (merge-base form,
    /// usually what you want in CI) or a single ref for working-tree
    /// changes against it.
    #[arg(long, default_value = "HEAD")]
    range: String,
    #[arg(long, default_value = ".")]
    repo: PathBuf,
    #[arg(long)]
    json: bool,
}

#[derive(Serialize)]
struct Output {
    projects: Vec<String>,
    unmatched: Vec<String>,
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    let all_files = git::ls_files(&cli.repo)?;
    let roots = discover::find_project_roots(&all_files);
    let changed = git::changed_files(&cli.repo, &cli.range)?;
    let result = matcher::map_changed_to_projects(&changed, &roots);

    if cli.json {
        let out = Output {
            projects: result.projects.into_iter().collect(),
            unmatched: result.unmatched,
        };
        println!("{}", serde_json::to_string_pretty(&out)?);
    } else {
        for project in &result.projects {
            println!("{project}");
        }
        if !result.unmatched.is_empty() {
            eprintln!(
                "# {} changed file(s) outside any known project:",
                result.unmatched.len()
            );
            for f in &result.unmatched {
                eprintln!("#   {f}");
            }
        }
    }
    Ok(())
}
