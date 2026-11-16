use clap::CommandFactory;
use todo::cli::Cli;

#[test]
fn verify_cli_structure() {
    Cli::command().debug_assert();
}

#[test]
fn test_completion_generation() {
    for shell in [
        clap_complete::Shell::Bash,
        clap_complete::Shell::Zsh,
        clap_complete::Shell::Fish,
        clap_complete::Shell::PowerShell,
    ] {
        let mut buf = Vec::new();
        let mut cmd = Cli::command();
        clap_complete::generate(shell, &mut cmd, "todo", &mut buf);
        assert!(!buf.is_empty());
        let output = String::from_utf8(buf).unwrap();
        assert!(output.contains("todo"));
    }
}
