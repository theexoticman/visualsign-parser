use crate::{
    AnnotatedPayloadField, SignablePayloadField, SignablePayloadFieldAmountV2,
    SignablePayloadFieldCommon, SignablePayloadFieldNumber, SignablePayloadFieldTextV2,
};
use base64::{engine::general_purpose::STANDARD as b64, Engine as _};

pub fn create_text_field(label: &str, text: &str) -> AnnotatedPayloadField {
    AnnotatedPayloadField {
        static_annotation: None,
        dynamic_annotation: None,
        signable_payload_field: SignablePayloadField::TextV2 {
            common: SignablePayloadFieldCommon {
                fallback_text: text.to_string(),
                label: label.to_string(),
            },
            text_v2: SignablePayloadFieldTextV2 {
                text: text.to_string(),
            },
        },
    }
}

pub fn create_number_field(label: &str, number: &str, unit: &str) -> AnnotatedPayloadField {
    AnnotatedPayloadField {
        static_annotation: None,
        dynamic_annotation: None,
        signable_payload_field: SignablePayloadField::Number {
            common: SignablePayloadFieldCommon {
                fallback_text: format!("{} {}", number, unit),
                label: label.to_string(),
            },
            number: SignablePayloadFieldNumber {
                number: number.to_string(),
            },
        },
    }
}

pub fn create_amount_field(label: &str, amount: &str, abbreviation: &str) -> AnnotatedPayloadField {
    AnnotatedPayloadField {
        static_annotation: None,
        dynamic_annotation: None,
        signable_payload_field: SignablePayloadField::AmountV2 {
            common: SignablePayloadFieldCommon {
                fallback_text: format!("{} {}", amount, abbreviation),
                label: label.to_string(),
            },
            amount_v2: SignablePayloadFieldAmountV2 {
                amount: amount.to_string(),
                abbreviation: Some(abbreviation.to_string()),
            },
        },
    }
}

fn default_hex_representation(data: &[u8]) -> String {
    data.iter()
        .map(|byte| format!("{:02x}", byte))
        .collect::<Vec<String>>()
        .join("")
}

/// Create a standard Raw Data field for expanded views
pub fn create_raw_data_field(
    data: &[u8],
    optional_fallback_string: Option<String>,
) -> AnnotatedPayloadField {
    let raw_data_fallback_string =
        optional_fallback_string.unwrap_or_else(|| default_hex_representation(data));

    AnnotatedPayloadField {
        signable_payload_field: SignablePayloadField::TextV2 {
            common: SignablePayloadFieldCommon {
                fallback_text: raw_data_fallback_string.to_string(),
                label: "Raw Data".to_string(),
            },
            text_v2: SignablePayloadFieldTextV2 {
                text: b64.encode(data),
            },
        },
        static_annotation: None,
        dynamic_annotation: None,
    }
}
