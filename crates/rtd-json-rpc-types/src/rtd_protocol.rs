// Copyright (c) LinkU Labs, Inc.
// SPDX-License-Identifier: Apache-2.0

use std::collections::BTreeMap;

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_with::DisplayFromStr;
use serde_with::serde_as;
use rtd_protocol_config::{ProtocolConfig, ProtocolConfigValue, ProtocolVersion};
use rtd_types::rtd_serde::Readable;
use rtd_types::rtd_serde::{AsProtocolVersion, BigInt};

#[serde_as]
#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema, PartialEq)]
#[serde(rename_all = "camelCase", rename = "ProtocolConfigValue")]
pub enum RtdProtocolConfigValue {
    U16(
        #[schemars(with = "BigInt<u16>")]
        #[serde_as(as = "BigInt<u16>")]
        u16,
    ),
    U32(
        #[schemars(with = "BigInt<u32>")]
        #[serde_as(as = "BigInt<u32>")]
        u32,
    ),
    U64(
        #[schemars(with = "BigInt<u64>")]
        #[serde_as(as = "BigInt<u64>")]
        u64,
    ),
    F64(
        #[schemars(with = "String")]
        #[serde_as(as = "DisplayFromStr")]
        f64,
    ),
    Bool(
        #[schemars(with = "String")]
        #[serde_as(as = "DisplayFromStr")]
        bool,
    ),
}

impl From<ProtocolConfigValue> for RtdProtocolConfigValue {
    fn from(value: ProtocolConfigValue) -> Self {
        match value {
            ProtocolConfigValue::u16(y) => RtdProtocolConfigValue::U16(y),
            ProtocolConfigValue::u32(y) => RtdProtocolConfigValue::U32(y),
            ProtocolConfigValue::u64(x) => RtdProtocolConfigValue::U64(x),
            ProtocolConfigValue::bool(z) => RtdProtocolConfigValue::Bool(z),
        }
    }
}

#[serde_as]
#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema, PartialEq)]
#[serde(rename_all = "camelCase", rename = "ProtocolConfig")]
pub struct ProtocolConfigResponse {
    #[schemars(with = "AsProtocolVersion")]
    #[serde_as(as = "Readable<AsProtocolVersion, _>")]
    pub min_supported_protocol_version: ProtocolVersion,
    #[schemars(with = "AsProtocolVersion")]
    #[serde_as(as = "Readable<AsProtocolVersion, _>")]
    pub max_supported_protocol_version: ProtocolVersion,
    #[schemars(with = "AsProtocolVersion")]
    #[serde_as(as = "Readable<AsProtocolVersion, _>")]
    pub protocol_version: ProtocolVersion,
    pub feature_flags: BTreeMap<String, bool>,
    pub attributes: BTreeMap<String, Option<RtdProtocolConfigValue>>,
}

impl From<ProtocolConfig> for ProtocolConfigResponse {
    fn from(config: ProtocolConfig) -> Self {
        ProtocolConfigResponse {
            protocol_version: config.version,
            attributes: config
                .attr_map()
                .into_iter()
                .map(|(k, v)| (k, v.map(RtdProtocolConfigValue::from)))
                .collect(),
            min_supported_protocol_version: ProtocolVersion::MIN,
            max_supported_protocol_version: ProtocolVersion::MAX,
            feature_flags: config.feature_map(),
        }
    }
}
