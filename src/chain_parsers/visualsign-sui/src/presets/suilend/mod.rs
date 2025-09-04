mod config;

use config::{
    BorrowRequestIndexes, ClaimRewardsAndDepositIndexes, ClaimRewardsIndexes, Config,
    DepositCTokensIntoObligationIndexes, DepositLiquidityAndMintCTokensIndexes,
    FulfillLiquidityRequestIndexes, LendingMarketFunctions, RebalanceStakerIndexes,
    RedeemCTokensAndWithdrawLiquidityRequestIndexes, RefreshReservePriceIndexes, SUILEND_CONFIG,
    WithdrawCTokensIndexes,
};

use crate::core::{CommandVisualizer, SuiIntegrationConfig, VisualizerContext, VisualizerKind};
use crate::utils::{
    SuiCoin, SuiPackage, decode_number, get_index, get_nested_result_value, get_tx_type_arg,
    truncate_address,
};

use sui_json_rpc_types::{SuiArgument, SuiCallArg, SuiCommand, SuiProgrammableMoveCall};

use visualsign::{
    AnnotatedPayloadField, SignablePayloadField, SignablePayloadFieldCommon,
    SignablePayloadFieldListLayout, SignablePayloadFieldPreviewLayout, SignablePayloadFieldTextV2,
    errors::VisualSignError,
    field_builders::{create_address_field, create_amount_field, create_text_field},
};

pub struct SuilendVisualizer;

impl CommandVisualizer for SuilendVisualizer {
    fn visualize_tx_commands(
        &self,
        context: &VisualizerContext,
    ) -> Result<Vec<AnnotatedPayloadField>, VisualSignError> {
        let Some(SuiCommand::MoveCall(pwc)) = context.commands().get(context.command_index())
        else {
            return Err(VisualSignError::MissingData(
                "Expected a `MoveCall` for Suilend parsing".into(),
            ));
        };

        let function = pwc.function.as_str().try_into()?;

        match function {
            LendingMarketFunctions::BorrowRequest => self.handle_borrow_request(context, pwc),
            LendingMarketFunctions::ClaimRewards => self.handle_claim_rewards(context, pwc),
            LendingMarketFunctions::Repay => self.handle_repay(context, pwc),
            LendingMarketFunctions::ClaimRewardsAndDeposit => {
                self.handle_claim_rewards_and_deposit(context, pwc)
            }
            LendingMarketFunctions::CreateObligation => self.handle_create_obligation(context, pwc),
            LendingMarketFunctions::DepositCTokensIntoObligation => {
                self.handle_deposit_ctokens_into_obligation(context, pwc)
            }
            LendingMarketFunctions::DepositLiquidityAndMintCTokens => {
                self.handle_deposit_liquidity_and_mint_ctokens(context, pwc)
            }
            LendingMarketFunctions::FulfillLiquidityRequest => {
                self.handle_fulfill_liquidity_request(context, pwc)
            }
            LendingMarketFunctions::RebalanceStaker => self.handle_rebalance_staker(context, pwc),
            LendingMarketFunctions::RedeemCTokensAndWithdrawLiquidityRequest => {
                self.handle_redeem_ctokens_and_withdraw_liquidity_request(context, pwc)
            }
            LendingMarketFunctions::RefreshReservePrice => {
                self.handle_refresh_reserve_price(context, pwc)
            }
            LendingMarketFunctions::UnstakeSuiFromStaker => {
                self.handle_unstake_sui_from_staker(context, pwc)
            }
            LendingMarketFunctions::WithdrawCTokens => self.handle_withdraw_ctokens(context, pwc),
        }
    }

    fn get_config(&self) -> Option<&dyn SuiIntegrationConfig> {
        Some(SUILEND_CONFIG.get_or_init(Config::new))
    }

    fn kind(&self) -> VisualizerKind {
        VisualizerKind::Lending("Suilend")
    }
}

fn get_repay_amount(
    commands: &[SuiCommand],
    inputs: &[SuiCallArg],
    transfer_args: &[SuiArgument],
) -> Result<Option<u64>, VisualSignError> {
    let command_index_with_input_amount = get_nested_result_value(transfer_args, 4, 0);
    let command_with_input_amount = commands
        .get(command_index_with_input_amount? as usize)
        .ok_or(VisualSignError::MissingData("Command not found".into()))?;

    match command_with_input_amount {
        SuiCommand::SplitCoins(_, args_with_input_index) => {
            let amount_arg = inputs
                .get(get_index(args_with_input_index, Some(0))? as usize)
                .ok_or(VisualSignError::MissingData(
                    "Amount argument not found".into(),
                ))?;
            Ok(Some(decode_number::<u64>(amount_arg)?))
        }
        _ => Ok(None),
    }
}

impl SuilendVisualizer {
    fn handle_repay(
        &self,
        context: &VisualizerContext,
        pwc: &SuiProgrammableMoveCall,
    ) -> Result<Vec<AnnotatedPayloadField>, VisualSignError> {
        let coin: SuiCoin = get_tx_type_arg(&pwc.type_arguments, 1).unwrap_or_default();
        let package: SuiPackage = get_tx_type_arg(&pwc.type_arguments, 0).unwrap_or_default();
        let reserve_index =
            RefreshReservePriceIndexes::get_reserve_array_index(context.inputs(), &pwc.arguments)
                .ok();
        let amount = get_repay_amount(context.commands(), context.inputs(), &pwc.arguments)
            .unwrap_or_default();

        let (title_text, amount_str, amount_field) = match amount {
            Some(amount) => (
                format!("Suilend: Repay {} {}", amount, coin.symbol()),
                amount.to_string(),
                create_amount_field("Amount", &amount.to_string(), coin.symbol())?,
            ),
            None => (
                format!("Suilend: Repay {} {}", "N/A", coin.symbol()),
                "N/A".to_string(),
                create_text_field("Amount", "N/A")?,
            ),
        };

        let subtitle_text = format!("From {}", truncate_address(&context.sender().to_string()));

        let mut summary = format!("Repay {} {} via {}", amount_str, coin.symbol(), package);
        if let Some(idx) = reserve_index {
            summary.push_str(&format!(" (reserve #{})", idx));
        }

        let condensed = SignablePayloadFieldListLayout {
            fields: vec![create_text_field("Summary", &summary)?],
        };

        let mut expanded_fields = vec![
            create_address_field(
                "From",
                &context.sender().to_string(),
                None,
                None,
                None,
                None,
            )?,
            create_text_field("Package", &package.to_string())?,
            create_text_field("Coin", &coin.to_string())?,
            amount_field,
        ];
        if let Some(idx) = reserve_index {
            expanded_fields.push(create_text_field("Reserve Index", &idx.to_string())?);
        }

        let expanded = SignablePayloadFieldListLayout {
            fields: expanded_fields,
        };

        let preview_layout = SignablePayloadFieldPreviewLayout {
            title: Some(SignablePayloadFieldTextV2 {
                text: title_text.clone(),
            }),
            subtitle: Some(SignablePayloadFieldTextV2 {
                text: subtitle_text,
            }),
            condensed: Some(condensed),
            expanded: Some(expanded),
        };

        Ok(vec![AnnotatedPayloadField {
            static_annotation: None,
            dynamic_annotation: None,
            signable_payload_field: SignablePayloadField::PreviewLayout {
                common: SignablePayloadFieldCommon {
                    fallback_text: title_text,
                    label: "Suilend Repay Command".to_string(),
                },
                preview_layout,
            },
        }])
    }

