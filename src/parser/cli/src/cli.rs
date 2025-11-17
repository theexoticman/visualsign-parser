use crate::chains;
use chains::{available_chains, parse_chain};
use clap::{Arg, Command};
use parser_app::registry::create_registry;
use std::fmt::Write;
use visualsign::vsptrait::VisualSignOptions;
use visualsign::{SignablePayload, SignablePayloadField};

/// Formats a `SignablePayload` in a human-readable tree format
fn format_human_readable(payload: &SignablePayload) -> String {
    let mut output = String::new();

    // Header
    writeln!(&mut output, "┌─ Transaction: {}", payload.title).unwrap();
    if let Some(subtitle) = &payload.subtitle {
        writeln!(&mut output, "│  Subtitle: {subtitle}").unwrap();
    }
    writeln!(&mut output, "│  Version: {}", payload.version).unwrap();
    if !payload.payload_type.is_empty() {
        writeln!(&mut output, "│  Type: {}", payload.payload_type).unwrap();
    }
    output.push_str("│\n");

    // Fields
    if !payload.fields.is_empty() {
        output.push_str("└─ Fields:\n");
        for (i, field) in payload.fields.iter().enumerate() {
            let is_last = i == payload.fields.len() - 1;
            let prefix = if is_last { "   └─" } else { "   ├─" };
            let continuation = if is_last { "      " } else { "   │  " };

            format_field(field, &mut output, prefix, continuation);
        }
    }

    output
}

/// Formats a single field with tree-like indentation
fn format_field(
    field: &SignablePayloadField,
    output: &mut String,
    prefix: &str,
    continuation: &str,
) {
    match field {
        SignablePayloadField::TextV2 { common, text_v2 } => {
            writeln!(output, "{} {}: {}", prefix, common.label, text_v2.text).unwrap();
        }
        SignablePayloadField::PreviewLayout {
            common,
            preview_layout,
        } => {
            writeln!(output, "{} {}", prefix, common.label).unwrap();

            if let Some(title) = &preview_layout.title {
                writeln!(output, "{}   Title: {}", continuation, title.text).unwrap();
            }
            if let Some(subtitle) = &preview_layout.subtitle {
                writeln!(output, "{}   Detail: {}", continuation, subtitle.text).unwrap();
            }

            // Condensed view (if present)
            if let Some(condensed_layout) = &preview_layout.condensed {
                if !condensed_layout.fields.is_empty() {
                    writeln!(output, "{continuation}   📋 Condensed View:").unwrap();
                    for (i, nested_field) in condensed_layout.fields.iter().enumerate() {
                        let is_last_nested = i == condensed_layout.fields.len() - 1;
                        let nested_prefix = format!(
                            "{}   {}",
                            continuation,
                            if is_last_nested { "└─" } else { "├─" }
                        );
                        let nested_continuation = format!(
                            "{}   {}",
                            continuation,
                            if is_last_nested { "   " } else { "│  " }
                        );
                        format_field(
                            &nested_field.signable_payload_field,
                            output,
                            &nested_prefix,
                            &nested_continuation,
                        );
                    }
                }
            }

            // Expanded view (if present)
            if let Some(expanded_layout) = &preview_layout.expanded {
                if !expanded_layout.fields.is_empty() {
                    writeln!(output, "{continuation}   📖 Expanded View:").unwrap();
                    for (i, nested_field) in expanded_layout.fields.iter().enumerate() {
                        let is_last_nested = i == expanded_layout.fields.len() - 1;
                        let nested_prefix = format!(
                            "{}   {}",
                            continuation,
                            if is_last_nested { "└─" } else { "├─" }
                        );
                        let nested_continuation = format!(
                            "{}   {}",
                            continuation,
                            if is_last_nested { "   " } else { "│  " }
                        );
                        format_field(
                            &nested_field.signable_payload_field,
                            output,
                            &nested_prefix,
                            &nested_continuation,
                        );
                    }
                }
            }
        }
        SignablePayloadField::AmountV2 { common, amount_v2 } => {
            writeln!(
                output,
                "{} {}: {} {}",
                prefix,
                common.label,
                amount_v2.amount,
                amount_v2.abbreviation.as_deref().unwrap_or("")
            )
            .unwrap();
        }
        SignablePayloadField::AddressV2 { common, address_v2 } => {
            writeln!(
                output,
                "{} {}: {}",
                prefix, common.label, address_v2.address
            )
            .unwrap();
        }
        _ => {
            writeln!(output, "{} Field: {}", prefix, common_label(field)).unwrap();
        }
    }
}

/// Helper to extract common label from any field type
fn common_label(field: &SignablePayloadField) -> String {
    match field {
        SignablePayloadField::TextV2 { common, .. }
        | SignablePayloadField::PreviewLayout { common, .. }
        | SignablePayloadField::AmountV2 { common, .. }
        | SignablePayloadField::AddressV2 { common, .. } => common.label.clone(),
        _ => "Unknown".to_string(),
    }
}

fn parse_and_display(chain: &str, raw_tx: &str, options: VisualSignOptions, output_format: &str) {
    let registry_chain = parse_chain(chain);

    let registry = create_registry();
    let signable_payload_str = registry.convert_transaction(&registry_chain, raw_tx, options);
    match signable_payload_str {
        Ok(payload) => match output_format {
            "json" => {
                if let Ok(json_output) = serde_json::to_string_pretty(&payload) {
                    println!("{json_output}");
                } else {
                    eprintln!("Error: Failed to serialize output as JSON");
                }
            }
            "text" => {
                println!("{payload:#?}");
            }
            "human" => {
                let human_output = format_human_readable(&payload);
                println!("{human_output}");
            }
            _ => {
                eprintln!("Error: Unsupported output format '{output_format}'");
            }
        },
        Err(err) => {
            eprintln!("Error: {err:?}");
        }
    }
}

/// app cli
pub struct Cli;
impl Cli {
    /// start the parser cli
    ///
    /// # Panics
    ///
    /// Executes the CLI application, parsing command line arguments and processing the transaction
    pub fn execute() {
        let chains = available_chains();
        let chain_help = format!("Chain type ({})", chains.join(", "));

        let matches = Command::new("visualsign-parser")
            .version("1.0")
            .about("Converts raw transactions to visual signing properties")
            .arg(
                Arg::new("chain")
                    .short('c')
                    .long("chain")
                    .value_name("CHAIN")
                    .help(&chain_help)
                    .value_parser(chains.clone())
                    .required(true),
            )
            .arg(
                Arg::new("transaction")
                    .short('t')
                    .long("transaction")
                    .value_name("RAW_TX")
                    .help("Raw transaction hex string")
                    .required(true),
            )
            .arg(
                Arg::new("output")
                    .short('o')
                    .long("output")
                    .value_name("FORMAT")
                    .help("Output format")
                    .value_parser(["text", "json", "human"])
                    .default_value("text"),
            )
            .get_matches();

        let chain = matches
            .get_one::<String>("chain")
            .expect("Chain is required");
        let raw_tx = matches
            .get_one::<String>("transaction")
            .expect("Transaction is required");
        let output_format = matches
            .get_one::<String>("output")
            .expect("Output format has default value");

        let options = VisualSignOptions {
            decode_transfers: true,
            transaction_name: None,
        };

        parse_and_display(chain, raw_tx, options, output_format);
    }
}
