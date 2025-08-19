pub mod logger;

use parser_cli::cli::Cli;

fn main() {
    logger::setup_logger();

    Cli::execute()
}
