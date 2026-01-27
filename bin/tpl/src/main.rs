mod output;

use std::path::PathBuf;

use clap::Parser;
use eyre::Result;
use tracing::Level;
use tracing_subscriber::FmtSubscriber;

use template_core::{
    ApplyEvent, SourceKind, apply_template, fetch_github, fetch_local, parse_source,
};

const EXAMPLES: &str = "\
Examples:
  tpl user/repo              Clone from GitHub default branch
  tpl user/repo#v1.0         Clone specific tag or branch
  tpl user/repo/subdir       Extract subdirectory only
  tpl file://./templates     Use local directory
  tpl user/repo ./dest       Specify destination";

/// Scaffold projects from GitHub repositories or local directories.
#[derive(Parser, Debug)]
#[command(name = "tpl")]
#[command(version, about, long_about = None)]
#[command(before_help = EXAMPLES)]
#[command(after_help = "For more info, see: https://github.com/longcipher/template")]
struct Cli {
    /// Template source specification.
    ///
    /// Formats:
    ///   user/repo              GitHub repository
    ///   user/repo#branch       With branch or tag
    ///   user/repo/path         With subdirectory
    ///   file://path            Local directory
    #[arg(value_name = "SOURCE")]
    source: String,

    /// Target directory for template files.
    #[arg(value_name = "DEST", default_value = ".")]
    destination: PathBuf,

    /// Use SSH for GitHub clone (git@github.com:...).
    ///
    /// Required for private repositories when using SSH authentication.
    #[arg(long, short = 's')]
    ssh: bool,

    /// Show detailed output including directory creation.
    #[arg(long, short = 'v')]
    verbose: bool,
}

fn main() -> Result<()> {
    // Parse CLI arguments
    let cli = Cli::parse();

    // Initialize tracing
    let level = if cli.verbose { Level::DEBUG } else { Level::INFO };
    let subscriber =
        FmtSubscriber::builder().with_max_level(level).without_time().with_target(false).finish();
    tracing::subscriber::set_global_default(subscriber)?;

    // Parse the source specification
    let source = parse_source(&cli.source)?;

    // Fetch the template source
    let (source_path, _temp_dir, respect_gitignore) = match source.kind {
        SourceKind::GitHub { owner, repo, subdir, revision } => {
            output::cloning(&owner, &repo);
            let (temp_dir, path) =
                fetch_github(&owner, &repo, revision.as_deref(), subdir.as_deref(), cli.ssh)?;
            output::cloned();
            // GitHub clones don't need gitignore filtering (already clean)
            (path, Some(temp_dir), false)
        }
        SourceKind::Local { path } => {
            output::local_template(&path);
            let resolved = fetch_local(&path)?;
            // Local templates should respect .gitignore
            (resolved, None, true)
        }
    };

    // Apply the template
    output::applying(&cli.destination);

    let result =
        apply_template(&source_path, &cli.destination, respect_gitignore, |event| match event {
            ApplyEvent::FileApplied(path) => {
                output::file_applied(&path);
            }
            ApplyEvent::FileSkipped(path) => {
                output::file_skipped(&path);
            }
            ApplyEvent::DirCreated(path) => {
                if cli.verbose {
                    output::dir_created(&path);
                }
            }
        })?;

    // Print summary
    output::summary(result.applied.len(), result.skipped.len());

    Ok(())
}
