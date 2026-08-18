//! List provenance checkpoints for the current workspace.

use crate::core::provenance::{CheckpointLinkState, ProvenanceTracker};

#[derive(Default)]
struct CheckpointOptions {
    session: Option<String>,
    commit: Option<String>,
    orphaned: bool,
}

pub(crate) fn cmd_checkpoints(args: &[String]) {
    if args.iter().any(|arg| arg == "--help" || arg == "-h") {
        print_help();
        return;
    }

    let options = match parse_options(args) {
        Ok(options) => options,
        Err(error) => {
            eprintln!("checkpoints: {error}");
            eprintln!("Run lean-ctx checkpoints --help for usage.");
            return;
        }
    };

    let project_root = match std::env::current_dir() {
        Ok(path) => path,
        Err(error) => {
            eprintln!("checkpoints: cannot determine workspace: {error}");
            return;
        }
    };
    let tracker = match ProvenanceTracker::new(&project_root) {
        Ok(tracker) => tracker,
        Err(error) => {
            eprintln!("checkpoints: cannot open provenance store: {error}");
            return;
        }
    };

    let mut checkpoints = tracker
        .store()
        .query_checkpoints(options.session.as_deref());
    checkpoints.retain(|checkpoint| {
        options
            .commit
            .as_deref()
            .is_none_or(|commit| checkpoint.commit_sha.starts_with(commit))
            && (!options.orphaned || matches!(checkpoint.link_state, CheckpointLinkState::Orphaned))
    });
    checkpoints.sort_by_key(|cp| std::cmp::Reverse(cp.observed_at));

    if checkpoints.is_empty() {
        println!("No checkpoints found.");
        return;
    }

    println!(
        "{:<18}  {:<18}  {:<12}  {:<10}  {:>5}  {:>9}  OBSERVED",
        "CHECKPOINT", "SESSION", "COMMIT", "STATE", "FILES", "DELTA"
    );
    println!("{}", "-".repeat(104));
    for checkpoint in checkpoints {
        let delta = format!("+{}/-{}", checkpoint.insertions, checkpoint.deletions);
        println!(
            "{:<18}  {:<18}  {:<12}  {:<10}  {:>5}  {:>9}  {}",
            shorten(&checkpoint.id, 18),
            shorten(&checkpoint.session_id, 18),
            shorten(&checkpoint.commit_sha, 12),
            format!("{:?}", checkpoint.link_state).to_uppercase(),
            checkpoint.files_touched,
            delta,
            checkpoint.observed_at.format("%Y-%m-%d %H:%M:%SZ"),
        );
    }
}

fn parse_options(args: &[String]) -> Result<CheckpointOptions, String> {
    let mut options = CheckpointOptions::default();
    let mut index = 0;

    while index < args.len() {
        match args[index].as_str() {
            "--session" => {
                index += 1;
                options.session = Some(value_after(args, index, "--session")?.to_owned());
            }
            "--commit" => {
                index += 1;
                options.commit = Some(value_after(args, index, "--commit")?.to_owned());
            }
            "--orphaned" => options.orphaned = true,
            argument => return Err(format!("unknown option '{argument}'")),
        }
        index += 1;
    }

    Ok(options)
}

fn value_after<'a>(args: &'a [String], index: usize, option: &str) -> Result<&'a str, String> {
    args.get(index)
        .map(String::as_str)
        .ok_or_else(|| format!("{option} requires a value"))
}

fn shorten(value: &str, width: usize) -> String {
    let mut chars = value.chars();
    let shortened: String = chars.by_ref().take(width).collect();
    if chars.next().is_some() {
        format!(
            "{}…",
            shortened
                .chars()
                .take(width.saturating_sub(1))
                .collect::<String>()
        )
    } else {
        shortened
    }
}

fn print_help() {
    println!("Usage: lean-ctx checkpoints [--session <id>] [--commit <sha>] [--orphaned]");
}
