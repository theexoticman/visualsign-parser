use parser_app::cli::Cli;

#[tokio::main]
async fn main() {
    Cli::execute().await
}
