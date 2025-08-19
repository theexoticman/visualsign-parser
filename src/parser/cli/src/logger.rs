use std::fs::File;
use std::io;

use tracing::subscriber::set_global_default;
use tracing_bunyan_formatter::{BunyanFormattingLayer, JsonStorageLayer};
use tracing_log::LogTracer;
use tracing_subscriber::fmt::layer;

use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::{EnvFilter, Layer, registry};

pub fn setup_logger() {
    LogTracer::init().expect("Failed to set logger");

    let stdout_layer = layer()
        .pretty()
        .with_writer(io::stdout)
        .with_filter(EnvFilter::from_default_env());

    let file = File::create("visualsign-parser-cli.log").expect("Failed to create log file");
    let formatting_layer = BunyanFormattingLayer::new("visualsign-parser-cli".into(), file);

    set_global_default(
        registry()
            .with(stdout_layer)
            .with(formatting_layer)
            .with(JsonStorageLayer),
    )
    .expect("Failed to set global default");
}
