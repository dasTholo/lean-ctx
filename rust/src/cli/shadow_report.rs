//! `lean-ctx shadow` — inspect or create local Shadow Mode reports.
use crate::core::shadow::{
    comparison::format_report,
    persistence::{list_reports, load_report},
    runtime::ShadowRuntime,
};

pub(crate) fn cmd_shadow(args: &[String]) {
    if args.iter().any(|a| matches!(a.as_str(), "-h" | "--help")) {
        usage();
        return;
    }
    let Some(mode) = parse(args) else {
        eprintln!("shadow: choose at most one of --latest, --list, or --force");
        usage();
        std::process::exit(2);
    };
    if mode == "list" {
        let mut found = false;
        for path in list_reports() {
            if let Some(report) = load_report(&path) {
                found = true;
                println!(
                    "{}  {} micros",
                    report.timestamp, report.savings.absolute_micros
                );
            }
        }
        if !found {
            println!("No Shadow reports recorded yet.");
        }
    } else {
        let report = if mode == "force" {
            ShadowRuntime::force_report()
        } else {
            ShadowRuntime::get_latest_report()
        };
        match report {
            Some(report) => println!("{}", format_report(&report)),
            None => println!("No Shadow report available."),
        }
    }
}

fn parse(args: &[String]) -> Option<&'static str> {
    match args {
        [] => Some("latest"),
        [flag] if flag == "--latest" => Some("latest"),
        [flag] if flag == "--list" => Some("list"),
        [flag] if flag == "--force" => Some("force"),
        _ => None,
    }
}

fn usage() {
    println!(
        "Inspect local Shadow Mode comparisons.\n\nUsage: lean-ctx shadow [--latest|--list|--force]\n\nExamples:\n  lean-ctx shadow\n  lean-ctx shadow --list\n  lean-ctx shadow --force"
    );
}

#[cfg(test)]
mod tests {
    use super::parse;

    #[test]
    fn parse_accepts_one_mode_only() {
        assert_eq!(parse(&[]), Some("latest"));
        assert_eq!(parse(&["--list".into()]), Some("list"));
        assert!(parse(&["--list".into(), "--force".into()]).is_none());
    }
}