    fn handle_unstake_sui_from_staker(
        &self,
        context: &VisualizerContext,
        pwc: &SuiProgrammableMoveCall,
    ) -> Result<Vec<AnnotatedPayloadField>, VisualSignError> {
        let package: SuiPackage = get_tx_type_arg(&pwc.type_arguments, 0).unwrap_or_default();
        // Uses SUI reserve index at arg 1
        let sui_reserve_index =
            RebalanceStakerIndexes::get_sui_reserve_array_index(context.inputs(), &pwc.arguments)?;

        let title_text = "Suilend: Unstake SUI from Staker".to_string();
        let subtitle_text = format!("From {}", truncate_address(&context.sender().to_string()));

        let condensed = SignablePayloadFieldListLayout {
            fields: vec![create_text_field(
                "Summary",
                &format!(
                    "Unstake SUI from staker (reserve #{}) via {}",
                    sui_reserve_index, package
                ),
            )?],
        };
        let expanded = SignablePayloadFieldListLayout {
            fields: vec![
                create_address_field(
                    "From",
                    &context.sender().to_string(),
                    None,
                    None,
                    None,
                    None,
                )?,
                create_text_field("Package", &package.to_string())?,
                create_text_field("SUI Reserve Index", &sui_reserve_index.to_string())?,
            ],
        };
        let preview_layout = SignablePayloadFieldPreviewLayout {
            title: Some(SignablePayloadFieldTextV2 {
                text: title_text.clone(),
            }),
            subtitle: Some(SignablePayloadFieldTextV2 {
                text: subtitle_text,
            }),
            condensed: Some(condensed),
            expanded: Some(expanded),
        };
        Ok(vec![AnnotatedPayloadField {
            static_annotation: None,
            dynamic_annotation: None,
            signable_payload_field: SignablePayloadField::PreviewLayout {
                common: SignablePayloadFieldCommon {
                    fallback_text: title_text,
                    label: "Suilend Unstake SUI".to_string(),
                },
                preview_layout,
            },
        }])
    }

    fn handle_withdraw_ctokens(
        &self,
        context: &VisualizerContext,
        pwc: &SuiProgrammableMoveCall,
    ) -> Result<Vec<AnnotatedPayloadField>, VisualSignError> {
        let coin: SuiCoin = get_tx_type_arg(&pwc.type_arguments, 1).unwrap_or_default();
        let package: SuiPackage = get_tx_type_arg(&pwc.type_arguments, 0).unwrap_or_default();
        let reserve_index =
            WithdrawCTokensIndexes::get_reserve_array_index(context.inputs(), &pwc.arguments)?;
        let amount = WithdrawCTokensIndexes::get_amount(context.inputs(), &pwc.arguments)?;

        let title_text = format!("Suilend: Withdraw cTokens ({})", coin.symbol());
        let subtitle_text = format!("From {}", truncate_address(&context.sender().to_string()));

        let condensed = SignablePayloadFieldListLayout {
            fields: vec![create_text_field(
                "Summary",
                &format!(
                    "Withdraw {} cTokens ({}) from reserve #{} via {}",
                    amount,
                    coin.symbol(),
                    reserve_index,
                    package
                ),
            )?],
        };
        let expanded = SignablePayloadFieldListLayout {
            fields: vec![
                create_address_field(
                    "From",
                    &context.sender().to_string(),
                    None,
                    None,
                    None,
                    None,
                )?,
                create_text_field("Package", &package.to_string())?,
                create_text_field("CToken Coin", &coin.to_string())?,
                create_amount_field("Amount", &amount.to_string(), coin.symbol())?,
                create_text_field("Reserve Index", &reserve_index.to_string())?,
            ],
        };
        let preview_layout = SignablePayloadFieldPreviewLayout {
            title: Some(SignablePayloadFieldTextV2 {
                text: title_text.clone(),
            }),
            subtitle: Some(SignablePayloadFieldTextV2 {
                text: subtitle_text,
            }),
            condensed: Some(condensed),
            expanded: Some(expanded),
        };
        Ok(vec![AnnotatedPayloadField {
            static_annotation: None,
            dynamic_annotation: None,
            signable_payload_field: SignablePayloadField::PreviewLayout {
                common: SignablePayloadFieldCommon {
                    fallback_text: title_text,
                    label: "Suilend Withdraw cTokens".to_string(),
                },
                preview_layout,
            },
        }])
    }
    fn handle_borrow_request(
        &self,
        context: &VisualizerContext,
        pwc: &SuiProgrammableMoveCall,
    ) -> Result<Vec<AnnotatedPayloadField>, VisualSignError> {
        let coin: SuiCoin = get_tx_type_arg(&pwc.type_arguments, 1).unwrap_or_default();
        let package: SuiPackage = get_tx_type_arg(&pwc.type_arguments, 0).unwrap_or_default();
        let reserve_index =
            BorrowRequestIndexes::get_reserve_array_index(context.inputs(), &pwc.arguments)?;
        let amount = BorrowRequestIndexes::get_amount(context.inputs(), &pwc.arguments)?;

        let title_text = format!("Suilend: Borrow Request {} {}", amount, coin.symbol());
        let subtitle_text = format!("From {}", truncate_address(&context.sender().to_string()));

        let condensed = SignablePayloadFieldListLayout {
            fields: vec![create_text_field(
                "Summary",
                &format!(
                    "Borrow {} {} from reserve #{} via {}",
                    amount,
                    coin.symbol(),
                    reserve_index,
                    package
                ),
            )?],
        };

        let expanded = SignablePayloadFieldListLayout {
            fields: vec![
                create_address_field(
                    "From",
                    &context.sender().to_string(),
                    None,
                    None,
                    None,
                    None,
                )?,
                create_text_field("Package", &package.to_string())?,
                create_text_field("Coin", &coin.to_string())?,
                create_text_field("Reserve Index", &reserve_index.to_string())?,
                create_amount_field("Amount", &amount.to_string(), coin.symbol())?,
            ],
        };

        let preview_layout = SignablePayloadFieldPreviewLayout {
            title: Some(SignablePayloadFieldTextV2 {
                text: title_text.clone(),
            }),
            subtitle: Some(SignablePayloadFieldTextV2 {
                text: subtitle_text,
            }),
            condensed: Some(condensed),
            expanded: Some(expanded),
        };

        Ok(vec![AnnotatedPayloadField {
            static_annotation: None,
            dynamic_annotation: None,
            signable_payload_field: SignablePayloadField::PreviewLayout {
                common: SignablePayloadFieldCommon {
                    fallback_text: title_text,
                    label: "Suilend Borrow Request".to_string(),
                },
                preview_layout,
            },
        }])
    }

