use crate::{SignablePayload, SignablePayloadField};

pub fn assert_has_field(payload: &SignablePayload, label: &str) {
    let (found, _) = check_signable_payload(payload, label);
    assert!(found, "Should have a {label} field");
}

pub fn assert_has_field_with_context(payload: &SignablePayload, label: &str, context: &str) {
    let (found, _) = check_signable_payload(payload, label);
    assert!(found, "Should have a {label} field in {context}");
}

pub fn assert_has_field_with_value(payload: &SignablePayload, label: &str, expected_value: &str) {
    let (found, values) = check_signable_payload(payload, label);
    assert!(
        found,
        "Should have a {label} field with value {expected_value}"
    );
    assert!(
        values.contains(&expected_value.to_string()),
        "Should have a {label} field with value {expected_value}. Actual values: {:?}",
        values
    );
}

pub fn assert_has_field_with_value_with_context(
    payload: &SignablePayload,
    label: &str,
    expected_value: &str,
    context: &str,
) {
    let (found, values) = check_signable_payload(payload, label);
    assert!(
        found,
        "Should have a {label} field with value {expected_value} in {context}"
    );
    assert!(
        values.iter().all(|x| x.eq(expected_value)),
        "Should have a {label} field with value {expected_value}. Actual values: {:?} (use `assert_has_fields_with_values_with_context` if there could be different expected values) in {context}",
        values
    );
}

pub fn assert_has_fields_with_values_with_context(
    payload: &SignablePayload,
    label: &str,
    expected_values: &[String],
    context: &str,
) {
    let (found, values) = check_signable_payload(payload, label);
    assert!(found, "Should have at least one {label} field in {context}");

    let expected: Vec<String> = expected_values.iter().map(|s| s.to_string()).collect();
    assert_eq!(
        values.len(),
        expected.len(),
        "Should have {} {label} field(s) in {context}. Actual values: {:?}",
        expected.len(),
        values
    );

    assert_eq!(
        values, expected,
        "Mismatch in {label} field values in {context}. Expected: {:?}, Actual: {:?}",
        expected, values
    );
}

pub fn check_signable_payload(payload: &SignablePayload, label: &str) -> (bool, Vec<String>) {
    let mut all_values: Vec<String> = Vec::new();

    for field in payload.fields.iter() {
        let (_, mut values) = check_signable_payload_field(field, label);
        all_values.append(&mut values);
    }

    (!all_values.is_empty(), all_values)
}

pub fn check_signable_payload_field(
    field: &SignablePayloadField,
    label: &str,
) -> (bool, Vec<String>) {
    match field {
        SignablePayloadField::Text { common, text } => {
            if common.label == label {
                (true, vec![text.text.to_string()])
            } else {
                (false, Vec::new())
            }
        }
        SignablePayloadField::TextV2 { common, text_v2 } => {
            if common.label == label {
                (true, vec![text_v2.text.to_string()])
            } else {
                (false, Vec::new())
            }
        }
        SignablePayloadField::Address { common, address } => {
            if common.label == label {
                (true, vec![address.address.to_string()])
            } else {
                (false, Vec::new())
            }
        }
        SignablePayloadField::AddressV2 { common, address_v2 } => {
            if common.label == label {
                (true, vec![address_v2.address.to_string()])
            } else {
                (false, Vec::new())
            }
        }
        SignablePayloadField::Number { common, number } => {
            if common.label == label {
                (true, vec![number.number.to_string()])
            } else {
                (false, Vec::new())
            }
        }
        SignablePayloadField::Amount { common, amount } => {
            if common.label == label {
                (true, vec![amount.amount.to_string()])
            } else {
                (false, Vec::new())
            }
        }
        SignablePayloadField::AmountV2 { common, amount_v2 } => {
            if common.label == label {
                (true, vec![amount_v2.amount.to_string()])
            } else {
                (false, Vec::new())
            }
        }
        SignablePayloadField::PreviewLayout {
            preview_layout,
            common,
        } => {
            let mut values: Vec<String> = Vec::new();

            if common.label == label {
                values.push(common.fallback_text.to_string());
            }

            if let Some(condensed) = preview_layout.condensed.as_ref() {
                for field in condensed.fields.iter() {
                    let (_, mut inner_values) =
                        check_signable_payload_field(&field.signable_payload_field, label);
                    values.append(&mut inner_values);
                }
            }

            if let Some(expanded) = preview_layout.expanded.as_ref() {
                for field in expanded.fields.iter() {
                    let (_, mut inner_values) =
                        check_signable_payload_field(&field.signable_payload_field, label);
                    values.append(&mut inner_values);
                }
            }

            (!values.is_empty(), values)
        }
        SignablePayloadField::ListLayout {
            list_layout,
            common,
        } => {
            let mut values: Vec<String> = Vec::new();

            if common.label == label {
                values.push(common.fallback_text.to_string());
            }

            for field in list_layout.fields.iter() {
                let (_, mut inner_values) =
                    check_signable_payload_field(&field.signable_payload_field, label);
                values.append(&mut inner_values);
            }

            (!values.is_empty(), values)
        }
        _ => (false, Vec::new()),
    }
}
