use crate::commands::utils::{get_amount, CoinObject};

use sui_types::base_types::{ObjectID, SuiAddress};

use sui_json_rpc_types::SuiCommand::{MoveCall, SplitCoins};
use sui_json_rpc_types::{
    SuiTransactionBlockData,
    SuiTransactionBlockDataAPI, SuiTransactionBlockKind,
};

#[derive(Debug, Clone)]
pub struct StakeAndWithdrawInfo {
    pub is_stake: bool,
    pub is_withdraw: bool,
    pub sender: SuiAddress,
    pub amount: u64,
    pub coin_object: CoinObject,
}

// TODO: All transaction with stake/withdraw don't contain multiple commands (few stakes or few withdraws), so we can simplify this
// to a single operation check. For now Vector is used to maintain consistency with other detection functions.
// We can refactor this to support multiple commands, e.g [SplitCoins, SplitCoins, MoveCall, MoveCall] for stakes or 
// [MoveCall, MoveCall...] for withdraws.
pub fn detect_stake_withdraw_from_transaction(
    tx_data: &SuiTransactionBlockData,
) -> Vec<Result<StakeAndWithdrawInfo, String>> {
    let SuiTransactionBlockKind::ProgrammableTransaction(transaction) = &tx_data.transaction()
    else {
        return vec![];
    };

    let mut result: StakeAndWithdrawInfo = StakeAndWithdrawInfo {
        is_stake: false,
        is_withdraw: false,
        sender: SuiAddress::default(),
        amount: 0,
        coin_object: CoinObject::Sui,
    };

    for (_, command) in transaction.commands.iter().enumerate() {
        match command {
            SplitCoins(_, amounts) => {
                let Some(stake_amount) = get_amount(transaction, amounts) else {
                    continue;
                };
                result.amount += stake_amount;
            }
            MoveCall(pwc) => {
                let is_package_correct = pwc.package == ObjectID::from_hex_literal("0x3").unwrap();

                if pwc.function.contains("add_stake") && is_package_correct {
                    result.is_stake = true;
                    result.sender = *tx_data.sender();
                }
                if pwc.function.contains("withdraw_stake") && is_package_correct {
                    result.is_withdraw = true;
                    result.sender = *tx_data.sender();
                }
            }
            _ => {}
        }
    }

    vec![Ok(result)]
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::visualsign::decode_transaction;

    use std::str::FromStr;
    use move_bytecode_utils::module_cache::SyncModuleCache;
    use visualsign::encodings::SupportedEncodings;
    use crate::module_resolver::SuiModuleResolver;

    #[test]
    fn test_detect_stake_from_transaction() -> anyhow::Result<()> {
        // https://suiscan.xyz/testnet/tx/EVRX1gVBobjWkPkQxMS7HmfyJMRsN731vKLr55Hs8CsQ
        let test_data = "AQAAAAAAAwAIAMqaOwAAAAABAQAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAFAQAAAAAAAAABACAq3llEhft5Vha3QVbJEJfsUXoFrEiDZN060exfU22z9AICAAEBAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAMKc3VpX3N5c3RlbRFyZXF1ZXN0X2FkZF9zdGFrZQADAQEAAgAAAQIAcejdgpB8Jf6jiDji5UojNgcnMe+hMYxgeQkXQ2pt0dQBNjBvtFXbz1t8Pmbnk9gXceo5l/swaubPVVoeS/2sxKA4pTMeAAAAACCPnPbp/cGLpqtryff+BH6155y1IG5DmU0k+CxCGau2Y3Ho3YKQfCX+o4g44uVKIzYHJzHvoTGMYHkJF0NqbdHU6AMAAAAAAADIPLsAAAAAAAABYQCpeJeKqeM2LwCf9q7Gqfn8bAtxwHrdxiniFvHAgmtyhtvWrwJEcofNqEVU23wLISwNAzuk9FUSMQCZWRJ9MCMGo4IlQs0JwIL5V+rQ1Sbdl7UIPeTFvAIk84iCZohpLR4=";

        let result = decode_transaction(test_data, SupportedEncodings::Base64).unwrap();
        let block_data = SuiTransactionBlockData::try_from_with_module_cache(
            result.clone(),
            &SyncModuleCache::new(SuiModuleResolver),
        )?;
        let result = detect_stake_withdraw_from_transaction(&block_data);

        assert_eq!(result.len(), 1);

        let transfer = result.first().unwrap().as_ref().unwrap();
        assert_eq!(transfer.coin_object, CoinObject::Sui);
        assert_eq!(transfer.amount, 1000000000);
        assert_eq!(
            transfer.sender,
            SuiAddress::from_str(
                "0x71e8dd82907c25fea38838e2e54a2336072731efa1318c60790917436a6dd1d4"
            )?
        );

        Ok(())
    }

    #[test]
    fn test_detect_withdraw_from_transaction() -> anyhow::Result<()> {
        // https://suiscan.xyz/testnet/tx/BAj1utF9V4GRfoEeg26tdEi2ohbxuzvy1MT8g3kTqqmA
        let test_data = "AQAAAAAAAgEBAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAUBAAAAAAAAAAEBABCSxc7L2q13MuUn/cVYZn+Md5VRqxQ67ZyG61PRGaVzD1dHHgAAAAAgytzn4J8UniWh8KZ0aYY3zWxFNy713Zt8KEJ5CsbDnmYBAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAADCnN1aV9zeXN0ZW0WcmVxdWVzdF93aXRoZHJhd19zdGFrZQACAQAAAQEAEXUn/c8vX26C+kmcA5je8mQ1UNY/foMRJF119GNfBDUBlYxs2uu+QYsm8zVrmO6l20gQkX4/QKYQ1SyX63NssJEPV0ceAAAAACDoQVCKOqAWv5mQEhv+yUlsVzk0bztsQnRGRuYmWrKYIRF1J/3PL19ugvpJnAOY3vJkNVDWP36DESRddfRjXwQ16AMAAAAAAADgFKMAAAAAAAABYQBnu1E8y8zTkJvglRpPw5aGwqCGuImyZD/ZoeURm7KCDEqcQVsSbspNK6Ear2Jrihv3vvZGbC9G0hiCPAKQnP4G6V0Ux7kG1mCQVW4PzmXtO3EWezfgwZE2hX01tE8IK6c=";

        let result = decode_transaction(test_data, SupportedEncodings::Base64).unwrap();
        let block_data = SuiTransactionBlockData::try_from_with_module_cache(
            result.clone(),
            &SyncModuleCache::new(SuiModuleResolver),
        )?;
        let result = detect_stake_withdraw_from_transaction(&block_data);
        
        assert_eq!(result.len(), 1);

        let transfer = result.first().unwrap().as_ref().unwrap();
        assert_eq!(transfer.coin_object, CoinObject::Sui);
        assert_eq!(transfer.amount, 0);
        assert_eq!(
            transfer.sender,
            SuiAddress::from_str(
                "0x117527fdcf2f5f6e82fa499c0398def2643550d63f7e8311245d75f4635f0435"
            )?
        );

        Ok(())
    }
}