    fn handle_claim_rewards(
        &self,
        context: &VisualizerContext,
        pwc: &SuiProgrammableMoveCall,
    ) -> Result<Vec<AnnotatedPayloadField>, VisualSignError> {
        let reward_coin: SuiCoin = get_tx_type_arg(&pwc.type_arguments, 1).unwrap_or_default();
        let package: SuiPackage = get_tx_type_arg(&pwc.type_arguments, 0).unwrap_or_default();
        let reserve_id = ClaimRewardsIndexes::get_reserve_id(context.inputs(), &pwc.arguments)?;
        let reward_index = ClaimRewardsIndexes::get_reward_index(context.inputs(), &pwc.arguments)?;
        let is_deposit =
            ClaimRewardsIndexes::get_is_deposit_reward(context.inputs(), &pwc.arguments)?;

        let reward_side = if is_deposit { "Deposit" } else { "Borrow" };
        let title_text = format!(
            "Suilend: Claim Rewards ({}) {}",
            reward_side,
            reward_coin.symbol()
        );
        let subtitle_text = format!("From {}", truncate_address(&context.sender().to_string()));

        let condensed = SignablePayloadFieldListLayout {
            fields: vec![create_text_field(
                "Summary",
                &format!(
                    "Claim {} rewards from reserve #{} (reward #{}) via {}",
                    reward_side, reserve_id, reward_index, package
                ),
            )?],
        };

        let expanded = SignablePayloadFieldListLayout {
            fields: vec![
                create_address_field(
                    "From",
                    &context.sender().to_string(),
                    None,
                    None,
                    None,
                    None,
                )?,
                create_text_field("Package", &package.to_string())?,
                create_text_field("Reward Coin", &reward_coin.to_string())?,
                create_text_field("Reserve Index", &reserve_id.to_string())?,
                create_text_field("Reward Index", &reward_index.to_string())?,
                create_text_field("Reward Side", reward_side)?,
            ],
        };

        let preview_layout = SignablePayloadFieldPreviewLayout {
            title: Some(SignablePayloadFieldTextV2 {
                text: title_text.clone(),
            }),
            subtitle: Some(SignablePayloadFieldTextV2 {
                text: subtitle_text,
            }),
            condensed: Some(condensed),
            expanded: Some(expanded),
        };

        Ok(vec![AnnotatedPayloadField {
            static_annotation: None,
            dynamic_annotation: None,
            signable_payload_field: SignablePayloadField::PreviewLayout {
                common: SignablePayloadFieldCommon {
                    fallback_text: title_text,
                    label: "Suilend Claim Rewards".to_string(),
                },
                preview_layout,
            },
        }])
    }

    fn handle_claim_rewards_and_deposit(
        &self,
        context: &VisualizerContext,
        pwc: &SuiProgrammableMoveCall,
    ) -> Result<Vec<AnnotatedPayloadField>, VisualSignError> {
        let coin: SuiCoin = get_tx_type_arg(&pwc.type_arguments, 1).unwrap_or_default();
        let package: SuiPackage = get_tx_type_arg(&pwc.type_arguments, 0).unwrap_or_default();
        let reward_reserve_id =
            ClaimRewardsAndDepositIndexes::get_reward_reserve_id(context.inputs(), &pwc.arguments)?;
        let reward_index =
            ClaimRewardsAndDepositIndexes::get_reward_index(context.inputs(), &pwc.arguments)?;
        let is_deposit =
            ClaimRewardsAndDepositIndexes::get_is_deposit_reward(context.inputs(), &pwc.arguments)?;
        let deposit_reserve_id = ClaimRewardsAndDepositIndexes::get_deposit_reserve_id(
            context.inputs(),
            &pwc.arguments,
        )?;

        let reward_side = if is_deposit { "Deposit" } else { "Borrow" };
        let title_text = format!("Suilend: Claim Rewards and Deposit ({})", coin.symbol());
        let subtitle_text = format!("From {}", truncate_address(&context.sender().to_string()));

        let condensed = SignablePayloadFieldListLayout {
            fields: vec![create_text_field(
                "Summary",
                &format!(
                    "Claim {} rewards from reserve #{} (reward #{}) and deposit to reserve #{} via {}",
                    reward_side, reward_reserve_id, reward_index, deposit_reserve_id, package
                ),
            )?],
        };

        let expanded = SignablePayloadFieldListLayout {
            fields: vec![
                create_address_field(
                    "From",
                    &context.sender().to_string(),
                    None,
                    None,
                    None,
                    None,
                )?,
                create_text_field("Package", &package.to_string())?,
                create_text_field("Coin", &coin.to_string())?,
                create_text_field("Reward Side", reward_side)?,
                create_text_field("Reward Reserve Index", &reward_reserve_id.to_string())?,
                create_text_field("Reward Index", &reward_index.to_string())?,
                create_text_field("Deposit Reserve Index", &deposit_reserve_id.to_string())?,
            ],
        };

        let preview_layout = SignablePayloadFieldPreviewLayout {
            title: Some(SignablePayloadFieldTextV2 {
                text: title_text.clone(),
            }),
            subtitle: Some(SignablePayloadFieldTextV2 {
                text: subtitle_text,
            }),
            condensed: Some(condensed),
            expanded: Some(expanded),
        };

        Ok(vec![AnnotatedPayloadField {
            static_annotation: None,
            dynamic_annotation: None,
            signable_payload_field: SignablePayloadField::PreviewLayout {
                common: SignablePayloadFieldCommon {
                    fallback_text: title_text,
                    label: "Suilend Claim Rewards and Deposit Command".to_string(),
                },
                preview_layout,
            },
        }])
    }

