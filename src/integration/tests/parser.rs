use base64::Engine;
use generated::health::{AppHealthRequest, AppHealthResponse};
use generated::parser::{Chain, ParseRequest};
use integration::TestArgs;

/// Helper function to create a complete Solana transaction from a message with empty signatures
fn create_solana_transaction_with_empty_signatures(message_base64: &str) -> String {
    // Decode the message
    let message_bytes = base64::engine::general_purpose::STANDARD
        .decode(message_base64)
        .unwrap();

    // Create a complete Solana transaction with empty signatures
    let mut transaction_bytes = Vec::new();

    // Add compact array length for signatures (0 signatures)
    transaction_bytes.push(0u8);

    // Add the message
    transaction_bytes.extend_from_slice(&message_bytes);

    // Encode the complete transaction back to base64
    base64::engine::general_purpose::STANDARD.encode(transaction_bytes)
}

/// Recursively validates that all fields in expected are present in actual
/// This catches missing fields but allows extra fields in actual implementation.
/// Instead of complicating this further, I'm focusing to ensure that the expected field texts are correct first
fn validate_json_structure(actual: &serde_json::Value, expected: &serde_json::Value, path: &str) {
    match (actual, expected) {
        (serde_json::Value::Object(actual_map), serde_json::Value::Object(expected_map)) => {
            for (key, expected_value) in expected_map {
                let current_path = if path.is_empty() {
                    key.clone()
                } else {
                    format!("{}.{}", path, key)
                };

                let actual_value = actual_map
                    .get(key)
                    .unwrap_or_else(|| panic!("Missing field '{}' in actual JSON", current_path));

                validate_json_structure(actual_value, expected_value, &current_path);
            }
        }
        (serde_json::Value::Array(actual_arr), serde_json::Value::Array(expected_arr)) => {
            assert_eq!(
                actual_arr.len(),
                expected_arr.len(),
                "Array length mismatch at '{}': expected {}, got {}",
                path,
                expected_arr.len(),
                actual_arr.len()
            );

            for (i, (actual_item, expected_item)) in
                actual_arr.iter().zip(expected_arr.iter()).enumerate()
            {
                let current_path = format!("{}[{}]", path, i);
                validate_json_structure(actual_item, expected_item, &current_path);
            }
        }
        _ => {
            assert_eq!(
                actual, expected,
                "Value mismatch at '{}': expected {:?}, got {:?}",
                path, expected, actual
            );
        }
    }
}

/// Validates that actual contains at least all fields from expected (strict subset check)
fn validate_required_fields_present(actual: &serde_json::Value, expected: &serde_json::Value) {
    validate_json_structure(actual, expected, "");
}

// XXX: if you're iterating on these tests and the underlying code, make sure you run `cargo build --all`.
// Otherwise, Rust will not recompile the app binaries used here.
// You can also use `make test`, which takes care of recompiling the binaries before running the tests.

#[tokio::test]
async fn parser_e2e() {
    async fn test(test_args: TestArgs) {
        let parse_request = ParseRequest {
            unsigned_payload: "unsignedpayload".to_string(),
            chain: Chain::Unspecified as i32,
        };

        let parse_response = test_args
            .parser_client
            .unwrap()
            .parse(tonic::Request::new(parse_request))
            .await
            .unwrap()
            .into_inner();

        let parsed_transaction = parse_response.parsed_transaction.unwrap().payload.unwrap();
        assert_eq!(
            parsed_transaction.signable_payload,
            "{\"Fields\":[{\"Type\":\"text_v2\",\"FallbackText\":\"Unspecified Chain\",\"Label\":\"Network\",\"TextV2\":{\"Text\":\"Unspecified Chain\"}},{\"Type\":\"text_v2\",\"FallbackText\":\"Raw Data\",\"Label\":\"Raw Data\",\"TextV2\":{\"Text\":\"unsignedpayload\"}}],\"PayloadType\":\"fill in parsed signable payload\",\"Title\":\"Unspecified Transaction\",\"Version\":\"0\"}"
        );
    }

    integration::Builder::new().execute(test).await
}

#[tokio::test]
async fn parser_health_check() {
    async fn test(test_args: TestArgs) {
        let request = tonic::Request::new(AppHealthRequest {});
        let response = test_args
            .health_check_client
            .unwrap()
            .app_health(request)
            .await;
        assert_eq!(
            response.unwrap().into_inner(),
            AppHealthResponse { code: 200 }
        );
    }

    integration::Builder::new().execute(test).await
}

#[tokio::test]
async fn parser_k8_health() {
    async fn test(test_args: TestArgs) {
        integration::k8_health(test_args).await;
    }

    integration::Builder::new().execute(test).await
}

