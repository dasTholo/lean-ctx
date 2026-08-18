//! `lean-ctx evidence` — evidence-bundle commands.

use anyhow::{Context, Result};
use clap::{Args, Parser, Subcommand};
use std::{
    fs::File,
    io::Read,
    path::{Path, PathBuf},
};
use zip::ZipArchive;

/// Arguments for evidence-bundle operations.
#[derive(Debug, Parser)]
#[command(
    name = "lean-ctx evidence",
    about = "Generate or inspect evidence bundles"
)]
struct EvidenceCommand {
    #[command(subcommand)]
    command: EvidenceSubcommand,
}

#[derive(Debug, Subcommand)]
enum EvidenceSubcommand {
    /// Compare providers and generate an evidence bundle.
    Run(EvidenceRunCommand),
    /// List an evidence bundle's files and display its manifest.
    Inspect(EvidenceInspectCommand),
}

#[derive(Debug, Args)]
struct EvidenceRunCommand {}

#[derive(Debug, Args)]
struct EvidenceInspectCommand {
    /// Path to the evidence-bundle ZIP archive.
    #[arg(value_name = "BUNDLE_PATH", value_hint = clap::ValueHint::FilePath)]
    bundle_path: PathBuf,
}

/// Parse, execute, print, and return the process status for evidence commands.
pub(crate) fn run(args: &[String]) -> i32 {
    let mut argv = Vec::with_capacity(args.len() + 1);
    argv.push("lean-ctx evidence".to_string());
    argv.extend(args.iter().cloned());

    let command = match EvidenceCommand::try_parse_from(argv) {
        Ok(command) => command,
        Err(error) => {
            let code = error.exit_code();
            eprint!("{error}");
            return code;
        }
    };

    match command.command {
        EvidenceSubcommand::Run(_) => {
            println!(
                "Provider comparison evidence generation is not available yet.\n\nUsage: lean-ctx evidence run"
            );
            0
        }
        EvidenceSubcommand::Inspect(command) => match inspect(&command.bundle_path) {
            Ok(()) => 0,
            Err(error) => {
                eprintln!("evidence inspect: {error:#}");
                1
            }
        },
    }
}

fn inspect(path: &Path) -> Result<()> {
    let file = File::open(path).with_context(|| format!("open {}", path.display()))?;
    let mut archive = ZipArchive::new(file).context("read ZIP archive")?;
    let mut names = Vec::with_capacity(archive.len());
    let mut manifest = None;

    for index in 0..archive.len() {
        let mut entry = archive
            .by_index(index)
            .with_context(|| format!("read ZIP entry {index}"))?;
        let name = entry.name().to_string();
        if name == "manifest.json" {
            let mut contents = String::new();
            entry
                .read_to_string(&mut contents)
                .context("read manifest.json")?;
            manifest = Some(contents);
        }
        names.push(name);
    }

    let manifest = manifest.context("missing manifest.json")?;
    println!("Evidence bundle: {}", path.display());
    println!("Files:");
    for name in names {
        println!("  {name}");
    }
    println!("Manifest:");
    println!("{manifest}");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::{Cursor, Write};

    #[test]
    fn inspect_reads_manifest_and_lists_entries() {
        let mut zip = zip::ZipWriter::new(Cursor::new(Vec::new()));
        let options = zip::write::SimpleFileOptions::default();
        zip.start_file("manifest.json", options).unwrap();
        zip.write_all(b"{\"bundle\":\"evidence-bundle\"}").unwrap();
        zip.start_file("audit/trail.jsonl", options).unwrap();
        zip.write_all(b"entry\n").unwrap();
        let path = std::env::temp_dir().join(format!(
            "lean-ctx-evidence-inspect-{}.zip",
            std::process::id()
        ));
        std::fs::write(&path, zip.finish().unwrap().into_inner()).unwrap();

        assert!(inspect(&path).is_ok());
        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn inspect_requires_manifest() {
        let mut zip = zip::ZipWriter::new(Cursor::new(Vec::new()));
        let options = zip::write::SimpleFileOptions::default();
        zip.start_file("audit/trail.jsonl", options).unwrap();
        zip.write_all(b"entry\n").unwrap();
        let path = std::env::temp_dir().join(format!(
            "lean-ctx-evidence-no-manifest-{}.zip",
            std::process::id()
        ));
        std::fs::write(&path, zip.finish().unwrap().into_inner()).unwrap();

        assert!(inspect(&path).is_err());
        let _ = std::fs::remove_file(path);
    }
}