    fn handle_create_obligation(
        &self,
        context: &VisualizerContext,
        pwc: &SuiProgrammableMoveCall,
    ) -> Result<Vec<AnnotatedPayloadField>, VisualSignError> {
        let package: SuiPackage = get_tx_type_arg(&pwc.type_arguments, 0).unwrap_or_default();
        let title_text = "Suilend: Create Obligation".to_string();
        let subtitle_text = format!("From {}", truncate_address(&context.sender().to_string()));

        let condensed = SignablePayloadFieldListLayout {
            fields: vec![create_text_field(
                "Summary",
                &format!("Create new obligation via {}", package),
            )?],
        };
        let expanded = SignablePayloadFieldListLayout {
            fields: vec![
                create_address_field(
                    "From",
                    &context.sender().to_string(),
                    None,
                    None,
                    None,
                    None,
                )?,
                create_text_field("Package", &package.to_string())?,
            ],
        };
        let preview_layout = SignablePayloadFieldPreviewLayout {
            title: Some(SignablePayloadFieldTextV2 {
                text: title_text.clone(),
            }),
            subtitle: Some(SignablePayloadFieldTextV2 {
                text: subtitle_text,
            }),
            condensed: Some(condensed),
            expanded: Some(expanded),
        };
        Ok(vec![AnnotatedPayloadField {
            static_annotation: None,
            dynamic_annotation: None,
            signable_payload_field: SignablePayloadField::PreviewLayout {
                common: SignablePayloadFieldCommon {
                    fallback_text: title_text,
                    label: "Suilend Create Obligation".to_string(),
                },
                preview_layout,
            },
        }])
    }

    fn handle_deposit_ctokens_into_obligation(
        &self,
        context: &VisualizerContext,
        pwc: &SuiProgrammableMoveCall,
    ) -> Result<Vec<AnnotatedPayloadField>, VisualSignError> {
        let coin: SuiCoin = get_tx_type_arg(&pwc.type_arguments, 1).unwrap_or_default();
        let package: SuiPackage = get_tx_type_arg(&pwc.type_arguments, 0).unwrap_or_default();
        let reserve_index = DepositCTokensIntoObligationIndexes::get_reserve_array_index(
            context.inputs(),
            &pwc.arguments,
        )?;

        let title_text = format!("Suilend: Deposit cTokens ({})", coin.symbol());
        let subtitle_text = format!("From {}", truncate_address(&context.sender().to_string()));

        let condensed = SignablePayloadFieldListLayout {
            fields: vec![create_text_field(
                "Summary",
                &format!(
                    "Deposit cTokens of {} into obligation from reserve #{} via {}",
                    coin.symbol(),
                    reserve_index,
                    package
                ),
            )?],
        };
        let expanded = SignablePayloadFieldListLayout {
            fields: vec![
                create_address_field(
                    "From",
                    &context.sender().to_string(),
                    None,
                    None,
                    None,
                    None,
                )?,
                create_text_field("Package", &package.to_string())?,
                create_text_field("CToken (underlying)", &coin.to_string())?,
                create_text_field("Reserve Index", &reserve_index.to_string())?,
            ],
        };
        let preview_layout = SignablePayloadFieldPreviewLayout {
            title: Some(SignablePayloadFieldTextV2 {
                text: title_text.clone(),
            }),
            subtitle: Some(SignablePayloadFieldTextV2 {
                text: subtitle_text,
            }),
            condensed: Some(condensed),
            expanded: Some(expanded),
        };
        Ok(vec![AnnotatedPayloadField {
            static_annotation: None,
            dynamic_annotation: None,
            signable_payload_field: SignablePayloadField::PreviewLayout {
                common: SignablePayloadFieldCommon {
                    fallback_text: title_text,
                    label: "Suilend Deposit cTokens".to_string(),
                },
                preview_layout,
            },
        }])
    }

    fn handle_deposit_liquidity_and_mint_ctokens(
        &self,
        context: &VisualizerContext,
        pwc: &SuiProgrammableMoveCall,
    ) -> Result<Vec<AnnotatedPayloadField>, VisualSignError> {
        let coin: SuiCoin = get_tx_type_arg(&pwc.type_arguments, 1).unwrap_or_default();
        let package: SuiPackage = get_tx_type_arg(&pwc.type_arguments, 0).unwrap_or_default();
        let reserve_index = DepositLiquidityAndMintCTokensIndexes::get_reserve_array_index(
            context.inputs(),
            &pwc.arguments,
        )?;

        let title_text = format!(
            "Suilend: Deposit Liquidity and Mint cTokens ({})",
            coin.symbol()
        );
        let subtitle_text = format!("From {}", truncate_address(&context.sender().to_string()));
        let condensed = SignablePayloadFieldListLayout {
            fields: vec![create_text_field(
                "Summary",
                &format!(
                    "Deposit liquidity of {} into reserve #{} via {}",
                    coin.symbol(),
                    reserve_index,
                    package
                ),
            )?],
        };
        let expanded = SignablePayloadFieldListLayout {
            fields: vec![
                create_address_field(
                    "From",
                    &context.sender().to_string(),
                    None,
                    None,
                    None,
                    None,
                )?,
                create_text_field("Package", &package.to_string())?,
                create_text_field("Coin", &coin.to_string())?,
                create_text_field("Reserve Index", &reserve_index.to_string())?,
            ],
        };
        let preview_layout = SignablePayloadFieldPreviewLayout {
            title: Some(SignablePayloadFieldTextV2 {
                text: title_text.clone(),
            }),
            subtitle: Some(SignablePayloadFieldTextV2 {
                text: subtitle_text,
            }),
            condensed: Some(condensed),
            expanded: Some(expanded),
        };
        Ok(vec![AnnotatedPayloadField {
            static_annotation: None,
            dynamic_annotation: None,
            signable_payload_field: SignablePayloadField::PreviewLayout {
                common: SignablePayloadFieldCommon {
                    fallback_text: title_text,
                    label: "Suilend Deposit Liquidity".to_string(),
                },
                preview_layout,
            },
        }])
    }

