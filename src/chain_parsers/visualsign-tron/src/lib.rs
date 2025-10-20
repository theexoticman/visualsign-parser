use visualsign::{
    SignablePayload, SignablePayloadField, SignablePayloadFieldCommon, SignablePayloadFieldTextV2,
    vsptrait::{
        Transaction, TransactionParseError, VisualSignConverter, VisualSignConverterFromString,
        VisualSignError, VisualSignOptions,
    },
};

use anychain_tron::protocol::Tron::transaction;
use anychain_tron::protocol::balance_contract::TransferContract;
use protobuf::Message;

// This is a standalone crate for handling unspecified or unknown transactions, mostly provided for testing and as a sample implementation template to start from
/// Wrapper for unspecified/unknown transactions
#[derive(Debug, Clone)]
pub struct TronTransactionWrapper {
    raw_data: String,
}

impl Transaction for TronTransactionWrapper {
    fn from_string(data: &str) -> Result<Self, TransactionParseError> {
        // Basic validation - try to decode hex
        let clean_hex = data.strip_prefix("0x").unwrap_or(data);
        hex::decode(clean_hex)
            .map_err(|e| TransactionParseError::DecodeError(e.to_string()))?;

        Ok(Self {
            raw_data: data.to_string(),
        })
    }

    fn transaction_type(&self) -> String {
        "Tron".to_string()
    }
}

impl TronTransactionWrapper {
    pub fn new(raw_data: String) -> Self {
        Self { raw_data }
    }

    pub fn raw_data(&self) -> &str {
        &self.raw_data
    }
}

/// Converter for unspecified/unknown chains
pub struct TronVisualSignConverter;

impl VisualSignConverter<TronTransactionWrapper> for TronVisualSignConverter {
    fn to_visual_sign_payload(
        &self,
        transaction_wrapper: TronTransactionWrapper,
        options: VisualSignOptions,
    ) -> Result<SignablePayload, VisualSignError> {
        convert_to_visual_sign_payload(&transaction_wrapper.raw_data, options)
    }
}

fn convert_to_visual_sign_payload(
    raw_transaction: &str,
    options: VisualSignOptions,
) -> Result<SignablePayload, VisualSignError> {
    // Decode hex to bytes
    let clean_hex = raw_transaction.strip_prefix("0x").unwrap_or(raw_transaction);
    let raw_data_bytes = hex::decode(clean_hex).map_err(|e| {
        VisualSignError::ParseError(TransactionParseError::DecodeError(format!(
            "Failed to decode hex: {}",
            e
        )))
    })?;

    // Parse the Transaction.raw message using protobuf
    let raw_data = transaction::Raw::parse_from_bytes(&raw_data_bytes).map_err(|e| {
        VisualSignError::ParseError(TransactionParseError::DecodeError(format!(
            "Failed to parse raw transaction data: {}",
            e
        )))
    })?;

    let chain_name = "Tron".to_string();

    let mut fields = vec![SignablePayloadField::TextV2 {
        common: SignablePayloadFieldCommon {
            fallback_text: "Tron".to_string(),
            label: "Network".to_string(),
        },
        text_v2: SignablePayloadFieldTextV2 { text: chain_name },
    }];

    // Parse contracts
    for contract in raw_data.contract.iter() {
        if let Some(parameter) = contract.parameter.as_ref() {
            // Decode specific contract types
            match parameter.type_url.as_str() {
                "type.googleapis.com/protocol.TransferContract" => {
                    if let Ok(transfer) = TransferContract::parse_from_bytes(&parameter.value) {
                        let from_address = address_to_base58(&transfer.owner_address);
                        let to_address = address_to_base58(&transfer.to_address);
                        let amount_trx = transfer.amount as f64 / 1_000_000.0;

                        fields.push(SignablePayloadField::TextV2 {
                            common: SignablePayloadFieldCommon {
                                fallback_text: from_address.clone(),
                                label: "From".to_string(),
                            },
                            text_v2: SignablePayloadFieldTextV2 { text: from_address },
                        });

                        fields.push(SignablePayloadField::TextV2 {
                            common: SignablePayloadFieldCommon {
                                fallback_text: to_address.clone(),
                                label: "To".to_string(),
                            },
                            text_v2: SignablePayloadFieldTextV2 { text: to_address },
                        });

                        fields.push(SignablePayloadField::TextV2 {
                            common: SignablePayloadFieldCommon {
                                fallback_text: format!("{} TRX", amount_trx),
                                label: "Amount".to_string(),
                            },
                            text_v2: SignablePayloadFieldTextV2 {
                                text: format!("{} TRX", amount_trx),
                            },
                        });
                    }
                }
                _ => {
                    // Unknown contract type
                    fields.push(SignablePayloadField::TextV2 {
                        common: SignablePayloadFieldCommon {
                            fallback_text: parameter.type_url.clone(),
                            label: "Contract Type".to_string(),
                        },
                        text_v2: SignablePayloadFieldTextV2 {
                            text: parameter.type_url.clone(),
                        },
                    });
                }
            }
        }
    }

    let title = options
        .transaction_name
        .unwrap_or_else(|| "Tron Transaction".to_string());

    Ok(SignablePayload::new(
        0,
        title,
        None,
        fields,
        "TronTx".to_string(),
    ))
}

impl VisualSignConverterFromString<TronTransactionWrapper>
    for TronVisualSignConverter
{
}

// Public API functions
pub fn transaction_to_visual_sign(
    raw_data: String,
    options: VisualSignOptions,
) -> Result<SignablePayload, VisualSignError> {
    let wrapper = TronTransactionWrapper::new(raw_data);
    let converter = TronVisualSignConverter;
    converter.to_visual_sign_payload(wrapper, options)
}

pub fn transaction_string_to_visual_sign(
    transaction_data: &str,
    options: VisualSignOptions,
) -> Result<SignablePayload, VisualSignError> {
    let converter = TronVisualSignConverter;
    converter.to_visual_sign_payload_from_string(transaction_data, options)
}

// Helper function to convert Tron address bytes to base58 format
fn address_to_base58(address_bytes: &[u8]) -> String {
    use sha2::{Digest, Sha256};

    // Add checksum
    let mut hasher = Sha256::new();
    hasher.update(address_bytes);
    let hash1 = hasher.finalize();

    let mut hasher = Sha256::new();
    hasher.update(&hash1);
    let hash2 = hasher.finalize();

    let mut with_checksum = address_bytes.to_vec();
    with_checksum.extend_from_slice(&hash2[..4]);

    base58::ToBase58::to_base58(&with_checksum[..])
}
