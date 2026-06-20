//! Minimal command-line argument handling.
//!
//! Rustle takes no real options — it just plays today's puzzle — so this is a
//! deliberately tiny, dependency-free parser that recognises only the
//! conventional `--help` and `--version` flags. Keeping it pure (a slice of
//! args in, a [`CliAction`] out) makes the precedence rules easy to unit-test
//! without spawning the process.

/// What the parsed command line asks the program to do.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CliAction {
    /// No recognised flags: start the game.
    Run,
    /// Print the help text and exit successfully.
    Help,
    /// Print the version line and exit successfully.
    Version,
    /// An unrecognised argument was given; the payload is the offending token.
    Unknown(String),
}

/// Parses the process arguments (including `argv[0]`) into a [`CliAction`].
///
/// Precedence is conventional: `--help` wins over everything, then `--version`,
/// then the first unrecognised token is reported. With no flags the result is
/// [`CliAction::Run`].
pub fn parse_args<I>(args: I) -> CliAction
where
    I: IntoIterator<Item = String>,
{
    // Skip argv[0] (the program name).
    let rest: Vec<String> = args.into_iter().skip(1).collect();

    if rest.iter().any(|a| a == "-h" || a == "--help") {
        return CliAction::Help;
    }
    if rest
        .iter()
        .any(|a| a == "-v" || a == "-V" || a == "--version")
    {
        return CliAction::Version;
    }
    if let Some(unknown) = rest.into_iter().next() {
        return CliAction::Unknown(unknown);
    }
    CliAction::Run
}

/// Returns the one-line version string, e.g. `strustle 0.2.0`.
pub fn version_line() -> String {
    format!("{} {}", env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION"))
}

/// Returns the full `--help` text.
pub fn help_text() -> String {
    format!(
        "{name} {version}\n\
         {description}\n\
         \n\
         USAGE:\n    \
         {name} [OPTIONS]\n\
         \n\
         OPTIONS:\n    \
         -h, --help       Print this help and exit\n    \
         -v, --version    Print version information and exit\n\
         \n\
         Run with no options to play today's puzzle.",
        name = env!("CARGO_PKG_NAME"),
        version = env!("CARGO_PKG_VERSION"),
        description = env!("CARGO_PKG_DESCRIPTION"),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn args(items: &[&str]) -> Vec<String> {
        items.iter().map(|s| s.to_string()).collect()
    }

    #[test]
    fn no_args_runs() {
        assert_eq!(parse_args(args(&["strustle"])), CliAction::Run);
    }

    #[test]
    fn help_flags() {
        assert_eq!(parse_args(args(&["strustle", "--help"])), CliAction::Help);
        assert_eq!(parse_args(args(&["strustle", "-h"])), CliAction::Help);
    }

    #[test]
    fn version_flags() {
        assert_eq!(
            parse_args(args(&["strustle", "--version"])),
            CliAction::Version
        );
        assert_eq!(parse_args(args(&["strustle", "-v"])), CliAction::Version);
        assert_eq!(parse_args(args(&["strustle", "-V"])), CliAction::Version);
    }

    #[test]
    fn help_takes_precedence_over_version() {
        assert_eq!(
            parse_args(args(&["strustle", "--version", "--help"])),
            CliAction::Help
        );
    }

    #[test]
    fn unknown_flag_is_reported() {
        assert_eq!(
            parse_args(args(&["strustle", "--nope"])),
            CliAction::Unknown("--nope".to_string())
        );
    }

    #[test]
    fn known_flag_beats_later_unknown() {
        // A recognised flag anywhere wins over an unrecognised one.
        assert_eq!(
            parse_args(args(&["strustle", "--version", "bogus"])),
            CliAction::Version
        );
    }

    #[test]
    fn version_line_contains_name_and_version() {
        let line = version_line();
        assert!(line.starts_with("strustle "));
        assert_eq!(line, format!("strustle {}", env!("CARGO_PKG_VERSION")));
    }

    #[test]
    fn help_text_lists_both_flags() {
        let text = help_text();
        assert!(text.contains("--help"));
        assert!(text.contains("--version"));
        assert!(text.contains("USAGE:"));
    }
}