    fn handle_fulfill_liquidity_request(
        &self,
        context: &VisualizerContext,
        pwc: &SuiProgrammableMoveCall,
    ) -> Result<Vec<AnnotatedPayloadField>, VisualSignError> {
        let coin: SuiCoin = get_tx_type_arg(&pwc.type_arguments, 1).unwrap_or_default();
        let package: SuiPackage = get_tx_type_arg(&pwc.type_arguments, 0).unwrap_or_default();
        let reserve_index = FulfillLiquidityRequestIndexes::get_reserve_array_index(
            context.inputs(),
            &pwc.arguments,
        )?;
        let title_text = format!("Suilend: Fulfill Liquidity Request ({})", coin.symbol());
        let subtitle_text = format!("From {}", truncate_address(&context.sender().to_string()));
        let condensed = SignablePayloadFieldListLayout {
            fields: vec![create_text_field(
                "Summary",
                &format!(
                    "Fulfill liquidity request for reserve #{} via {}",
                    reserve_index, package
                ),
            )?],
        };
        let expanded = SignablePayloadFieldListLayout {
            fields: vec![
                create_address_field(
                    "From",
                    &context.sender().to_string(),
                    None,
                    None,
                    None,
                    None,
                )?,
                create_text_field("Package", &package.to_string())?,
                create_text_field("Coin", &coin.to_string())?,
                create_text_field("Reserve Index", &reserve_index.to_string())?,
            ],
        };
        let preview_layout = SignablePayloadFieldPreviewLayout {
            title: Some(SignablePayloadFieldTextV2 {
                text: title_text.clone(),
            }),
            subtitle: Some(SignablePayloadFieldTextV2 {
                text: subtitle_text,
            }),
            condensed: Some(condensed),
            expanded: Some(expanded),
        };
        Ok(vec![AnnotatedPayloadField {
            static_annotation: None,
            dynamic_annotation: None,
            signable_payload_field: SignablePayloadField::PreviewLayout {
                common: SignablePayloadFieldCommon {
                    fallback_text: title_text,
                    label: "Suilend Fulfill Liquidity".to_string(),
                },
                preview_layout,
            },
        }])
    }

    fn handle_rebalance_staker(
        &self,
        context: &VisualizerContext,
        pwc: &SuiProgrammableMoveCall,
    ) -> Result<Vec<AnnotatedPayloadField>, VisualSignError> {
        let package: SuiPackage = get_tx_type_arg(&pwc.type_arguments, 0).unwrap_or_default();
        let sui_reserve_index =
            RebalanceStakerIndexes::get_sui_reserve_array_index(context.inputs(), &pwc.arguments)?;
        let title_text = "Suilend: Rebalance Staker".to_string();
        let subtitle_text = format!("From {}", truncate_address(&context.sender().to_string()));
        let condensed = SignablePayloadFieldListLayout {
            fields: vec![create_text_field(
                "Summary",
                &format!(
                    "Rebalance SUI staker for reserve #{} via {}",
                    sui_reserve_index, package
                ),
            )?],
        };
        let expanded = SignablePayloadFieldListLayout {
            fields: vec![
                create_address_field(
                    "From",
                    &context.sender().to_string(),
                    None,
                    None,
                    None,
                    None,
                )?,
                create_text_field("Package", &package.to_string())?,
                create_text_field("SUI Reserve Index", &sui_reserve_index.to_string())?,
            ],
        };
        let preview_layout = SignablePayloadFieldPreviewLayout {
            title: Some(SignablePayloadFieldTextV2 {
                text: title_text.clone(),
            }),
            subtitle: Some(SignablePayloadFieldTextV2 {
                text: subtitle_text,
            }),
            condensed: Some(condensed),
            expanded: Some(expanded),
        };
        Ok(vec![AnnotatedPayloadField {
            static_annotation: None,
            dynamic_annotation: None,
            signable_payload_field: SignablePayloadField::PreviewLayout {
                common: SignablePayloadFieldCommon {
                    fallback_text: title_text,
                    label: "Suilend Rebalance Staker".to_string(),
                },
                preview_layout,
            },
        }])
    }

    fn handle_redeem_ctokens_and_withdraw_liquidity_request(
        &self,
        context: &VisualizerContext,
        pwc: &SuiProgrammableMoveCall,
    ) -> Result<Vec<AnnotatedPayloadField>, VisualSignError> {
        let coin: SuiCoin = get_tx_type_arg(&pwc.type_arguments, 1).unwrap_or_default();
        let package: SuiPackage = get_tx_type_arg(&pwc.type_arguments, 0).unwrap_or_default();
        let reserve_index =
            RedeemCTokensAndWithdrawLiquidityRequestIndexes::get_reserve_array_index(
                context.inputs(),
                &pwc.arguments,
            )?;
        let title_text = format!(
            "Suilend: Redeem cTokens and Withdraw Liquidity ({})",
            coin.symbol()
        );
        let subtitle_text = format!("From {}", truncate_address(&context.sender().to_string()));
        let condensed = SignablePayloadFieldListLayout {
            fields: vec![create_text_field(
                "Summary",
                &format!(
                    "Redeem cTokens of {} and withdraw liquidity from reserve #{} via {}",
                    coin.symbol(),
                    reserve_index,
                    package
                ),
            )?],
        };
        let expanded = SignablePayloadFieldListLayout {
            fields: vec![
                create_address_field(
                    "From",
                    &context.sender().to_string(),
                    None,
                    None,
                    None,
                    None,
                )?,
                create_text_field("Package", &package.to_string())?,
                create_text_field("Coin", &coin.to_string())?,
                create_text_field("Reserve Index", &reserve_index.to_string())?,
            ],
        };
        let preview_layout = SignablePayloadFieldPreviewLayout {
            title: Some(SignablePayloadFieldTextV2 {
                text: title_text.clone(),
            }),
            subtitle: Some(SignablePayloadFieldTextV2 {
                text: subtitle_text,
            }),
            condensed: Some(condensed),
            expanded: Some(expanded),
        };
        Ok(vec![AnnotatedPayloadField {
            static_annotation: None,
            dynamic_annotation: None,
            signable_payload_field: SignablePayloadField::PreviewLayout {
                common: SignablePayloadFieldCommon {
                    fallback_text: title_text,
                    label: "Suilend Redeem cTokens".to_string(),
                },
                preview_layout,
            },
        }])
    }

