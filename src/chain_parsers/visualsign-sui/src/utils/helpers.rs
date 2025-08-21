#![cfg(test)]

use crate::transaction_string_to_visual_sign;
use visualsign::SignablePayload;
use visualsign::vsptrait::VisualSignOptions;

pub fn payload_from_b64(data: &str) -> SignablePayload {
    transaction_string_to_visual_sign(
        data,
        VisualSignOptions {
            decode_transfers: true,
            transaction_name: None,
        },
    )
    .expect("Failed to visualize tx commands")
}
