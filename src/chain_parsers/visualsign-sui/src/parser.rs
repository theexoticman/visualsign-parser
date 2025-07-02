use crate::module_resolver::SuiModuleResolver;

use anyhow::{Result, anyhow};
use base64::{Engine, engine::general_purpose};
use std::str::FromStr;

use move_bytecode_utils::module_cache::SyncModuleCache;

use sui_json_rpc_types::SuiTransactionBlock;
use sui_types::transaction::SenderSignedData;

#[derive(Debug, Clone, PartialEq)]
pub enum TransactionEncoding {
    Base64,
}

impl FromStr for TransactionEncoding {
    type Err = &'static str;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "base64" => Ok(TransactionEncoding::Base64),
            _ => Err("Invalid encoding. Use 'base64'"),
        }
    }
}

pub fn parse_sui_transaction(
    unsigned_tx: String,
    encoding: TransactionEncoding,
) -> Result<SuiTransactionBlock> {
    if unsigned_tx.is_empty() {
        return Err(anyhow!("Transaction is empty"));
    }

    decode_transaction_data(&unsigned_tx, &encoding)
}

pub fn decode_transaction_data(
    data: &str,
    encoding: &TransactionEncoding,
) -> Result<SuiTransactionBlock> {
    let bcs_data = match encoding {
        TransactionEncoding::Base64 => general_purpose::STANDARD
            .decode(data)
            .map_err(|e| anyhow!("Invalid base64 encoding: {}", e)),
    }?;

    let sender_signed_data = bcs::from_bytes::<SenderSignedData>(&bcs_data)?;
    let module_cache = SyncModuleCache::new(SuiModuleResolver);

    SuiTransactionBlock::try_from(sender_signed_data, &module_cache)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_empty_transaction() {
        let result = parse_sui_transaction("".to_string(), TransactionEncoding::Base64);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().to_string(), "Transaction is empty");
    }

    #[test]
    fn test_parse_basic_transaction_info() {
        let test_data = "AQAAAAAAAgAI6AMAAAAAAAAAIKHjrlUcKr48a86iLT8ZNWpkcIbWvVasDQnk7u0GKQt2AgIAAQEAAAEBAgAAAQEA1ukuAC4mw6+yCIABwbWCC2TyvDUb/aWiNCrL+fXBysIBy0he+AoLr5B5piHELIsMtlzpmG4cgf0W7ogDjwBKWu3zD9AUAAAAACB0zCGEALsfD5u98y58qbKGIiXkCtDxxN2Pu+r/HyOy1tbpLgAuJsOvsgiAAcG1ggtk8rw1G/2lojQqy/n1wcrC6AMAAAAAAABAS0wAAAAAAAABYQBMegviWYFsLskcYMnTIhZRxiZkET3j2RqtgG1g7f1/EuPjfCHfTvgDqVys+AA6jLWojR35eW4HoOh8qURdshkADNDs6YjOg+HDmdMLe0zMuMDJKqzwIYg08CT6mXiLc2Y=";
        let result = parse_sui_transaction(test_data.to_string(), TransactionEncoding::Base64);

        assert!(result.is_ok());
    }
}