    fn handle_refresh_reserve_price(
        &self,
        context: &VisualizerContext,
        pwc: &SuiProgrammableMoveCall,
    ) -> Result<Vec<AnnotatedPayloadField>, VisualSignError> {
        let package: SuiPackage = get_tx_type_arg(&pwc.type_arguments, 0).unwrap_or_default();
        let reserve_index =
            RefreshReservePriceIndexes::get_reserve_array_index(context.inputs(), &pwc.arguments)?;
        let title_text = "Suilend: Refresh Reserve Price".to_string();
        let subtitle_text = format!("From {}", truncate_address(&context.sender().to_string()));
        let condensed = SignablePayloadFieldListLayout {
            fields: vec![create_text_field(
                "Summary",
                &format!(
                    "Refresh price for reserve #{} via {}",
                    reserve_index, package
                ),
            )?],
        };
        let expanded = SignablePayloadFieldListLayout {
            fields: vec![
                create_address_field(
                    "From",
                    &context.sender().to_string(),
                    None,
                    None,
                    None,
                    None,
                )?,
                create_text_field("Package", &package.to_string())?,
                create_text_field("Reserve Index", &reserve_index.to_string())?,
            ],
        };
        let preview_layout = SignablePayloadFieldPreviewLayout {
            title: Some(SignablePayloadFieldTextV2 {
                text: title_text.clone(),
            }),
            subtitle: Some(SignablePayloadFieldTextV2 {
                text: subtitle_text,
            }),
            condensed: Some(condensed),
            expanded: Some(expanded),
        };
        Ok(vec![AnnotatedPayloadField {
            static_annotation: None,
            dynamic_annotation: None,
            signable_payload_field: SignablePayloadField::PreviewLayout {
                common: SignablePayloadFieldCommon {
                    fallback_text: title_text,
                    label: "Suilend Refresh Reserve Price".to_string(),
                },
                preview_layout,
            },
        }])
    }
}

#[cfg(test)]
mod tests {
    use crate::utils::payload_from_b64;

    use visualsign::test_utils::assert_has_field;

    const SUILEND_REPAY_LABEL: &str = "Suilend Repay Command";

