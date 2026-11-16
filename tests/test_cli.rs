use clap::CommandFactory;
use todo::cli::Cli;

#[test]
fn verify_cli_structure() {
    Cli::command().debug_assert();
}
