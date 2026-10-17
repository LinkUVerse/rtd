// Copyright (c) LinkU Labs, Inc.
// SPDX-License-Identifier: Apache-2.0

//! Read-only inspection of explicit local genesis inputs. Never generates keys or a chain.
use anyhow::{Result, bail, ensure};
use fastcrypto::encoding::{Encoding, Hex};
use fastcrypto::hash::{HashFunction, Sha256};
use rtd_protocol_config::ProtocolVersion;
use serde::de::{MapAccess, SeqAccess, Visitor};
use serde::{Deserialize, Deserializer, Serialize};
use serde_yaml::{Mapping, Value};

use crate::genesis_config::GenesisConfig;

const PARAMETERS: &[&str] = &[
    "chain_start_timestamp_ms",
    "protocol_version",
    "allow_insertion_of_extra_objects",
    "epoch_duration_ms",
    "stake_subsidy_start_epoch",
    "stake_subsidy_initial_distribution_amount",
    "stake_subsidy_period_length",
    "stake_subsidy_decrease_rate",
];

// serde_yaml 0.8 Value otherwise collapses duplicate mapping keys before typed parsing.
pub(crate) struct Unique(pub(crate) Value);
impl<'de> Deserialize<'de> for Unique {
    fn deserialize<D: Deserializer<'de>>(d: D) -> std::result::Result<Self, D::Error> {
        struct V;
        impl<'de> Visitor<'de> for V {
            type Value = Unique;
            fn expecting(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
                f.write_str("a scalar, sequence, or unique string-keyed mapping")
            }
            fn visit_bool<E>(self, v: bool) -> std::result::Result<Unique, E> {
                Ok(Unique(Value::Bool(v)))
            }
            fn visit_i64<E>(self, v: i64) -> std::result::Result<Unique, E> {
                Ok(Unique(Value::Number(v.into())))
            }
            fn visit_u64<E>(self, v: u64) -> std::result::Result<Unique, E> {
                Ok(Unique(Value::Number(v.into())))
            }
            fn visit_f64<E: serde::de::Error>(self, _: f64) -> std::result::Result<Unique, E> {
                Err(E::custom("floating point genesis input is not supported"))
            }
            fn visit_str<E>(self, v: &str) -> std::result::Result<Unique, E> {
                Ok(Unique(Value::String(v.into())))
            }
            fn visit_string<E>(self, v: String) -> std::result::Result<Unique, E> {
                Ok(Unique(Value::String(v)))
            }
            fn visit_unit<E>(self) -> std::result::Result<Unique, E> {
                Ok(Unique(Value::Null))
            }
            fn visit_none<E>(self) -> std::result::Result<Unique, E> {
                Ok(Unique(Value::Null))
            }
            fn visit_seq<A: SeqAccess<'de>>(
                self,
                mut a: A,
            ) -> std::result::Result<Unique, A::Error> {
                let mut items = Vec::new();
                while let Some(Unique(v)) = a.next_element()? {
                    items.push(v);
                }
                Ok(Unique(Value::Sequence(items)))
            }
            fn visit_map<A: MapAccess<'de>>(
                self,
                mut a: A,
            ) -> std::result::Result<Unique, A::Error> {
                let mut m = Mapping::new();
                while let Some(Unique(k)) = a.next_key()? {
                    if !matches!(k, Value::String(_)) || m.contains_key(&k) {
                        return Err(serde::de::Error::custom(
                            "non-string or duplicate genesis key",
                        ));
                    }
                    let Unique(v) = a.next_value()?;
                    m.insert(k, v);
                }
                Ok(Unique(Value::Mapping(m)))
            }
        }
        d.deserialize_any(V)
    }
}

fn mapping<'a>(v: &'a Value, allowed: &[&str], required: &[&str]) -> Result<&'a Mapping> {
    let m = v
        .as_mapping()
        .ok_or_else(|| anyhow::anyhow!("genesis mapping required"))?;
    ensure!(
        m.iter()
            .all(|(k, _)| k.as_str().is_some_and(|s| allowed.contains(&s))),
        "unknown genesis field"
    );
    ensure!(
        required
            .iter()
            .all(|k| m.contains_key(&Value::String((*k).into()))),
        "explicit genesis field missing"
    );
    Ok(m)
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Inspection {
    pub version: u8,
    pub source_sha256: String,
    pub protocol_min: u64,
    pub protocol_max: u64,
    pub parameters: rtd_config::genesis::GenesisCeremonyParameters,
    pub accounts: Vec<crate::genesis_config::AccountConfig>,
    pub validator_token_recipient: Option<rtd_types::base_types::RtdAddress>,
    pub validator_allocation_policy: &'static str,
    pub generated_genesis: bool,
}

