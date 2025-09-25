use crate::errors::VisualSignError;
use serde::{Deserialize, Serialize, Serializer};
use serde::ser::SerializeMap;
use serde_json::Value;
pub mod encodings;
pub mod errors;
pub mod field_builders;
pub mod registry;
pub mod test_utils;
pub mod vsptrait;

// A function to check if a string is empty (used for skip_serializing_if)
fn is_empty_string(s: &str) -> bool {
    s.is_empty()
}

// A bare bones implementation of the SignablePayload struct and its associated methods
// The fields are serialized alphabetically to ensure that default serialization works the same
// and the canonical representation is done by simply sorting the fields first
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct SignablePayload {
    #[serde(rename = "Fields")]
    pub fields: Vec<SignablePayloadField>,
    #[serde(rename = "PayloadType", skip_serializing_if = "is_empty_string")]
    pub payload_type: String,
    #[serde(rename = "Subtitle", skip_serializing_if = "Option::is_none")]
    pub subtitle: Option<String>,
    #[serde(rename = "Title")]
    pub title: String,
    #[serde(rename = "Version")]
    pub version: String,
}

// Common fields shared by all field types
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct SignablePayloadFieldCommon {
    #[serde(rename = "FallbackText")]
    pub fallback_text: String,
    #[serde(rename = "Label")]
    pub label: String,
}

// Now SignablePayloadField is an enum with variants for each field type
#[derive(Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(tag = "Type")]
pub enum SignablePayloadField {
    #[serde(rename = "text")]
    Text {
        #[serde(flatten)]
        common: SignablePayloadFieldCommon,
        #[serde(rename = "Text")]
        text: SignablePayloadFieldText,
    },

    #[serde(rename = "text_v2")]
    TextV2 {
        #[serde(flatten)]
        common: SignablePayloadFieldCommon,
        #[serde(rename = "TextV2")]
        text_v2: SignablePayloadFieldTextV2,
    },

    #[serde(rename = "address")]
    Address {
        #[serde(flatten)]
        common: SignablePayloadFieldCommon,
        #[serde(rename = "Address")]
        address: SignablePayloadFieldAddress,
    },

    #[serde(rename = "address_v2")]
    AddressV2 {
        #[serde(flatten)]
        common: SignablePayloadFieldCommon,
        #[serde(rename = "AddressV2")]
        address_v2: SignablePayloadFieldAddressV2,
    },

    #[serde(rename = "number")]
    Number {
        #[serde(flatten)]
        common: SignablePayloadFieldCommon,
        #[serde(rename = "Number")]
        number: SignablePayloadFieldNumber,
    },

    #[serde(rename = "amount")]
    Amount {
        #[serde(flatten)]
        common: SignablePayloadFieldCommon,
        #[serde(rename = "Amount")]
        amount: SignablePayloadFieldAmount,
    },

    #[serde(rename = "amount_v2")]
    AmountV2 {
        #[serde(flatten)]
        common: SignablePayloadFieldCommon,
        #[serde(rename = "AmountV2")]
        amount_v2: SignablePayloadFieldAmountV2,
    },

    #[serde(rename = "divider")]
    Divider {
        #[serde(flatten)]
        common: SignablePayloadFieldCommon,
        #[serde(rename = "Divider")]
        divider: SignablePayloadFieldDivider,
    },

    #[serde(rename = "preview_layout")]
    PreviewLayout {
        #[serde(flatten)]
        common: SignablePayloadFieldCommon,
        #[serde(rename = "PreviewLayout")]
        preview_layout: SignablePayloadFieldPreviewLayout,
    },

    #[serde(rename = "list_layout")]
    ListLayout {
        #[serde(flatten)]
        common: SignablePayloadFieldCommon,
        #[serde(rename = "ListLayout")]
        list_layout: SignablePayloadFieldListLayout,
    },

    #[serde(rename = "unknown")]
    Unknown {
        #[serde(flatten)]
        common: SignablePayloadFieldCommon,
        #[serde(rename = "Unknown")]
        unknown: SignablePayloadFieldUnknown,
    },
}

// Trait to ensure all SignablePayloadField variants implement serialization correctly
trait FieldSerializer {
    fn serialize_to_map(&self) -> Result<std::collections::BTreeMap<String, serde_json::Value>, serde_json::Error>;
    fn get_expected_fields(&self) -> Vec<&'static str>;
}

// Macro to help serialize field variants with alphabetical ordering and verification
macro_rules! serialize_field_variant {
    ($fields:expr, $variant_name:literal, $common:expr, $(($field_name:literal, $field_value:expr)),* $(,)?) => {
        // Add common fields
        $fields.insert("FallbackText".to_string(), serde_json::to_value(&$common.fallback_text).unwrap());
        $fields.insert("Label".to_string(), serde_json::to_value(&$common.label).unwrap());
        $fields.insert("Type".to_string(), serde_json::Value::String($variant_name.to_string()));

        // Add variant-specific fields
        $(
            $fields.insert($field_name.to_string(), serde_json::to_value($field_value).unwrap());
        )*
    };
}

