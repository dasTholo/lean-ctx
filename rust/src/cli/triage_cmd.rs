//! `lean-ctx triage accuracy` — semantic-shadow agreement report.

use crate::core::triage::accuracy_tracker::AccuracyTracker;

pub(crate) fn cmd_triage(args: &[String]) {
    match render(args) {
        Ok(output) => print!("{output}"),
        Err(error) => {
            eprintln!("triage: {error}");
            eprintln!("Usage: lean-ctx triage accuracy [--json]");
            std::process::exit(2);
        }
    }
}

fn render(args: &[String]) -> Result<String, String> {
    let json = match args {
        [command] if command == "accuracy" => false,
        [command, flag] if command == "accuracy" && flag == "--json" => true,
        _ => return Err("expected `accuracy [--json]`".to_string()),
    };
    let report = AccuracyTracker::report();
    if json {
        serde_json::to_string_pretty(&report)
            .map_err(|error| format!("serialize accuracy report: {error}"))
    } else {
        Ok(format!(
            "Semantic triage shadow accuracy\n\n{:<28} {}\n{:<28} {:.1}%\n{:<28} {:.1}%\n{:<28} {:.1}%\n{:<28} {:.1}%\n{:<28} {}\n",
            "Tasks compared",
            report.total_tasks_compared,
            "Agreement rate",
            report.agreement_rate * 100.0,
            "Intent F1",
            report.per_field_f1.intent * 100.0,
            "Complexity F1",
            report.per_field_f1.complexity * 100.0,
            "Scope F1",
            report.per_field_f1.scope * 100.0,
            "Ready for promotion",
            if report.ready_for_promotion {
                "yes"
            } else {
                "no"
            },
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accuracy_command_renders_json() {
        let _data = crate::core::data_dir::isolated_data_dir();
        let args = ["accuracy".to_string(), "--json".to_string()];
        assert!(serde_json::from_str::<serde_json::Value>(&render(&args).unwrap()).is_ok());
    }
}
