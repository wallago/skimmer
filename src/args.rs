//! Command-line arguments parser.

use clap::Parser;

/// Argument parser powered by [`clap`].
#[derive(Clone, Debug, Default, Parser)]
#[clap(
    version,
    author = clap::crate_authors!("\n"),
    about,
    rename_all_env = "screaming-snake",
    help_template = "\
{before-help}skimmer {version}
{author-with-newline}{about-with-newline}
{usage-heading}
  {usage}

{all-args}{after-help}
",
)]
pub struct Args {
    /// Increase logging verbosity (-v, -vv, -vvv)
    #[arg(short, long, action = clap::ArgAction::Count)]
    pub verbose: u8,

    /// Path to the application config file (TOML).
    #[arg(long, value_name = "PATH")]
    pub config: Option<std::path::PathBuf>,

    /// Rss list feed.
    #[arg(long, value_name = "PATH", default_value_t = false)]
    pub feeds: bool,
}

#[cfg(test)]
mod tests {
    use clap::CommandFactory;

    use super::*;
    #[test]
    fn test_args() {
        Args::command().debug_assert();
    }
}