// Implementation of FieldSerializer for SignablePayloadField
impl FieldSerializer for SignablePayloadField {
    fn serialize_to_map(&self) -> Result<std::collections::BTreeMap<String, serde_json::Value>, serde_json::Error> {
        let mut fields = std::collections::HashMap::new();

        // Use the macro to serialize each variant - macro uses unwrap() internally
        match self {
            SignablePayloadField::Text { common, text } => {
                serialize_field_variant!(fields, "text", common, ("Text", text));
            },
            SignablePayloadField::TextV2 { common, text_v2 } => {
                serialize_field_variant!(fields, "text_v2", common, ("TextV2", text_v2));
            },
            SignablePayloadField::Address { common, address } => {
                serialize_field_variant!(fields, "address", common, ("Address", address));
            },
            SignablePayloadField::AddressV2 { common, address_v2 } => {
                serialize_field_variant!(fields, "address_v2", common, ("AddressV2", address_v2));
            },
            SignablePayloadField::Number { common, number } => {
                serialize_field_variant!(fields, "number", common, ("Number", number));
            },
            SignablePayloadField::Amount { common, amount } => {
                serialize_field_variant!(fields, "amount", common, ("Amount", amount));
            },
            SignablePayloadField::AmountV2 { common, amount_v2 } => {
                serialize_field_variant!(fields, "amount_v2", common, ("AmountV2", amount_v2));
            },
            SignablePayloadField::Divider { common, divider } => {
                serialize_field_variant!(fields, "divider", common, ("Divider", divider));
            },
            SignablePayloadField::PreviewLayout { common, preview_layout } => {
                serialize_field_variant!(fields, "preview_layout", common, ("PreviewLayout", preview_layout));
            },
            SignablePayloadField::ListLayout { common, list_layout } => {
                serialize_field_variant!(fields, "list_layout", common, ("ListLayout", list_layout));
            },
            SignablePayloadField::Unknown { common, unknown } => {
                serialize_field_variant!(fields, "unknown", common, ("Unknown", unknown));
            },
        }

        // Convert to BTreeMap for alphabetical ordering
        Ok(fields.into_iter().collect())
    }

    fn get_expected_fields(&self) -> Vec<&'static str> {
        let mut base_fields = vec!["FallbackText", "Label", "Type"];

        match self {
            SignablePayloadField::Text { .. } => base_fields.push("Text"),
            SignablePayloadField::TextV2 { .. } => base_fields.push("TextV2"),
            SignablePayloadField::Address { .. } => base_fields.push("Address"),
            SignablePayloadField::AddressV2 { .. } => base_fields.push("AddressV2"),
            SignablePayloadField::Number { .. } => base_fields.push("Number"),
            SignablePayloadField::Amount { .. } => base_fields.push("Amount"),
            SignablePayloadField::AmountV2 { .. } => base_fields.push("AmountV2"),
            SignablePayloadField::Divider { .. } => base_fields.push("Divider"),
            SignablePayloadField::PreviewLayout { .. } => base_fields.push("PreviewLayout"),
            SignablePayloadField::ListLayout { .. } => base_fields.push("ListLayout"),
            SignablePayloadField::Unknown { .. } => base_fields.push("Unknown"),
        }

        base_fields.sort();
        base_fields
    }
}

// Custom Serialize implementation to ensure alphabetical field ordering with verification
impl Serialize for SignablePayloadField {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        // Use the trait method to get serialized fields
        let sorted_map = self.serialize_to_map().map_err(serde::ser::Error::custom)?;

        // Verify that all expected fields are present
        let expected_fields = self.get_expected_fields();
        let actual_fields: Vec<_> = sorted_map.keys().map(|s| s.as_str()).collect();

        // Check for missing fields
        for expected in &expected_fields {
            if !actual_fields.contains(expected) {
                return Err(serde::ser::Error::custom(format!(
                    "Missing expected field '{}' in serialization of {:?}. Expected fields: {:?}, Actual fields: {:?}",
                    expected,
                    std::mem::discriminant(self),
                    expected_fields,
                    actual_fields
                )));
            }
        }

        // Check for unexpected fields (fields that shouldn't be there)
        for actual in &actual_fields {
            if !expected_fields.contains(actual) {
                return Err(serde::ser::Error::custom(format!(
                    "Unexpected field '{}' found in serialization of {:?}. Expected fields: {:?}",
                    actual,
                    std::mem::discriminant(self),
                    expected_fields
                )));
            }
        }

        // Serialize the verified, sorted map
        let mut map_ser = serializer.serialize_map(Some(sorted_map.len()))?;
        for (k, v) in sorted_map {
            map_ser.serialize_entry(&k, &v)?;
        }
        map_ser.end()
    }
}

// Helper methods for the enum
impl SignablePayloadField {
    pub fn fallback_text(&self) -> &String {
        match self {
            SignablePayloadField::Text { common, .. } => &common.fallback_text,
            SignablePayloadField::TextV2 { common, .. } => &common.fallback_text,
            SignablePayloadField::Address { common, .. } => &common.fallback_text,
            SignablePayloadField::AddressV2 { common, .. } => &common.fallback_text,
            SignablePayloadField::Number { common, .. } => &common.fallback_text,
            SignablePayloadField::Amount { common, .. } => &common.fallback_text,
            SignablePayloadField::AmountV2 { common, .. } => &common.fallback_text,
            SignablePayloadField::Divider { common, .. } => &common.fallback_text,
            SignablePayloadField::PreviewLayout { common, .. } => &common.fallback_text,
            SignablePayloadField::ListLayout { common, .. } => &common.fallback_text,
            SignablePayloadField::Unknown { common, .. } => &common.fallback_text,
        }
    }

    pub fn label(&self) -> &String {
        match self {
            SignablePayloadField::Text { common, .. } => &common.label,
            SignablePayloadField::TextV2 { common, .. } => &common.label,
            SignablePayloadField::Address { common, .. } => &common.label,
            SignablePayloadField::AddressV2 { common, .. } => &common.label,
            SignablePayloadField::Number { common, .. } => &common.label,
            SignablePayloadField::Amount { common, .. } => &common.label,
            SignablePayloadField::AmountV2 { common, .. } => &common.label,
            SignablePayloadField::Divider { common, .. } => &common.label,
            SignablePayloadField::PreviewLayout { common, .. } => &common.label,
            SignablePayloadField::ListLayout { common, .. } => &common.label,
            SignablePayloadField::Unknown { common, .. } => &common.label,
        }
    }