pub fn inspect(bytes: &[u8]) -> Result<Inspection> {
    ensure!(bytes.len() <= 1024 * 1024, "genesis input exceeds one MiB");
    let Unique(value) = serde_yaml::from_slice(bytes)
        .map_err(|_| anyhow::anyhow!("invalid or duplicate genesis YAML"))?;
    let top = [
        "ssfn_config_info",
        "validator_config_info",
        "parameters",
        "accounts",
        "validator_token_recipient",
    ];
    mapping(
        &value,
        &top,
        &[
            "ssfn_config_info",
            "validator_config_info",
            "parameters",
            "accounts",
        ],
    )?;
    // Explicit validator records contain private keys and defaults that generate keys/ports.
    // This local inspector refuses them rather than silently generating an effective identity.
    ensure!(
        value["ssfn_config_info"].is_null() && value["validator_config_info"].is_null(),
        "explicit validator or SSFN records require a dedicated reviewed inspection mode"
    );
    mapping(&value["parameters"], PARAMETERS, PARAMETERS)?;
    for key in PARAMETERS
        .iter()
        .filter(|key| **key != "allow_insertion_of_extra_objects")
    {
        ensure!(
            value["parameters"][*key].as_u64().is_some(),
            "genesis parameters require unsigned integers"
        );
    }
    ensure!(
        value["parameters"]["allow_insertion_of_extra_objects"]
            .as_bool()
            .is_some(),
        "explicit boolean required"
    );
    let accounts = value["accounts"]
        .as_sequence()
        .ok_or_else(|| anyhow::anyhow!("explicit accounts required"))?;
    ensure!(
        !accounts.is_empty(),
        "at least one explicit account required"
    );
    for account in accounts {
        mapping(
            account,
            &["address", "gas_amounts"],
            &["address", "gas_amounts"],
        )?;
        ensure!(
            account["address"].as_str().is_some(),
            "explicit account address required"
        );
        let amounts = account["gas_amounts"]
            .as_sequence()
            .ok_or_else(|| anyhow::anyhow!("gas amounts required"))?;
        ensure!(
            !amounts.is_empty() && amounts.iter().all(|n| n.as_u64().is_some()),
            "gas allocations require unsigned integers"
        );
    }
    let config: GenesisConfig = serde_yaml::from_value(value)
        .map_err(|_| anyhow::anyhow!("native genesis types rejected input"))?;
    let version = config.parameters.protocol_version;
    ensure!(
        version >= ProtocolVersion::MIN && version <= ProtocolVersion::MAX,
        "genesis protocol unsupported by this native build"
    );
    if config.parameters.epoch_duration_ms == 0
        || config.parameters.stake_subsidy_period_length == 0
        || config.parameters.stake_subsidy_decrease_rate > 10_000
    {
        bail!("invalid genesis epoch or subsidy parameters");
    }
    Ok(Inspection {
        version: 1,
        source_sha256: Hex::encode(Sha256::digest(bytes)),
        protocol_min: ProtocolVersion::MIN.as_u64(),
        protocol_max: ProtocolVersion::MAX.as_u64(),
        parameters: config.parameters,
        accounts: config.accounts,
        validator_token_recipient: config.validator_token_recipient,
        validator_allocation_policy: if config.validator_token_recipient.is_some() {
            "explicit-yaml-recipient"
        } else {
            "native-validator-defaults-no-recipient-override"
        },
        generated_genesis: false,
    })
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GenesisVerification {
    pub version: u8,
    pub source_sha256: String,
    pub genesis_blob_sha256: String,
    pub chain_id: String,
    pub committee_size: usize,
    pub protocol_version: u64,
    pub epoch_duration_ms: u64,
    pub chain_start_timestamp_ms: u64,
    pub allocation_policy: &'static str,
    pub allocations: Vec<VerifiedAllocation>,
    pub validator_allocations: Vec<ValidatorAllocation>,
    pub verified: bool,
}
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ValidatorAllocation {
    pub validator: rtd_types::base_types::RtdAddress,
    pub recipient: rtd_types::base_types::RtdAddress,
    pub native_liquid_mist: u64,
    pub staked_mist: u64,
}
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct VerifiedAllocation {
    pub address: rtd_types::base_types::RtdAddress,
    pub configured_coin_amounts: Vec<u64>,
    pub actual_coin_amounts: Vec<u64>,
    pub configured_total_mist: String,
    pub actual_total_mist: String,
}

/// Inspect the actual generated objects using the same native RTD types as the
/// genesis builder. This checks configuration/materialization, not an external
/// chain's checkpoint certificate or an independently supplied trust anchor.
pub fn verify_blob(input: &[u8], blob: &[u8]) -> Result<GenesisVerification> {
    use rtd_types::{
        gas_coin::GasCoin,
        governance::StakedRtd,
        object::Owner,
        rtd_system_state::{RtdSystemStateTrait, get_rtd_system_state},
    };
    use std::collections::{BTreeMap, BTreeSet};
    let inspection = inspect(input)?;
    ensure!(
        !blob.is_empty() && blob.len() <= 128 * 1024 * 1024,
        "invalid genesis blob size"
    );
    let genesis: rtd_config::genesis::Genesis = bcs::from_bytes(blob)?;
    ensure!(
        bcs::to_bytes(&genesis)? == blob,
        "noncanonical native genesis encoding"
    );
    let mut ids = BTreeSet::new();
    ensure!(
        genesis.objects().iter().all(|o| ids.insert(o.id())),
        "duplicate genesis object ID"
    );
    let summary = get_rtd_system_state(&genesis.objects())?.into_rtd_system_state_summary();
    let p = &inspection.parameters;
    ensure!(
        summary.epoch == 0
            && !summary.safe_mode
            && summary.protocol_version == p.protocol_version.as_u64()
            && summary.epoch_duration_ms == p.epoch_duration_ms
            && summary.epoch_start_timestamp_ms == p.chain_start_timestamp_ms
            && summary.stake_subsidy_start_epoch == p.stake_subsidy_start_epoch
            && summary.stake_subsidy_current_distribution_amount
                == p.stake_subsidy_initial_distribution_amount
            && summary.stake_subsidy_period_length == p.stake_subsidy_period_length
            && summary.stake_subsidy_decrease_rate == p.stake_subsidy_decrease_rate,
        "generated genesis parameters differ from authoritative YAML"
    );
    let mut expected: BTreeMap<_, Vec<u64>> = BTreeMap::new();
    for account in &inspection.accounts {
        let address = account
            .address
            .ok_or_else(|| anyhow::anyhow!("explicit allocation address required"))?;
        expected
            .entry(address)
            .or_default()
            .extend(&account.gas_amounts);
    }
    let mut actual: BTreeMap<_, Vec<u64>> = BTreeMap::new();
    let mut stakes = BTreeMap::new();
    for object in genesis.objects() {
        if let Owner::AddressOwner(owner) = object.owner()
            && let Ok(coin) = GasCoin::try_from(object)
        {
            actual.entry(*owner).or_default().push(coin.value());
        }
        if let Owner::AddressOwner(owner) = object.owner()
            && let Ok(stake) = StakedRtd::try_from(object)
        {
            stakes
                .entry(stake.pool_id())
                .or_insert_with(Vec::new)
                .push((*owner, stake.principal()));
        }
    }
    let mut validator_allocations = Vec::new();
    let mut validator_coins: BTreeMap<_, Vec<u64>> = BTreeMap::new();
    for validator in &summary.active_validators {
        let recipient = inspection
            .validator_token_recipient
            .unwrap_or(validator.rtd_address);
        ensure!(
            stakes.remove(&validator.staking_pool_id)
                == Some(vec![(recipient, validator.staking_pool_rtd_balance)]),
            "native validator stake allocation differs"
        );
        validator_coins
            .entry(recipient)
            .or_default()
            .push(crate::genesis_config::DEFAULT_GAS_AMOUNT);
        validator_allocations.push(ValidatorAllocation {
            validator: validator.rtd_address,
            recipient,
            native_liquid_mist: crate::genesis_config::DEFAULT_GAS_AMOUNT,
            staked_mist: validator.staking_pool_rtd_balance,
        });
    }
    // Validate the combined coin multiset, while keeping configured user funds,
    // native validator liquid funds and locked stake distinct in the report.
    let mut combined = expected.clone();
    for (address, coins) in validator_coins {
        combined.entry(address).or_default().extend(coins);
    }
    for (address, mut coins) in combined {
        coins.sort_unstable();
        let mut observed = actual.get(&address).cloned().unwrap_or_default();
        observed.sort_unstable();
        ensure!(
            coins == observed,
            "configured and native validator coin allocations differ"
        );
    }
    let mut allocations = Vec::new();
    for (address, mut configured) in expected {
        configured.sort_unstable();
        let mut observed = actual.remove(&address).unwrap_or_default();
        observed.sort_unstable();
        if inspection.validator_token_recipient == Some(address) {
            // Explicit YAML redirection permits native validator allocations in
            // addition to user allocations. Match the configured multiset first.
            let mut remaining = observed.clone();
            for amount in &configured {
                let index = remaining
                    .binary_search(amount)
                    .map_err(|_| anyhow::anyhow!("configured allocation missing"))?;
                remaining.remove(index);
            }
        } else {
            ensure!(
                configured == observed,
                "configured account allocations differ from generated coins"
            );
        }
        allocations.push(VerifiedAllocation {
            address,
            configured_total_mist: configured
                .iter()
                .map(|v| *v as u128)
                .sum::<u128>()
                .to_string(),
            actual_total_mist: observed
                .iter()
                .map(|v| *v as u128)
                .sum::<u128>()
                .to_string(),
            configured_coin_amounts: configured,
            actual_coin_amounts: observed,
        });
    }
    Ok(GenesisVerification {
        version: 1,
        source_sha256: inspection.source_sha256,
        genesis_blob_sha256: Hex::encode(Sha256::digest(blob)),
        chain_id: genesis.checkpoint().digest().to_string(),
        committee_size: genesis.committee().num_members(),
        protocol_version: summary.protocol_version,
        epoch_duration_ms: summary.epoch_duration_ms,
        chain_start_timestamp_ms: summary.epoch_start_timestamp_ms,
        allocation_policy: inspection.validator_allocation_policy,
        allocations,
        validator_allocations,
        verified: true,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    const INPUT: &str = "ssfn_config_info: null\nvalidator_config_info: null\nparameters:\n  chain_start_timestamp_ms: 1790173312486\n  protocol_version: 138\n  allow_insertion_of_extra_objects: true\n  epoch_duration_ms: 86400000\n  stake_subsidy_start_epoch: 0\n  stake_subsidy_initial_distribution_amount: 1000000000000000\n  stake_subsidy_period_length: 10\n  stake_subsidy_decrease_rate: 1000\naccounts:\n- address: '0x0000000000000000000000000000000000000000000000000000000000001234'\n  gas_amounts: [5200000000000000]\n";
    #[tokio::test]
    async fn preserves_authoritative_parameters_without_clock_or_recipient_override() {
        let a = inspect(INPUT.as_bytes()).unwrap();
        let b = inspect(INPUT.as_bytes()).unwrap();
        assert_eq!(
            serde_json::to_vec(&a).unwrap(),
            serde_json::to_vec(&b).unwrap()
        );
        assert_eq!(a.parameters.chain_start_timestamp_ms, 1790173312486);
        assert_eq!(a.parameters.epoch_duration_ms, 86400000);
        assert_eq!(a.parameters.protocol_version.as_u64(), 138);
        assert!(a.validator_token_recipient.is_none());
        assert_eq!(a.accounts[0].gas_amounts, vec![5200000000000000]);
        let explicit = format!(
            "{INPUT}validator_token_recipient: '0x0000000000000000000000000000000000000000000000000000000000005678'\n"
        );
        assert!(
            inspect(explicit.as_bytes())
                .unwrap()
                .validator_token_recipient
                .is_some()
        );
    }
    #[tokio::test]
    async fn rejects_implicit_unknown_ambiguous_and_incompatible_inputs() {
        for invalid in [
            INPUT.replace("  chain_start_timestamp_ms: 1790173312486\n", ""),
            INPUT.replace(
                "protocol_version: 138",
                "protocol_version: 18446744073709551615",
            ),
            INPUT.replace("protocol_version: 138", "protocol_version: true"),
            INPUT.replace(
                "epoch_duration_ms: 86400000",
                "epoch_duration_ms: '86400000'",
            ),
            INPUT.replace("epoch_duration_ms: 86400000", "epoch_duration_ms: 0"),
            INPUT.replace("'0x0000000000000000000000000000000000000000000000000000000000001234'", "null"),
            INPUT.replace("[5200000000000000]", "[-1]"),
            INPUT.replace("[5200000000000000]", "[true]"),
            INPUT.replace("[5200000000000000]", "[1.5]"),
            INPUT.replace("ssfn_config_info: null", "ssfn_config_info: []"),
            format!("{INPUT}unknown: true\n"),
            format!("{INPUT}accounts: []\n"),
            INPUT.replace(
                "  protocol_version: 138",
                "  protocol_version: 138\n  protocol_version: 139",
            ),
            INPUT.replace("  gas_amounts:", "  address: '0x0000000000000000000000000000000000000000000000000000000000005678'\n  gas_amounts:"),
            format!("{INPUT}\n---\n{INPUT}"),
        ] {
            assert!(inspect(invalid.as_bytes()).is_err());
        }
    }
    #[tokio::test]
    async fn verifies_materialized_native_genesis_and_rejects_different_authority() {
        let config: GenesisConfig = serde_yaml::from_str(INPUT).unwrap();
        let directory = tempfile::tempdir().unwrap();
        let network = crate::network_config_builder::ConfigBuilder::new(directory.path())
            .committee_size(std::num::NonZeroUsize::new(1).unwrap())
            .with_genesis_config(config)
            .build();
        let blob = network.genesis.to_bytes();
        let report = verify_blob(INPUT.as_bytes(), &blob).unwrap();
        assert_eq!(report.protocol_version, 138);
        assert_eq!(
            report.allocations[0].actual_coin_amounts,
            vec![5200000000000000]
        );
        assert_eq!(report.validator_allocations.len(), 1);
        assert_eq!(
            report.validator_allocations[0].recipient,
            report.validator_allocations[0].validator
        );
        assert!(report.validator_allocations[0].staked_mist > 0);
        for changed in [
            INPUT.replace("protocol_version: 138", "protocol_version: 139"),
            INPUT.replace("epoch_duration_ms: 86400000", "epoch_duration_ms: 300000"),
            INPUT.replace("1790173312486", "1790173312487"),
            INPUT.replace("[5200000000000000]", "[5200000000000001]"),
            INPUT.replace(
                "stake_subsidy_period_length: 10",
                "stake_subsidy_period_length: 11",
            ),
        ] {
            assert!(verify_blob(changed.as_bytes(), &blob).is_err());
        }
        assert!(verify_blob(INPUT.as_bytes(), &blob[..blob.len() - 1]).is_err());
        let mut trailing = blob.clone();
        trailing.push(0);
        assert!(verify_blob(INPUT.as_bytes(), &trailing).is_err());
    }

    #[tokio::test]
    async fn separates_redirected_validator_liquid_funds_and_locked_stake() {
        let input = format!(
            "{INPUT}validator_token_recipient: '0x0000000000000000000000000000000000000000000000000000000000001234'\n"
        );
        let config: GenesisConfig = serde_yaml::from_str(&input).unwrap();
        let directory = tempfile::tempdir().unwrap();
        let network = crate::network_config_builder::ConfigBuilder::new(directory.path())
            .committee_size(std::num::NonZeroUsize::new(2).unwrap())
            .with_genesis_config(config)
            .build();
        let report = verify_blob(input.as_bytes(), &network.genesis.to_bytes()).unwrap();
        assert_eq!(report.validator_allocations.len(), 2);
        assert_eq!(
            report.allocations[0].configured_total_mist,
            "5200000000000000"
        );
        assert_eq!(
            report.allocations[0].actual_total_mist,
            (5200000000000000u128 + 2 * crate::genesis_config::DEFAULT_GAS_AMOUNT as u128)
                .to_string()
        );
        for validator in report.validator_allocations {
            assert_eq!(validator.recipient, report.allocations[0].address);
            assert_ne!(validator.recipient, validator.validator);
            assert!(validator.staked_mist > 0);
        }
    }
}