#[tokio::test]
async fn parser_solana_native_transfer_e2e() {
    async fn test(test_args: TestArgs) {
        // Base64 encoded Solana transfer transaction
        // This was generated using the Solana CLI using solana transfer --sign-only which only prints message, that needs to be wrapped into a transaction
        let solana_transfer_message = "AQABA/vgkNkfnOBm+03UvAGzKxIiY/EUhIUGtwFgGKTg7zW14SNc89MK6BlbxE3LXXr+fSM83MDCJQ1wYYDwn39kZloAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAALHd9i9ddFp9Pbk7SBekGKS8baviinT//+klSnP/gLqsAQICAAEMAgAAAACAxqR+jQMA";

        let solana_tx = create_solana_transaction_with_empty_signatures(solana_transfer_message);
        println!("Solana transaction: {}", solana_tx);
        let parse_request = ParseRequest {
            unsigned_payload: solana_tx,
            chain: Chain::Solana as i32,
        };

        let parse_response = test_args
            .parser_client
            .unwrap()
            .parse(tonic::Request::new(parse_request))
            .await
            .unwrap()
            .into_inner();

        let parsed_transaction = parse_response.parsed_transaction.unwrap().payload.unwrap();

        // this is currently optimized around just being able to copy the json output from parser as-is and pass the eye-test
        let expected_sp = serde_json::json!({
          "Fields": [
            {
              "Type": "text_v2",
              "FallbackText": "Solana",
              "Label": "Network",
              "TextV2": {
                "Text": "Solana"
              }
            },
            {
              "Type": "text_v2",
              "FallbackText": "HxDwUk6kcpR7UStRPHnCSCqCGqWWxqBcgf8YkGrAnwak, G9r1RYmVnXptzCA2an46rNnHsCAQLvjyM6vR2mo3LpG1, 11111111111111111111111111111111",
              "Label": "Account Keys",
              "TextV2": {
                "Text": "HxDwUk6kcpR7UStRPHnCSCqCGqWWxqBcgf8YkGrAnwak, G9r1RYmVnXptzCA2an46rNnHsCAQLvjyM6vR2mo3LpG1, 11111111111111111111111111111111"
              }
            },
            {
              "Type": "text_v2",
              "FallbackText": "Transfer 1: HxDwUk6kcpR7UStRPHnCSCqCGqWWxqBcgf8YkGrAnwak -> G9r1RYmVnXptzCA2an46rNnHsCAQLvjyM6vR2mo3LpG1: 1000000000000000",
              "Label": "Transfer 1",
              "TextV2": {
                "Text": "From: HxDwUk6kcpR7UStRPHnCSCqCGqWWxqBcgf8YkGrAnwak\nTo: G9r1RYmVnXptzCA2an46rNnHsCAQLvjyM6vR2mo3LpG1\nAmount: 1000000000000000"
              }
            },
            {
              "Type": "preview_layout",
              "FallbackText": "Program ID: 11111111111111111111111111111111\nData: 020000000080c6a47e8d0300",
              "Label": "Instruction 1",
              "PreviewLayout": {
                "Title": {
                  "Text": "Transfer: 1000000000000000 lamports"
                },
                "Subtitle": {
                  "Text": ""
                },
                "Condensed": {
                  "Fields": [
                    {
                      "Type": "text_v2",
                      "FallbackText": "Transfer: 1000000000000000 lamports",
                      "Label": "Instruction",
                      "TextV2": {
                        "Text": "Transfer: 1000000000000000 lamports"
                      }
                    }
                  ]
                },
                "Expanded": {
                  "Fields": [
                    {
                      "Type": "text_v2",
                      "FallbackText": "11111111111111111111111111111111",
                      "Label": "Program ID",
                      "TextV2": {
                        "Text": "11111111111111111111111111111111"
                      }
                    },
                    {
                      "Type": "amount_v2",
                      "FallbackText": "1000000 SOL",
                      "Label": "Transfer Amount",
                      "AmountV2": {
                        "Amount": "1000000000000000",
                        "Abbreviation": "lamports"
                      }
                    },
                    {
                      "Type": "text_v2",
                      "FallbackText": "020000000080c6a47e8d0300",
                      "Label": "Raw Data",
                      "TextV2": {
                        "Text": "020000000080c6a47e8d0300"
                      }
                    }
                  ]
                }
              }
            }
          ],
          "PayloadType": "SolanaTx",
          "Title": "Solana Transaction",
          "Version": "0"
        });

        // Verify the transaction contains Solana-specific fields
        let signable_payload: serde_json::Value =
            serde_json::from_str(&parsed_transaction.signable_payload).unwrap();

        // Validate that the parsed transaction contains all expected fields
        validate_required_fields_present(&signable_payload, &expected_sp);
    }

    integration::Builder::new().execute(test).await
}

// TODO SPL transfers
// TODO major DeFi examples
// Maybe this should be handled by the chain_parsers?