    pub fn field_type(&self) -> &str {
        match self {
            SignablePayloadField::Text { .. } => "text",
            SignablePayloadField::TextV2 { .. } => "text_v2",
            SignablePayloadField::Address { .. } => "address",
            SignablePayloadField::AddressV2 { .. } => "address_v2",
            SignablePayloadField::Number { .. } => "number",
            SignablePayloadField::Amount { .. } => "amount",
            SignablePayloadField::AmountV2 { .. } => "amount_v2",
            SignablePayloadField::Divider { .. } => "divider",
            SignablePayloadField::PreviewLayout { .. } => "preview_layout",
            SignablePayloadField::ListLayout { .. } => "list_layout",
            SignablePayloadField::Unknown { .. } => "unknown",
        }
    }
}

// Update all struct definitions to use String instead of NormalString
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct SignablePayloadFieldPreviewLayout {
    #[serde(rename = "Title", skip_serializing_if = "Option::is_none")]
    pub title: Option<SignablePayloadFieldTextV2>,
    #[serde(rename = "Subtitle", skip_serializing_if = "Option::is_none")]
    pub subtitle: Option<SignablePayloadFieldTextV2>,
    #[serde(rename = "Condensed", skip_serializing_if = "Option::is_none")]
    pub condensed: Option<SignablePayloadFieldListLayout>,
    #[serde(rename = "Expanded", skip_serializing_if = "Option::is_none")]
    pub expanded: Option<SignablePayloadFieldListLayout>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct SignablePayloadFieldListLayout {
    #[serde(rename = "Fields")]
    pub fields: Vec<AnnotatedPayloadField>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct SignablePayloadFieldText {
    #[serde(rename = "Text")]
    pub text: String,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct SignablePayloadFieldTextV2 {
    #[serde(rename = "Text")]
    pub text: String,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct SignablePayloadFieldAddress {
    #[serde(rename = "Address")]
    pub address: String,
    #[serde(rename = "Name")]
    pub name: String,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct SignablePayloadFieldAddressV2 {
    #[serde(rename = "Address")]
    pub address: String,
    #[serde(rename = "Name", skip_serializing_if = "is_empty_string")]
    pub name: String,
    #[serde(rename = "Memo", skip_serializing_if = "Option::is_none")]
    pub memo: Option<String>,
    #[serde(rename = "AssetLabel", skip_serializing_if = "is_empty_string")]
    pub asset_label: String,
    #[serde(rename = "BadgeText", skip_serializing_if = "Option::is_none")]
    pub badge_text: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct SignablePayloadFieldNumber {
    #[serde(rename = "Number")]
    pub number: String,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct SignablePayloadFieldAmount {
    #[serde(rename = "Amount")]
    pub amount: String,
    #[serde(rename = "Abbreviation", skip_serializing_if = "Option::is_none")]
    pub abbreviation: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct SignablePayloadFieldAmountV2 {
    #[serde(rename = "Amount")]
    pub amount: String,
    #[serde(rename = "Abbreviation", skip_serializing_if = "Option::is_none")]
    pub abbreviation: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct SignablePayloadFieldDivider {
    #[serde(rename = "Style")]
    pub style: DividerStyle,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct SignablePayloadFieldUnknown {
    #[serde(rename = "Data")]
    pub data: String,
    #[serde(rename = "Explanation")]
    pub explanation: String,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct SignablePayloadFieldStaticAnnotation {
    #[serde(rename = "Text")]
    pub text: String,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct SignablePayloadFieldDynamicAnnotation {
    #[serde(rename = "Type")]
    pub field_type: String,
    #[serde(rename = "ID")]
    pub id: String,
    #[serde(rename = "Params")]
    pub params: Vec<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct AnnotatedPayload {
    #[serde(rename = "Version")]
    pub version: String,
    #[serde(rename = "Title", skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    #[serde(rename = "Subtitle", skip_serializing_if = "Option::is_none")]
    pub subtitle: Option<String>,
    #[serde(rename = "Fields", skip_serializing_if = "Option::is_none")]
    pub fields: Option<Vec<AnnotatedPayloadField>>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct AnnotatedPayloadField {
    #[serde(flatten)]
    pub signable_payload_field: SignablePayloadField,
    #[serde(rename = "StaticAnnotation", skip_serializing_if = "Option::is_none")]
    pub static_annotation: Option<SignablePayloadFieldStaticAnnotation>,
    #[serde(rename = "DynamicAnnotation", skip_serializing_if = "Option::is_none")]
    pub dynamic_annotation: Option<SignablePayloadFieldDynamicAnnotation>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct UserIntent {
    #[serde(rename = "Type")]
    pub intent_type: String,
    #[serde(rename = "Payload")]
    pub payload: Value,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct DividerStyle(String);

impl DividerStyle {
    pub const THIN: DividerStyle = DividerStyle(String::new());
}

impl SignablePayload {
    pub fn new(
        version: i64,
        title: String,
        subtitle: Option<String>,
        fields: Vec<SignablePayloadField>,
        payload_type: String,
    ) -> Self {
        SignablePayload {
            version: version.to_string(),
            title,
            subtitle,
            payload_type,
            fields,
        }
    }

    pub fn to_json(&self) -> Result<String, Box<dyn std::error::Error>> {
        // First convert to a standard JSON value
        let value = serde_json::to_value(self)?;

        // Convert to a completely new object with alphabetically sorted keys
        let sorted_value = sort_json_alphabetically(value);

        // Serialize without pretty-printing and without escape HTML
        let mut buf = Vec::new();
        let formatter = serde_json::ser::CompactFormatter;
        let mut ser = serde_json::Serializer::with_formatter(&mut buf, formatter);
        sorted_value.serialize(&mut ser)?;

        // Convert bytes to string
        Ok(String::from_utf8(buf)?)
    }

    // Add this method for debugging
    pub fn to_pretty_json(&self) -> Result<String, Box<dyn std::error::Error>> {
        let value = serde_json::to_value(self)?;
        let sorted_value = sort_json_alphabetically(value);
        Ok(serde_json::to_string_pretty(&sorted_value)?)
    }
}

// Helper function to recursively sort JSON by keys alphabetically
fn sort_json_alphabetically(value: serde_json::Value) -> serde_json::Value {
    match value {
        serde_json::Value::Object(map) => {
            // Create a BTreeMap (which is sorted by keys)
            let mut sorted_map = std::collections::BTreeMap::new();

            // Insert all entries, recursively sorting nested objects
            for (key, val) in map {
                sorted_map.insert(key, sort_json_alphabetically(val));
            }

            // Convert back to serde_json::Value
            serde_json::Value::Object(serde_json::Map::from_iter(sorted_map))
        }
        serde_json::Value::Array(arr) => {
            // Recursively sort array elements (if they are objects)
            serde_json::Value::Array(arr.into_iter().map(sort_json_alphabetically).collect())
        }
        // Other value types (string, number, boolean, null) don't need sorting
        other => other,
    }
}

impl SignablePayload {
    /// Validates that the payload only contains safe ASCII characters to prevent unicode confusion
    /// This should be called before returning any SignablePayload to ensure consistent character safety
    /// I understand that this might be overly cautious, but it's better to be safe at launch and incrementally open up unicode support later
    pub fn validate_charset(&self) -> Result<(), VisualSignError> {
        let json_str = self.to_json().map_err(|e| {
            VisualSignError::SerializationError(format!(
                "Failed to serialize for validation: {}",
                e
            ))
        })?;

        // Check for unicode escapes
        if json_str.contains("\\u") {
            return Err(VisualSignError::ValidationError(
                "Restricted Characters Detected".to_string(),
            ));
        }

        // Use Rust's built-in ASCII validation
        if !json_str.is_ascii() {
            return Err(VisualSignError::ValidationError(
                "Restricted Characters Detected".to_string(),
            ));
        }

        // Additional validation for printable characters
        for (i, ch) in json_str.char_indices() {
            if !ch.is_ascii_graphic() && !ch.is_ascii_whitespace() {
                return Err(VisualSignError::ValidationError(format!(
                    "JSON output contains non-printable character '{}' (U+{:02X}) at position {}",
                    ch.escape_default(),
                    ch as u32,
                    i
                )));
            }
        }

        Ok(())
    }

    /// Validates and returns the JSON string, ensuring charset safety
    pub fn to_validated_json(&self) -> Result<String, VisualSignError> {
        self.validate_charset()?;
        self.to_json().map_err(|e| {
            VisualSignError::SerializationError(format!("Serialization failed: {}", e))
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;
    use serde_json::json;

    #[test]
    fn test_signable_payload_to_json() {
        let fields = vec![
            SignablePayloadField::Text {
                common: SignablePayloadFieldCommon {
                    fallback_text: "FallbackText1".to_string(),
                    label: "Label1".to_string(),
                },
                text: SignablePayloadFieldText {
                    text: "Text1".to_string(),
                },
            },
            SignablePayloadField::Text {
                common: SignablePayloadFieldCommon {
                    fallback_text: "FallbackText2".to_string(),
                    label: "Label2".to_string(),
                },
                text: SignablePayloadFieldText {
                    text: "Text2".to_string(),
                },
            },
        ];

        let payload = SignablePayload::new(
            1,
            "Test Title".to_string(),
            Some("Test Subtitle".to_string()),
            fields,
            "Test Payload Type".to_string(),
        );

        let json = payload.to_json().unwrap();
        println!("{}", json);
    }

    #[test]
    fn test_eth_user_intent_equivalence() {
        // this is a relatively lazy attempt to keep this consistent with the Go implementation at
        let from_address = "0xYourFromAddress";

        let fields = vec![
            SignablePayloadField::TextV2 {
                common: SignablePayloadFieldCommon {
                    fallback_text: "Ethereum Regnet".to_string(),
                    label: "Network".to_string(),
                },
                text_v2: SignablePayloadFieldTextV2 {
                    text: "Ethereum Regnet".to_string(),
                },
            },
            SignablePayloadField::AddressV2 {
                common: SignablePayloadFieldCommon {
                    fallback_text: from_address.to_string(),
                    label: "From".to_string(),
                },
                address_v2: SignablePayloadFieldAddressV2 {
                    address: from_address.to_string(),
                    name: "".to_string(),
                    memo: None,
                    asset_label: "".to_string(),
                    badge_text: None,
                },
            },
            SignablePayloadField::AddressV2 {
                common: SignablePayloadFieldCommon {
                    fallback_text: "0xb06E442b696513d54B05b5De58494E902E6e08Cb".to_string(),
                    label: "Contract Address".to_string(),
                },
                address_v2: SignablePayloadFieldAddressV2 {
                    address: "0xb06E442b696513d54B05b5De58494E902E6e08Cb".to_string(),
                    name: "".to_string(),
                    memo: None,
                    asset_label: "".to_string(),
                    badge_text: None,
                },
            },
            SignablePayloadField::TextV2 {
                common: SignablePayloadFieldCommon {
                    fallback_text: "0x00".to_string(),
                    label: "Data".to_string(),
                },
                text_v2: SignablePayloadFieldTextV2 {
                    text: "0x00".to_string(),
                },
            },
            SignablePayloadField::AmountV2 {
                common: SignablePayloadFieldCommon {
                    fallback_text: "0 ETH_R".to_string(),
                    label: "Value".to_string(),
                },
                amount_v2: SignablePayloadFieldAmountV2 {
                    amount: "0".to_string(),
                    abbreviation: Some("ETH_R".to_string()),
                },
            },
            SignablePayloadField::AmountV2 {
                common: SignablePayloadFieldCommon {
                    fallback_text: "0.000000000000000004 ETH_R".to_string(),
                    label: "Max Fee".to_string(),
                },
                amount_v2: SignablePayloadFieldAmountV2 {
                    amount: "0.000000000000000004".to_string(),
                    abbreviation: Some("ETH_R".to_string()),
                },
            },
        ];

        let payload =
            SignablePayload::new(15, "Withdraw".to_string(), None, fields, "".to_string());

        let json = payload.to_json().unwrap();
        println!("{}", json);

        let expected_json = json!({
            "Version": "15",
            "Title": "Withdraw",
            "Fields": [
                {
                    "FallbackText": "Ethereum Regnet",
                    "Type": "text_v2",
                    "Label": "Network",
                    "TextV2": {
                        "Text": "Ethereum Regnet"
                    }
                },
                {
                    "FallbackText": "0xYourFromAddress",
                    "Type": "address_v2",
                    "Label": "From",
                    "AddressV2": {
                        "Address": "0xYourFromAddress"
                    }
                },
                {
                    "FallbackText": "0xb06E442b696513d54B05b5De58494E902E6e08Cb",
                    "Type": "address_v2",
                    "Label": "Contract Address",
                    "AddressV2": {
                        "Address": "0xb06E442b696513d54B05b5De58494E902E6e08Cb"
                    }
                },
                {
                    "FallbackText": "0x00",
                    "Type": "text_v2",
                    "Label": "Data",
                    "TextV2": {
                        "Text": "0x00"
                    }
                },
                {
                    "FallbackText": "0 ETH_R",
                    "Type": "amount_v2",
                    "Label": "Value",
                    "AmountV2": {
                        "Amount": "0",
                        "Abbreviation": "ETH_R"
                    }
                },
                {
                    "FallbackText": "0.000000000000000004 ETH_R",
                    "Type": "amount_v2",
                    "Label": "Max Fee",
                    "AmountV2": {
                        "Amount": "0.000000000000000004",
                        "Abbreviation": "ETH_R"
                    }
                }
            ]
        });

        let generated_json: Value = serde_json::from_str(&json).unwrap();
        assert_eq!(generated_json, expected_json);
    }

    #[test]
    fn test_extensibility_with_new_field_type() {
        // This test demonstrates how easy it is to add a new field type
        // by actually implementing and testing a new variant

        // Define new field type structs (these would normally be added to the main code)
        #[derive(Serialize, Debug, Clone, PartialEq, Eq)]
        struct TestCurrencyField {
            #[serde(rename = "CurrencyCode")]
            currency_code: String,
            #[serde(rename = "Symbol")]
            symbol: String,
        }

        // Create a test enum that extends SignablePayloadField with a new Currency variant
        #[derive(Debug, Clone, PartialEq, Eq)]
        enum ExtendedSignablePayloadField {
            // Include an existing variant to test alongside the new one
            TextV2 {
                common: SignablePayloadFieldCommon,
                text_v2: SignablePayloadFieldTextV2,
            },
            // New Currency variant - this shows how easy it is to add
            Currency {
                common: SignablePayloadFieldCommon,
                currency: TestCurrencyField,
            },
        }

        // Implement Serialize for the extended enum using our macro
        impl Serialize for ExtendedSignablePayloadField {
            fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
            where
                S: Serializer,
            {
                let mut fields = std::collections::HashMap::new();

                match self {
                    ExtendedSignablePayloadField::TextV2 { common, text_v2 } => {
                        serialize_field_variant!(fields, "text_v2", common, ("TextV2", text_v2));
                    },
                    // Adding the new Currency variant is just one line!
                    ExtendedSignablePayloadField::Currency { common, currency } => {
                        serialize_field_variant!(fields, "currency", common, ("Currency", currency));
                    },
                }

                let sorted_map: std::collections::BTreeMap<String, serde_json::Value> = fields.into_iter().collect();
                let mut map_ser = serializer.serialize_map(Some(sorted_map.len()))?;
                for (k, v) in sorted_map {
                    map_ser.serialize_entry(&k, &v)?;
                }
                map_ser.end()
            }
        }

        // Test the new Currency field type
        let currency_field = ExtendedSignablePayloadField::Currency {
            common: SignablePayloadFieldCommon {
                fallback_text: "USD ($)".to_string(),
                label: "Payment Currency".to_string(),
            },
            currency: TestCurrencyField {
                currency_code: "USD".to_string(),
                symbol: "$".to_string(),
            },
        };

        let json = serde_json::to_string(&currency_field).unwrap();
        println!("New Currency Field JSON: {}", json);

        // Verify alphabetical ordering
        let value: serde_json::Value = serde_json::from_str(&json).unwrap();
        if let serde_json::Value::Object(map) = value {
            let keys: Vec<_> = map.keys().cloned().collect();
            println!("Currency Field Keys in order: {:?}", keys);

            // Expected order: Currency, FallbackText, Label, Type
            assert_eq!(keys, vec!["Currency", "FallbackText", "Label", "Type"]);
        } else {
            panic!("Expected JSON object");
        }

        // Test that TextV2 still works correctly alongside the new field
        let text_field = ExtendedSignablePayloadField::TextV2 {
            common: SignablePayloadFieldCommon {
                fallback_text: "Test Text".to_string(),
                label: "Test Label".to_string(),
            },
            text_v2: SignablePayloadFieldTextV2 {
                text: "Hello World".to_string(),
            },
        };

        let json2 = serde_json::to_string(&text_field).unwrap();
        println!("TextV2 Field JSON: {}", json2);

        let value2: serde_json::Value = serde_json::from_str(&json2).unwrap();
        if let serde_json::Value::Object(map) = value2 {
            let keys: Vec<_> = map.keys().cloned().collect();
            println!("TextV2 Field Keys in order: {:?}", keys);

            // Expected order: FallbackText, Label, TextV2, Type
            assert_eq!(keys, vec!["FallbackText", "Label", "TextV2", "Type"]);
        } else {
            panic!("Expected JSON object");
        }

        println!("✅ Successfully demonstrated adding new field type with automatic alphabetical ordering!");
    }

    #[test]
    fn test_multiple_new_field_types_extensibility() {
        // This test demonstrates adding multiple new field types at once
        // showing how the macro approach scales easily

        #[derive(Serialize, Debug, Clone, PartialEq, Eq)]
        struct TestDateTimeField {
            #[serde(rename = "DateTime")]
            date_time: String,
            #[serde(rename = "Format")]
            format: String,
        }

        #[derive(Serialize, Debug, Clone, PartialEq, Eq)]
        struct TestPercentageField {
            #[serde(rename = "Value")]
            value: String,
            #[serde(rename = "Precision")]
            precision: u32,
        }

        #[derive(Serialize, Debug, Clone, PartialEq, Eq)]
        struct TestLocationField {
            #[serde(rename = "Latitude")]
            latitude: String,  // Use string to avoid float comparison issues in tests
            #[serde(rename = "Longitude")]
            longitude: String,
            #[serde(rename = "Address")]
            address: String,
        }

        // Extended enum with multiple new field types
        #[derive(Debug, Clone, PartialEq, Eq)]
        enum MultiExtendedSignablePayloadField {
            DateTime {
                common: SignablePayloadFieldCommon,
                date_time: TestDateTimeField,
            },
            Percentage {
                common: SignablePayloadFieldCommon,
                percentage: TestPercentageField,
            },
            Location {
                common: SignablePayloadFieldCommon,
                location: TestLocationField,
            },
        }

        impl Serialize for MultiExtendedSignablePayloadField {
            fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
            where
                S: Serializer,
            {
                let mut fields = std::collections::HashMap::new();

                match self {
                    // Each new field type is just one line using the macro!
                    MultiExtendedSignablePayloadField::DateTime { common, date_time } => {
                        serialize_field_variant!(fields, "date_time", common, ("DateTime", date_time));
                    },
                    MultiExtendedSignablePayloadField::Percentage { common, percentage } => {
                        serialize_field_variant!(fields, "percentage", common, ("Percentage", percentage));
                    },
                    MultiExtendedSignablePayloadField::Location { common, location } => {
                        serialize_field_variant!(fields, "location", common, ("Location", location));
                    },
                }

                let sorted_map: std::collections::BTreeMap<String, serde_json::Value> = fields.into_iter().collect();
                let mut map_ser = serializer.serialize_map(Some(sorted_map.len()))?;
                for (k, v) in sorted_map {
                    map_ser.serialize_entry(&k, &v)?;
                }
                map_ser.end()
            }
        }

        // Test DateTime field
        let datetime_field = MultiExtendedSignablePayloadField::DateTime {
            common: SignablePayloadFieldCommon {
                fallback_text: "2024-01-15 14:30:00 UTC".to_string(),
                label: "Transaction Time".to_string(),
            },
            date_time: TestDateTimeField {
                date_time: "2024-01-15T14:30:00Z".to_string(),
                format: "ISO8601".to_string(),
            },
        };

        let json = serde_json::to_string(&datetime_field).unwrap();
        let value: serde_json::Value = serde_json::from_str(&json).unwrap();
        if let serde_json::Value::Object(map) = value {
            let keys: Vec<_> = map.keys().cloned().collect();
            // Expected order: DateTime, FallbackText, Label, Type
            assert_eq!(keys, vec!["DateTime", "FallbackText", "Label", "Type"]);
        }

        // Test Percentage field
        let percentage_field = MultiExtendedSignablePayloadField::Percentage {
            common: SignablePayloadFieldCommon {
                fallback_text: "15.50%".to_string(),
                label: "Fee Rate".to_string(),
            },
            percentage: TestPercentageField {
                value: "15.50".to_string(),
                precision: 2,
            },
        };

        let json2 = serde_json::to_string(&percentage_field).unwrap();
        let value2: serde_json::Value = serde_json::from_str(&json2).unwrap();
        if let serde_json::Value::Object(map) = value2 {
            let keys: Vec<_> = map.keys().cloned().collect();
            // Expected order: FallbackText, Label, Percentage, Type
            assert_eq!(keys, vec!["FallbackText", "Label", "Percentage", "Type"]);
        }

        // Test Location field
        let location_field = MultiExtendedSignablePayloadField::Location {
            common: SignablePayloadFieldCommon {
                fallback_text: "New York, NY (40.7128, -74.0060)".to_string(),
                label: "Transaction Location".to_string(),
            },
            location: TestLocationField {
                latitude: "40.7128".to_string(),
                longitude: "-74.0060".to_string(),
                address: "New York, NY".to_string(),
            },
        };

        let json3 = serde_json::to_string(&location_field).unwrap();
        let value3: serde_json::Value = serde_json::from_str(&json3).unwrap();
        if let serde_json::Value::Object(map) = value3 {
            let keys: Vec<_> = map.keys().cloned().collect();
            // Expected order: FallbackText, Label, Location, Type
            assert_eq!(keys, vec!["FallbackText", "Label", "Location", "Type"]);
        }

        println!("✅ Successfully demonstrated adding multiple new field types easily!");
        println!("   - DateTime field: automatic alphabetical ordering");
        println!("   - Percentage field: automatic alphabetical ordering");
        println!("   - Location field: automatic alphabetical ordering");
        println!("   - Each new type required only ONE line of macro code!");
    }

    #[test]
    fn test_field_completeness_verification() {
        // This test demonstrates that the new approach catches missing or incorrect field serialization

        // Create a test enum with intentionally incomplete serialization to show error detection
        #[derive(Debug, Clone, PartialEq, Eq)]
        enum IncompleteTestField {
            TestVariant {
                common: SignablePayloadFieldCommon,
                test_data: String,
            },
        }

        // Implement trait with MISSING field on purpose
        impl FieldSerializer for IncompleteTestField {
            fn serialize_to_map(&self) -> Result<std::collections::BTreeMap<String, serde_json::Value>, serde_json::Error> {
                let mut fields = std::collections::HashMap::new();
                match self {
                    IncompleteTestField::TestVariant { common, test_data: _ } => {
                        // Intentionally FORGET to serialize test_data to demonstrate detection
                        fields.insert("FallbackText".to_string(), serde_json::to_value(&common.fallback_text).unwrap());
                        fields.insert("Label".to_string(), serde_json::to_value(&common.label).unwrap());
                        fields.insert("Type".to_string(), serde_json::Value::String("test".to_string()));
                        // Missing: "TestData" field!
                    },
                }
                Ok(fields.into_iter().collect())
            }

            fn get_expected_fields(&self) -> Vec<&'static str> {
                vec!["FallbackText", "Label", "TestData", "Type"] // Expects TestData but we didn't serialize it
            }
        }

        // Implement Serialize using the verification logic
        impl Serialize for IncompleteTestField {
            fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
            where
                S: Serializer,
            {
                let sorted_map = self.serialize_to_map().map_err(serde::ser::Error::custom)?;
                let expected_fields = self.get_expected_fields();
                let actual_fields: Vec<_> = sorted_map.keys().map(|s| s.as_str()).collect();

                // Check for missing fields
                for expected in &expected_fields {
                    if !actual_fields.contains(expected) {
                        return Err(serde::ser::Error::custom(format!(
                            "Missing expected field '{}'. Expected: {:?}, Actual: {:?}",
                            expected, expected_fields, actual_fields
                        )));
                    }
                }

                let mut map_ser = serializer.serialize_map(Some(sorted_map.len()))?;
                for (k, v) in sorted_map {
                    map_ser.serialize_entry(&k, &v)?;
                }
                map_ser.end()
            }
        }

        // Test that serialization fails when fields are missing
        let incomplete_field = IncompleteTestField::TestVariant {
            common: SignablePayloadFieldCommon {
                fallback_text: "Test".to_string(),
                label: "Test Label".to_string(),
            },
            test_data: "This should be serialized but isn't".to_string(),
        };

        let result = serde_json::to_string(&incomplete_field);

        // This should FAIL because we forgot to serialize TestData
        assert!(result.is_err(), "Expected serialization to fail due to missing TestData field");

        let error_msg = result.unwrap_err().to_string();
        assert!(error_msg.contains("Missing expected field 'TestData'"),
                "Error should mention missing TestData field, got: {}", error_msg);

        println!("✅ Successfully caught missing field serialization!");
        println!("   Error: {}", error_msg);
    }

    #[test]
    fn test_field_completeness_verification_with_correct_implementation() {
        // This test shows a CORRECT implementation that passes verification

        #[derive(Serialize, Debug, Clone, PartialEq, Eq)]
        struct TestDataStruct {
            #[serde(rename = "Data")]
            data: String,
        }

        #[derive(Debug, Clone, PartialEq, Eq)]
        enum CompleteTestField {
            TestVariant {
                common: SignablePayloadFieldCommon,
                test_data: TestDataStruct,
            },
        }

        // Implement trait with ALL required fields
        impl FieldSerializer for CompleteTestField {
            fn serialize_to_map(&self) -> Result<std::collections::BTreeMap<String, serde_json::Value>, serde_json::Error> {
                let mut fields = std::collections::HashMap::new();
                match self {
                    CompleteTestField::TestVariant { common, test_data } => {
                        // Correctly serialize ALL fields
                        serialize_field_variant!(fields, "test", common, ("TestData", test_data));
                    },
                }
                Ok(fields.into_iter().collect())
            }

            fn get_expected_fields(&self) -> Vec<&'static str> {
                vec!["FallbackText", "Label", "TestData", "Type"] // Matches what we actually serialize
            }
        }

        // Implement Serialize using the verification logic
        impl Serialize for CompleteTestField {
            fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
            where
                S: Serializer,
            {
                let sorted_map = self.serialize_to_map().map_err(serde::ser::Error::custom)?;
                let expected_fields = self.get_expected_fields();
                let actual_fields: Vec<_> = sorted_map.keys().map(|s| s.as_str()).collect();

                for expected in &expected_fields {
                    if !actual_fields.contains(expected) {
                        return Err(serde::ser::Error::custom(format!(
                            "Missing expected field '{}'. Expected: {:?}, Actual: {:?}",
                            expected, expected_fields, actual_fields
                        )));
                    }
                }

                let mut map_ser = serializer.serialize_map(Some(sorted_map.len()))?;
                for (k, v) in sorted_map {
                    map_ser.serialize_entry(&k, &v)?;
                }
                map_ser.end()
            }
        }

        // Test that serialization succeeds when all fields are present
        let complete_field = CompleteTestField::TestVariant {
            common: SignablePayloadFieldCommon {
                fallback_text: "Test".to_string(),
                label: "Test Label".to_string(),
            },
            test_data: TestDataStruct {
                data: "This is properly serialized".to_string(),
            },
        };

        let result = serde_json::to_string(&complete_field);
        assert!(result.is_ok(), "Expected serialization to succeed: {:?}", result);

        let json = result.unwrap();
        let value: serde_json::Value = serde_json::from_str(&json).unwrap();
        if let serde_json::Value::Object(map) = value {
            let keys: Vec<_> = map.keys().cloned().collect();
            // Verify alphabetical ordering: FallbackText, Label, TestData, Type
            assert_eq!(keys, vec!["FallbackText", "Label", "TestData", "Type"]);
        }

        println!("✅ Correctly implemented field serialization passed verification!");
        println!("   JSON: {}", json);
    }

    #[test]
    fn test_original_signable_payload_field_verification() {
        // Test that the original SignablePayloadField enum passes all verification
        // This confirms our refactoring maintains correctness and adds verification

        let test_fields = vec![
            // TextV2
            SignablePayloadField::TextV2 {
                common: SignablePayloadFieldCommon {
                    fallback_text: "Test Text".to_string(),
                    label: "Text Field".to_string(),
                },
                text_v2: SignablePayloadFieldTextV2 {
                    text: "Hello World".to_string(),
                },
            },
            // AmountV2
            SignablePayloadField::AmountV2 {
                common: SignablePayloadFieldCommon {
                    fallback_text: "100 USD".to_string(),
                    label: "Amount Field".to_string(),
                },
                amount_v2: SignablePayloadFieldAmountV2 {
                    amount: "100".to_string(),
                    abbreviation: Some("USD".to_string()),
                },
            },
            // Address
            SignablePayloadField::Address {
                common: SignablePayloadFieldCommon {
                    fallback_text: "0x123...abc".to_string(),
                    label: "Address Field".to_string(),
                },
                address: SignablePayloadFieldAddress {
                    address: "0x123abc".to_string(),
                    name: "Test Address".to_string(),
                },
            },
        ];

        for (i, field) in test_fields.iter().enumerate() {
            // Verify each field type serializes correctly with verification
            let result = serde_json::to_string(field);
            assert!(result.is_ok(),
                    "Field {} should serialize successfully: {:?}", i, result);

            let json = result.unwrap();
            let value: serde_json::Value = serde_json::from_str(&json).unwrap();

            // Verify alphabetical ordering
            if let serde_json::Value::Object(map) = value {
                let keys: Vec<_> = map.keys().cloned().collect();
                let mut expected_keys = keys.clone();
                expected_keys.sort();

                assert_eq!(keys, expected_keys,
                          "Fields should be in alphabetical order for field {}: got {:?}", i, keys);

                // Verify all expected fields are present
                let expected_field_count = field.get_expected_fields().len();
                assert_eq!(keys.len(), expected_field_count,
                          "Field {} should have exactly {} fields: {:?}", i, expected_field_count, keys);

                println!("✅ Field {} verified: {} fields in alphabetical order: {:?}",
                        i, keys.len(), keys);
            }
        }

        println!("✅ All SignablePayloadField variants pass verification with alphabetical ordering!");
    }

    #[test]
    fn test_field_alphabetical_ordering() {
        // Test that fields within SignablePayloadField are ordered alphabetically

        // Test TextV2 field
        let field = SignablePayloadField::TextV2 {
            common: SignablePayloadFieldCommon {
                fallback_text: "Test Fallback".to_string(),
                label: "Test Label".to_string(),
            },
            text_v2: SignablePayloadFieldTextV2 {
                text: "Test Text".to_string(),
            },
        };

        let json = serde_json::to_string(&field).unwrap();
        println!("TextV2 Field JSON: {}", json);

        // Parse back to check field order
        let value: serde_json::Value = serde_json::from_str(&json).unwrap();
        if let serde_json::Value::Object(map) = value {
            let keys: Vec<_> = map.keys().cloned().collect();
            println!("Keys in order: {:?}", keys);

            // Expected order: FallbackText, Label, TextV2, Type
            assert_eq!(keys, vec!["FallbackText", "Label", "TextV2", "Type"]);
        } else {
            panic!("Expected JSON object");
        }

        // Test AmountV2 field
        let field2 = SignablePayloadField::AmountV2 {
            common: SignablePayloadFieldCommon {
                fallback_text: "0 ETH".to_string(),
                label: "Value".to_string(),
            },
            amount_v2: SignablePayloadFieldAmountV2 {
                amount: "0".to_string(),
                abbreviation: Some("ETH".to_string()),
            },
        };

        let json2 = serde_json::to_string(&field2).unwrap();
        println!("AmountV2 Field JSON: {}", json2);

        let value2: serde_json::Value = serde_json::from_str(&json2).unwrap();
        if let serde_json::Value::Object(map) = value2 {
            let keys: Vec<_> = map.keys().cloned().collect();
            println!("Keys in order: {:?}", keys);

            // Expected order: AmountV2, FallbackText, Label, Type
            assert_eq!(keys, vec!["AmountV2", "FallbackText", "Label", "Type"]);
        } else {
            panic!("Expected JSON object");
        }
    }

    #[test]
    fn test_alphabetical_sorting() {
        let payload = SignablePayload::new(
            1,
            "Z_Title".to_string(),          // Starts with Z
            Some("A_Subtitle".to_string()), // Starts with A
            vec![],                         // Empty fields
            "M_PayloadType".to_string(),    // Starts with M
        );

        let json = payload.to_json().unwrap();
        assert_sorted_alphabetically(json);

        // Lets try using serde_json to serialize the payload and ensure that ordering is still same
        // this is more to ensure that engineer isn't changing the order of fields
        let serde_default_json = serde_json::to_string(&payload).unwrap();
        assert_sorted_alphabetically(serde_default_json);
    }

    fn assert_sorted_alphabetically(json: String) {
        println!("Sorted JSON: {}", json);
        // ensure that ordering si preserved when using to_json()
        let pos_fields = json.find("Fields").unwrap_or(0);
        let pos_payload = json.find("PayloadType").unwrap_or(0);
        let pos_subtitle = json.find("Subtitle").unwrap_or(0);
        let pos_title = json.find("Title").unwrap_or(0);
        let pos_version = json.find("Version").unwrap_or(0);

        assert!(
            pos_fields < pos_payload,
            "Fields should come before PayloadType"
        );
        assert!(
            pos_payload < pos_subtitle,
            "PayloadType should come before Subtitle"
        );
        assert!(
            pos_subtitle < pos_title,
            "Subtitle should come before Title"
        );
        assert!(pos_title < pos_version, "Title should come before Version");
    }
}