    #[test]
    #[ignore]
    fn test_suilend_repay_commands() {
        // https://suivision.xyz/txblock/FTckS194eV3LBGCfcqiW8LxD7E3Nif5MNWqZa21jE5fn
        let test_data = "AQAAAAAAVAEAEJ0lGrZLg0k4fd7CnC3PHeUk4Yh3dKeuucRY+eHLLsIhYvojAAAAACA68M75doP0H4ycZhHHVWnuoawjwXSf1m3S6CclNjwMhgEA3cMpkB1SkWDo8iRkghAWMsqQvjNLjzn3ae9TN2gHmk3F8PkjAAAAACAZ/2eCHht1tG6JwPG+NwqQuIiyiJS7Hc9njPh5hiVqQAEA/ZphTw0iXDXAE8i3rO7s6DMeN4zPiqYGFW2szQcZzbrF8PkjAAAAACBahAh129Xm3K8VZa0DLp/IhtjhLwtGecYgbnWv6UHVLAAIqihr7gAAAAABAYQDDSbYXqpwNQhKBX8vEfcBt+Lk7ah1Ub7Lx8l1Bezhc4GNBAAAAAABAAgIAAAAAAAAAAAgsZy6F1dy5MTegTGRTIFnSUs3AWE285Y7YYmVzrhnL+wBAQAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAGAQAAAAAAAAAAACANaK359B8XWjdEYyfOP63+MktSMVzzaOL7OPlGLjfjZwAgHN1js/wdOgZ6UKiiRJFcISVKTKQUEEIDcD8FAXPSvGwACAoAAAAAAAAAAAgRAAAAAAAAAAABAQAICgAAAAAAAAAAIBzdY7P8HToGelCookSRXCElSkykFBBCA3A/BQFz0rxsAAgKAAAAAAAAAAAIEgAAAAAAAAAAAQEACAoAAAAAAAAAACAc3WOz/B06BnpQqKJEkVwhJUpMpBQQQgNwPwUBc9K8bAAICgAAAAAAAAAACBMAAAAAAAAAAAEBAAgKAAAAAAAAAAAgHN1js/wdOgZ6UKiiRJFcISVKTKQUEEIDcD8FAXPSvGwACAoAAAAAAAAAAAgUAAAAAAAAAAABAQAICgAAAAAAAAAAIBzdY7P8HToGelCookSRXCElSkykFBBCA3A/BQFz0rxsAAgKAAAAAAAAAAAIFQAAAAAAAAAAAQEACAoAAAAAAAAAACAc3WOz/B06BnpQqKJEkVwhJUpMpBQQQgNwPwUBc9K8bAAICgAAAAAAAAAACBYAAAAAAAAAAAEBAAgKAAAAAAAAAAAgHN1js/wdOgZ6UKiiRJFcISVKTKQUEEIDcD8FAXPSvGwACAoAAAAAAAAAAAgXAAAAAAAAAAABAQAICgAAAAAAAAAAIBzdY7P8HToGelCookSRXCElSkykFBBCA3A/BQFz0rxsAAgKAAAAAAAAAAAIGAAAAAAAAAAAAQEACAoAAAAAAAAAACBN4TSxBHPB7o0nezGMXRuf6mfLuvM2o0Q9f2ZmyCQbSQAIAAAAAAAAAAAACCEAAAAAAAAAAAEAAAgKAAAAAAAAAAAgTeE0sQRzwe6NJ3sxjF0bn+pny7rzNqNEPX9mZsgkG0kACAoAAAAAAAAAAAgYAAAAAAAAAAABAQAICgAAAAAAAAAAIFjJcfPjR67llrdId/50CM32AukIWrxwy1n9u+lnBdvRAAgAAAAAAAAAAAAIEgAAAAAAAAAAAQEACAoAAAAAAAAAACBYyXHz40eu5Za3SHf+dAjN9gLpCFq8cMtZ/bvpZwXb0QAIAAAAAAAAAAAACBMAAAAAAAAAAAEBAAgKAAAAAAAAAAAgWMlx8+NHruWWt0h3/nQIzfYC6QhavHDLWf276WcF29EACAAAAAAAAAAAAAgUAAAAAAAAAAABAQAICgAAAAAAAAAAIFjJcfPjR67llrdId/50CM32AukIWrxwy1n9u+lnBdvRAAgAAAAAAAAAAAAIFQAAAAAAAAAAAQEACAoAAAAAAAAAACBYyXHz40eu5Za3SHf+dAjN9gLpCFq8cMtZ/bvpZwXb0QAIAAAAAAAAAAAACBYAAAAAAAAAAAEBAAgKAAAAAAAAABMDAQAAAgEBAAECAAIBAAABAQMAAEPSW+alXbTnzAjdkUuDJufVb7ZMZ/D7lho0nihy9MwIDmxlbmRpbmdfbWFya2V0BXJlcGF5Agf5WwYUHtShdPI5QXMjvePyCbly9ZMNhSHqOKUq/zpt3wdzdWlsZW5kCU1BSU5fUE9PTAAH3ut6RmLuyfLz3vA/uTemY93aouIVuAeKKE0Ca3lGwnAEZGVlcARERUVQAAUBBAABBQABBgABBwADAQAAAAEBAwEAAAABCAAAQ9Jb5qVdtOfMCN2RS4Mm59Vvtkxn8PuWGjSeKHL0zAgObGVuZGluZ19tYXJrZXQZY2xhaW1fcmV3YXJkc19hbmRfZGVwb3NpdAIH+VsGFB7UoXTyOUFzI73j8gm5cvWTDYUh6jilKv86bd8Hc3VpbGVuZAlNQUlOX1BPT0wAB4NVaJH0oPIzznsFz+f5V9QCBJKjT1QFssuTd9BgvvS/CnNwcmluZ19zdWkKU1BSSU5HX1NVSQAHAQQAAQkAAQcAAQoAAQsAAQwAAQ0AAEPSW+alXbTnzAjdkUuDJufVb7ZMZ/D7lho0nihy9MwIDmxlbmRpbmdfbWFya2V0GWNsYWltX3Jld2FyZHNfYW5kX2RlcG9zaXQCB/lbBhQe1KF08jlBcyO94/IJuXL1kw2FIeo4pSr/Om3fB3N1aWxlbmQJTUFJTl9QT09MAAeDVWiR9KDyM857Bc/n+VfUAgSSo09UBbLLk3fQYL70vwpzcHJpbmdfc3VpClNQUklOR19TVUkABwEEAAEOAAEHAAEPAAEQAAERAAESAABD0lvmpV2058wI3ZFLgybn1W+2TGfw+5YaNJ4ocvTMCA5sZW5kaW5nX21hcmtldBljbGFpbV9yZXdhcmRzX2FuZF9kZXBvc2l0Agf5WwYUHtShdPI5QXMjvePyCbly9ZMNhSHqOKUq/zpt3wdzdWlsZW5kCU1BSU5fUE9PTAAHg1VokfSg8jPOewXP5/lX1AIEkqNPVAWyy5N30GC+9L8Kc3ByaW5nX3N1aQpTUFJJTkdfU1VJAAcBBAABEwABBwABFAABFQABFgABFwAAQ9Jb5qVdtOfMCN2RS4Mm59Vvtkxn8PuWGjSeKHL0zAgObGVuZGluZ19tYXJrZXQZY2xhaW1fcmV3YXJkc19hbmRfZGVwb3NpdAIH+VsGFB7UoXTyOUFzI73j8gm5cvWTDYUh6jilKv86bd8Hc3VpbGVuZAlNQUlOX1BPT0wAB4NVaJH0oPIzznsFz+f5V9QCBJKjT1QFssuTd9BgvvS/CnNwcmluZ19zdWkKU1BSSU5HX1NVSQAHAQQAARgAAQcAARkAARoAARsAARwAAEPSW+alXbTnzAjdkUuDJufVb7ZMZ/D7lho0nihy9MwIDmxlbmRpbmdfbWFya2V0GWNsYWltX3Jld2FyZHNfYW5kX2RlcG9zaXQCB/lbBhQe1KF08jlBcyO94/IJuXL1kw2FIeo4pSr/Om3fB3N1aWxlbmQJTUFJTl9QT09MAAeDVWiR9KDyM857Bc/n+VfUAgSSo09UBbLLk3fQYL70vwpzcHJpbmdfc3VpClNQUklOR19TVUkABwEEAAEdAAEHAAEeAAEfAAEgAAEhAABD0lvmpV2058wI3ZFLgybn1W+2TGfw+5YaNJ4ocvTMCA5sZW5kaW5nX21hcmtldBljbGFpbV9yZXdhcmRzX2FuZF9kZXBvc2l0Agf5WwYUHtShdPI5QXMjvePyCbly9ZMNhSHqOKUq/zpt3wdzdWlsZW5kCU1BSU5fUE9PTAAHg1VokfSg8jPOewXP5/lX1AIEkqNPVAWyy5N30GC+9L8Kc3ByaW5nX3N1aQpTUFJJTkdfU1VJAAcBBAABIgABBwABIwABJAABJQABJgAAQ9Jb5qVdtOfMCN2RS4Mm59Vvtkxn8PuWGjSeKHL0zAgObGVuZGluZ19tYXJrZXQZY2xhaW1fcmV3YXJkc19hbmRfZGVwb3NpdAIH+VsGFB7UoXTyOUFzI73j8gm5cvWTDYUh6jilKv86bd8Hc3VpbGVuZAlNQUlOX1BPT0wAB4NVaJH0oPIzznsFz+f5V9QCBJKjT1QFssuTd9BgvvS/CnNwcmluZ19zdWkKU1BSSU5HX1NVSQAHAQQAAScAAQcAASgAASkAASoAASsAAEPSW+alXbTnzAjdkUuDJufVb7ZMZ/D7lho0nihy9MwIDmxlbmRpbmdfbWFya2V0GWNsYWltX3Jld2FyZHNfYW5kX2RlcG9zaXQCB/lbBhQe1KF08jlBcyO94/IJuXL1kw2FIeo4pSr/Om3fB3N1aWxlbmQJTUFJTl9QT09MAAeDVWiR9KDyM857Bc/n+VfUAgSSo09UBbLLk3fQYL70vwpzcHJpbmdfc3VpClNQUklOR19TVUkABwEEAAEsAAEHAAEtAAEuAAEvAAEwAABD0lvmpV2058wI3ZFLgybn1W+2TGfw+5YaNJ4ocvTMCA5sZW5kaW5nX21hcmtldBljbGFpbV9yZXdhcmRzX2FuZF9kZXBvc2l0Agf5WwYUHtShdPI5QXMjvePyCbly9ZMNhSHqOKUq/zpt3wdzdWlsZW5kCU1BSU5fUE9PTAAHg1VokfSg8jPOewXP5/lX1AIEkqNPVAWyy5N30GC+9L8Kc3ByaW5nX3N1aQpTUFJJTkdfU1VJAAcBBAABMQABBwABMgABMwABNAABNQAAQ9Jb5qVdtOfMCN2RS4Mm59Vvtkxn8PuWGjSeKHL0zAgObGVuZGluZ19tYXJrZXQZY2xhaW1fcmV3YXJkc19hbmRfZGVwb3NpdAIH+VsGFB7UoXTyOUFzI73j8gm5cvWTDYUh6jilKv86bd8Hc3VpbGVuZAlNQUlOX1BPT0wAB4NVaJH0oPIzznsFz+f5V9QCBJKjT1QFssuTd9BgvvS/CnNwcmluZ19zdWkKU1BSSU5HX1NVSQAHAQQAATYAAQcAATcAATgAATkAAToAAEPSW+alXbTnzAjdkUuDJufVb7ZMZ/D7lho0nihy9MwIDmxlbmRpbmdfbWFya2V0GWNsYWltX3Jld2FyZHNfYW5kX2RlcG9zaXQCB/lbBhQe1KF08jlBcyO94/IJuXL1kw2FIeo4pSr/Om3fB3N1aWxlbmQJTUFJTl9QT09MAAeDVWiR9KDyM857Bc/n+VfUAgSSo09UBbLLk3fQYL70vwpzcHJpbmdfc3VpClNQUklOR19TVUkABwEEAAE7AAEHAAE8AAE9AAE+AAE/AABD0lvmpV2058wI3ZFLgybn1W+2TGfw+5YaNJ4ocvTMCA5sZW5kaW5nX21hcmtldBljbGFpbV9yZXdhcmRzX2FuZF9kZXBvc2l0Agf5WwYUHtShdPI5QXMjvePyCbly9ZMNhSHqOKUq/zpt3wdzdWlsZW5kCU1BSU5fUE9PTAAHg1VokfSg8jPOewXP5/lX1AIEkqNPVAWyy5N30GC+9L8Kc3ByaW5nX3N1aQpTUFJJTkdfU1VJAAcBBAABQAABBwABQQABQgABQwABRAAAQ9Jb5qVdtOfMCN2RS4Mm59Vvtkxn8PuWGjSeKHL0zAgObGVuZGluZ19tYXJrZXQZY2xhaW1fcmV3YXJkc19hbmRfZGVwb3NpdAIH+VsGFB7UoXTyOUFzI73j8gm5cvWTDYUh6jilKv86bd8Hc3VpbGVuZAlNQUlOX1BPT0wAB4NVaJH0oPIzznsFz+f5V9QCBJKjT1QFssuTd9BgvvS/CnNwcmluZ19zdWkKU1BSSU5HX1NVSQAHAQQAAUUAAQcAAUYAAUcAAUgAAUkAAEPSW+alXbTnzAjdkUuDJufVb7ZMZ/D7lho0nihy9MwIDmxlbmRpbmdfbWFya2V0GWNsYWltX3Jld2FyZHNfYW5kX2RlcG9zaXQCB/lbBhQe1KF08jlBcyO94/IJuXL1kw2FIeo4pSr/Om3fB3N1aWxlbmQJTUFJTl9QT09MAAeDVWiR9KDyM857Bc/n+VfUAgSSo09UBbLLk3fQYL70vwpzcHJpbmdfc3VpClNQUklOR19TVUkABwEEAAFKAAEHAAFLAAFMAAFNAAFOAABD0lvmpV2058wI3ZFLgybn1W+2TGfw+5YaNJ4ocvTMCA5sZW5kaW5nX21hcmtldBljbGFpbV9yZXdhcmRzX2FuZF9kZXBvc2l0Agf5WwYUHtShdPI5QXMjvePyCbly9ZMNhSHqOKUq/zpt3wdzdWlsZW5kCU1BSU5fUE9PTAAHg1VokfSg8jPOewXP5/lX1AIEkqNPVAWyy5N30GC+9L8Kc3ByaW5nX3N1aQpTUFJJTkdfU1VJAAcBBAABTwABBwABUAABUQABUgABUwANaK359B8XWjdEYyfOP63+MktSMVzzaOL7OPlGLjfjZwG+q4Xt5/4FqoEe9uq7tTIOrUkKac446qtO8DibDhQXmavz+SMAAAAAINwOJolnI8NVzHRjl9lNo8PRv6MfrxQs255wQ77TlXJgDWit+fQfF1o3RGMnzj+t/jJLUjFc82ji+zj5Ri4342f5AQAAAAAAAGDDfgAAAAAAAAFhAK7FhAiarg/k6SSfPJRpT1Z+IyE3hhDosgmNpor/Yw+jwWpPMJQErH9EWK35U4wTvYKisuyh8OJ3uvUsnYav3QauLSm1lIJYulFzOKYYn5ZEZHmnXDqIWAdTMPm8ZbSuKw==";

        let payload = payload_from_b64(test_data);
        assert_has_field(&payload, SUILEND_REPAY_LABEL);
    }

