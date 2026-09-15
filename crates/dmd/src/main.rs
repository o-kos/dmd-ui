use std::path::PathBuf;

use clap::{Parser, Subcommand, ValueEnum};

#[derive(Clone, Copy, Debug, Default, ValueEnum)]
enum Backend {
    #[default]
    Auto,
    Native,
    Replay,
}

#[derive(Debug, Parser)]
#[command(version, about = "Inspect and process recorded signals")]
struct Cli {
    /// Module and recording search directories, separated by semicolons.
    #[arg(short = 's', long, global = true, value_delimiter = ';')]
    search_path: Vec<PathBuf>,
    /// Select the execution backend.
    #[arg(long, global = true, value_enum, default_value = "auto")]
    backend: Backend,
    /// Write events to this path.
    #[arg(long, global = true)]
    events: Option<PathBuf>,
    #[command(subcommand)]
    command: Command,
}

#[derive(Debug, Subcommand)]
enum Command {
    /// Run signal processing.
    Run,
    /// Check results.
    Check,
    /// Inspect module information.
    Info,
    /// List modules.
    List,
    /// Inspect module masks.
    Mask,
    /// Convert a signal.
    Convert,
    /// Record a run.
    Record,
}

impl Command {
    fn name(&self) -> &'static str {
        match self {
            Self::Run => "run",
            Self::Check => "check",
            Self::Info => "info",
            Self::List => "list",
            Self::Mask => "mask",
            Self::Convert => "convert",
            Self::Record => "record",
        }
    }
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    anyhow::bail!("{}: not implemented", cli.command.name());
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn global_options_parse_before_and_after_command() {
        let cli = Cli::try_parse_from([
            "dmd",
            "-s",
            "signals;modules",
            "run",
            "--backend",
            "replay",
            "--events",
            "events",
        ])
        .unwrap();
        assert_eq!(
            cli.search_path,
            [PathBuf::from("signals"), PathBuf::from("modules")]
        );
        assert!(matches!(cli.backend, Backend::Replay));
        assert_eq!(cli.events, Some(PathBuf::from("events")));
        let cli = Cli::try_parse_from(["dmd", "list"]).unwrap();
        assert!(matches!(cli.backend, Backend::Auto));
    }

    #[test]
    fn invalid_surface_is_rejected() {
        for args in [
            vec!["dmd"],
            vec!["dmd", "unknown"],
            vec!["dmd", "--backend", "other", "run"],
        ] {
            assert!(Cli::try_parse_from(args).is_err());
        }
    }
}