    #[test]
    #[ignore]
    fn test_visualizer_kind_for_suilend_repay() {
        // https://suivision.xyz/txblock/FTckS194eV3LBGCfcqiW8LxD7E3Nif5MNWqZa21jE5fn
        let _test_data = "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";

        // let block_data = crate::core::commands::tests::block_data_from_b64(test_data);
        // let (tx_commands, tx_inputs) = match block_data.transaction() {
        //     SuiTransactionBlockKind::ProgrammableTransaction(tx) => (&tx.commands, &tx.inputs),
        //     _ => panic!("expected programmable transaction"),
        // };
        //
        // let visualizer = crate::presets::suilend::SuilendVisualizer;
        // let results: Vec<_> = tx_commands
        //     .iter()
        //     .enumerate()
        //     .filter_map(|(command_index, _)| {
        //         visualize_with_any(
        //             &[&visualizer],
        //             &VisualizerContext::new(
        //                 block_data.sender(),
        //                 command_index,
        //                 tx_commands,
        //                 tx_inputs,
        //             ),
        //         )
        //     })
        //     .map(|res| res.unwrap())
        //     .collect();
        //
        // assert!(
        //     results
        //         .iter()
        //         .any(|r| matches!(r.kind, VisualizerKind::Lending(name) if name == "Suilend")),
        //     "should contain a suilend lending visualization"
        // );
    }
}
