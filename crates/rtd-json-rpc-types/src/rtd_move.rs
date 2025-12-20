// Copyright (c) LinkU Labs, Inc.
// SPDX-License-Identifier: Apache-2.0

use colored::Colorize;
use itertools::Itertools;
use move_binary_format::file_format::{Ability, AbilitySet, DatatypeTyParameter, Visibility};
use move_binary_format::normalized::{
    self, Enum as NormalizedEnum, Field as NormalizedField, Function as NormalizedFunction,
    Module as NormalizedModule, Struct as NormalizedStruct, Type as NormalizedType,
};
use move_command_line_common::error_bitset::ErrorBitset;
use move_core_types::annotated_value::{MoveStruct, MoveValue, MoveVariant};
use move_core_types::identifier::Identifier;
use move_core_types::language_storage::StructTag;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use serde_with::serde_as;
use std::collections::BTreeMap;
use std::fmt;
use std::fmt::{Display, Formatter, Write};
use std::hash::Hash;
use rtd_macros::EnumVariantOrder;
use tracing::warn;

use rtd_types::base_types::{ObjectID, RtdAddress};
use rtd_types::execution_status::MoveLocation;
use rtd_types::rtd_serde::RtdStructTag;

pub type RtdMoveTypeParameterIndex = u16;

#[cfg(test)]
#[path = "unit_tests/rtd_move_tests.rs"]
mod rtd_move_tests;

#[derive(Serialize, Deserialize, Debug, JsonSchema, Clone)]
pub enum RtdMoveAbility {
    Copy,
    Drop,
    Store,
    Key,
}

#[derive(Serialize, Deserialize, Debug, JsonSchema, Clone)]
pub struct RtdMoveAbilitySet {
    pub abilities: Vec<RtdMoveAbility>,
}

#[derive(Serialize, Deserialize, Debug, JsonSchema, Clone)]
pub enum RtdMoveVisibility {
    Private,
    Public,
    Friend,
}

#[derive(Serialize, Deserialize, Debug, JsonSchema, Clone)]
#[serde(rename_all = "camelCase")]
pub struct RtdMoveStructTypeParameter {
    pub constraints: RtdMoveAbilitySet,
    pub is_phantom: bool,
}

#[derive(Serialize, Deserialize, Debug, JsonSchema, Clone)]
pub struct RtdMoveNormalizedField {
    pub name: String,
    #[serde(rename = "type")]
    pub type_: RtdMoveNormalizedType,
}

#[derive(Serialize, Deserialize, Debug, JsonSchema, Clone)]
#[serde(rename_all = "camelCase")]
pub struct RtdMoveNormalizedStruct {
    pub abilities: RtdMoveAbilitySet,
    pub type_parameters: Vec<RtdMoveStructTypeParameter>,
    pub fields: Vec<RtdMoveNormalizedField>,
}

#[derive(Serialize, Deserialize, Debug, JsonSchema, Clone)]
#[serde(rename_all = "camelCase")]
pub struct RtdMoveNormalizedEnum {
    pub abilities: RtdMoveAbilitySet,
    pub type_parameters: Vec<RtdMoveStructTypeParameter>,
    pub variants: BTreeMap<String, Vec<RtdMoveNormalizedField>>,
    #[serde(default)]
    pub variant_declaration_order: Option<Vec<String>>,
}

#[derive(Serialize, Deserialize, Debug, JsonSchema, Clone)]
pub enum RtdMoveNormalizedType {
    Bool,
    U8,
    U16,
    U32,
    U64,
    U128,
    U256,
    Address,
    Signer,
    Struct {
        #[serde(flatten)]
        inner: Box<RtdMoveNormalizedStructType>,
    },
    Vector(Box<RtdMoveNormalizedType>),
    TypeParameter(RtdMoveTypeParameterIndex),
    Reference(Box<RtdMoveNormalizedType>),
    MutableReference(Box<RtdMoveNormalizedType>),
}

#[derive(Serialize, Deserialize, Debug, JsonSchema, Clone)]
#[serde(rename_all = "camelCase")]
pub struct RtdMoveNormalizedStructType {
    pub address: String,
    pub module: String,
    pub name: String,
    pub type_arguments: Vec<RtdMoveNormalizedType>,
}

#[derive(Serialize, Deserialize, Debug, JsonSchema, Clone)]
#[serde(rename_all = "camelCase")]
pub struct RtdMoveNormalizedFunction {
    pub visibility: RtdMoveVisibility,
    pub is_entry: bool,
    pub type_parameters: Vec<RtdMoveAbilitySet>,
    pub parameters: Vec<RtdMoveNormalizedType>,
    pub return_: Vec<RtdMoveNormalizedType>,
}

#[derive(Serialize, Deserialize, Debug, JsonSchema, Clone)]
pub struct RtdMoveModuleId {
    address: String,
    name: String,
}

#[derive(Serialize, Deserialize, Debug, JsonSchema, Clone)]
#[serde(rename_all = "camelCase")]
pub struct RtdMoveNormalizedModule {
    pub file_format_version: u32,
    pub address: String,
    pub name: String,
    pub friends: Vec<RtdMoveModuleId>,
    pub structs: BTreeMap<String, RtdMoveNormalizedStruct>,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub enums: BTreeMap<String, RtdMoveNormalizedEnum>,
    pub exposed_functions: BTreeMap<String, RtdMoveNormalizedFunction>,
}

impl PartialEq for RtdMoveNormalizedModule {
    fn eq(&self, other: &Self) -> bool {
        self.file_format_version == other.file_format_version
            && self.address == other.address
            && self.name == other.name
    }
}

impl<S: std::hash::Hash + Eq + ToString> From<&NormalizedModule<S>> for RtdMoveNormalizedModule {
    fn from(module: &NormalizedModule<S>) -> Self {
        Self {
            file_format_version: module.file_format_version,
            address: module.address().to_hex_literal(),
            name: module.name().to_string(),
            friends: module
                .friends
                .iter()
                .map(|module_id| RtdMoveModuleId {
                    address: module_id.address.to_hex_literal(),
                    name: module_id.name.to_string(),
                })
                .collect::<Vec<RtdMoveModuleId>>(),
            structs: module
                .structs
                .iter()
                .map(|(name, struct_)| {
                    (name.to_string(), RtdMoveNormalizedStruct::from(&**struct_))
                })
                .collect::<BTreeMap<String, RtdMoveNormalizedStruct>>(),
            enums: module
                .enums
                .iter()
                .map(|(name, enum_)| (name.to_string(), RtdMoveNormalizedEnum::from(&**enum_)))
                .collect(),
            exposed_functions: module
                .functions
                .iter()
                .filter(|(_name, function)| {
                    function.is_entry || function.visibility != Visibility::Private
                })
                .map(|(name, function)| {
                    // TODO: Do we want to expose the private functions as well?

                    (
                        name.to_string(),
                        RtdMoveNormalizedFunction::from(&**function),
                    )
                })
                .collect::<BTreeMap<String, RtdMoveNormalizedFunction>>(),
        }
    }
}

impl<S: Hash + Eq + ToString> From<&NormalizedFunction<S>> for RtdMoveNormalizedFunction {
    fn from(function: &NormalizedFunction<S>) -> Self {
        Self {
            visibility: match function.visibility {
                Visibility::Private => RtdMoveVisibility::Private,
                Visibility::Public => RtdMoveVisibility::Public,
                Visibility::Friend => RtdMoveVisibility::Friend,
            },
            is_entry: function.is_entry,
            type_parameters: function
                .type_parameters
                .iter()
                .copied()
                .map(|a| a.into())
                .collect::<Vec<RtdMoveAbilitySet>>(),
            parameters: function
                .parameters
                .iter()
                .map(|t| RtdMoveNormalizedType::from(&**t))
                .collect::<Vec<RtdMoveNormalizedType>>(),
            return_: function
                .return_
                .iter()
                .map(|t| RtdMoveNormalizedType::from(&**t))
                .collect::<Vec<RtdMoveNormalizedType>>(),
        }
    }
}

impl<S: Hash + Eq + ToString> From<&NormalizedStruct<S>> for RtdMoveNormalizedStruct {
    fn from(struct_: &NormalizedStruct<S>) -> Self {
        Self {
            abilities: struct_.abilities.into(),
            type_parameters: struct_
                .type_parameters
                .iter()
                .copied()
                .map(RtdMoveStructTypeParameter::from)
                .collect::<Vec<RtdMoveStructTypeParameter>>(),
            fields: struct_
                .fields
                .0
                .values()
                .map(|f| RtdMoveNormalizedField::from(&**f))
                .collect::<Vec<RtdMoveNormalizedField>>(),
        }
    }
}

impl<S: Hash + Eq + ToString> From<&NormalizedEnum<S>> for RtdMoveNormalizedEnum {
    fn from(value: &NormalizedEnum<S>) -> Self {
        let variants = value
            .variants
            .values()
            .map(|variant| {
                (
                    variant.name.to_string(),
                    variant
                        .fields
                        .0
                        .values()
                        .map(|f| RtdMoveNormalizedField::from(&**f))
                        .collect::<Vec<RtdMoveNormalizedField>>(),
                )
            })
            .collect::<Vec<(String, Vec<RtdMoveNormalizedField>)>>();
        let variant_declaration_order = variants
            .iter()
            .map(|(name, _)| name.clone())
            .collect::<Vec<String>>();
        let variants = variants.into_iter().collect();
        Self {
            abilities: value.abilities.into(),
            type_parameters: value
                .type_parameters
                .iter()
                .copied()
                .map(RtdMoveStructTypeParameter::from)
                .collect::<Vec<RtdMoveStructTypeParameter>>(),
            variants,
            variant_declaration_order: Some(variant_declaration_order),
        }
    }
}

impl From<DatatypeTyParameter> for RtdMoveStructTypeParameter {
    fn from(type_parameter: DatatypeTyParameter) -> Self {
        Self {
            constraints: type_parameter.constraints.into(),
            is_phantom: type_parameter.is_phantom,
        }
    }
}

impl<S: ToString> From<&NormalizedField<S>> for RtdMoveNormalizedField {
    fn from(normalized_field: &NormalizedField<S>) -> Self {
        Self {
            name: normalized_field.name.to_string(),
            type_: RtdMoveNormalizedType::from(&normalized_field.type_),
        }
    }
}

impl<S: ToString> From<&NormalizedType<S>> for RtdMoveNormalizedType {
    fn from(type_: &NormalizedType<S>) -> Self {
        match type_ {
            NormalizedType::Bool => RtdMoveNormalizedType::Bool,
            NormalizedType::U8 => RtdMoveNormalizedType::U8,
            NormalizedType::U16 => RtdMoveNormalizedType::U16,
            NormalizedType::U32 => RtdMoveNormalizedType::U32,
            NormalizedType::U64 => RtdMoveNormalizedType::U64,
            NormalizedType::U128 => RtdMoveNormalizedType::U128,
            NormalizedType::U256 => RtdMoveNormalizedType::U256,
            NormalizedType::Address => RtdMoveNormalizedType::Address,
            NormalizedType::Signer => RtdMoveNormalizedType::Signer,
            NormalizedType::Datatype(dt) => {
                let normalized::Datatype {
                    module,
                    name,
                    type_arguments,
                } = &**dt;
                RtdMoveNormalizedType::new_struct(
                    module.address.to_hex_literal(),
                    module.name.to_string(),
                    name.to_string(),
                    type_arguments
                        .iter()
                        .map(RtdMoveNormalizedType::from)
                        .collect::<Vec<RtdMoveNormalizedType>>(),
                )
            }
            NormalizedType::Vector(v) => {
                RtdMoveNormalizedType::Vector(Box::new(RtdMoveNormalizedType::from(&**v)))
            }
            NormalizedType::TypeParameter(t) => RtdMoveNormalizedType::TypeParameter(*t),
            NormalizedType::Reference(false, r) => {
                RtdMoveNormalizedType::Reference(Box::new(RtdMoveNormalizedType::from(&**r)))
            }
            NormalizedType::Reference(true, mr) => RtdMoveNormalizedType::MutableReference(
                Box::new(RtdMoveNormalizedType::from(&**mr)),
            ),
        }
    }
}

impl From<AbilitySet> for RtdMoveAbilitySet {
    fn from(set: AbilitySet) -> RtdMoveAbilitySet {
        Self {
            abilities: set
                .into_iter()
                .map(|a| match a {
                    Ability::Copy => RtdMoveAbility::Copy,
                    Ability::Drop => RtdMoveAbility::Drop,
                    Ability::Key => RtdMoveAbility::Key,
                    Ability::Store => RtdMoveAbility::Store,
                })
                .collect::<Vec<RtdMoveAbility>>(),
        }
    }
}

impl RtdMoveNormalizedType {
    pub fn new_struct(
        address: String,
        module: String,
        name: String,
        type_arguments: Vec<RtdMoveNormalizedType>,
    ) -> Self {
        RtdMoveNormalizedType::Struct {
            inner: Box::new(RtdMoveNormalizedStructType {
                address,
                module,
                name,
                type_arguments,
            }),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, JsonSchema, Clone)]
pub enum ObjectValueKind {
    ByImmutableReference,
    ByMutableReference,
    ByValue,
}

#[derive(Serialize, Deserialize, Debug, JsonSchema, Clone)]
pub enum MoveFunctionArgType {
    Pure,
    Object(ObjectValueKind),
}

#[serde_as]
#[derive(Debug, Deserialize, Serialize, JsonSchema, Clone, Eq, PartialEq, EnumVariantOrder)]
#[serde(untagged, rename = "MoveValue")]
pub enum RtdMoveValue {
    // u64 and u128 are converted to String to avoid overflow
    Number(u32),
    Bool(bool),
    Address(RtdAddress),
    Vector(Vec<RtdMoveValue>),
    String(String),
    UID { id: ObjectID },
    Struct(RtdMoveStruct),
    Option(Box<Option<RtdMoveValue>>),
    Variant(RtdMoveVariant),
}

impl RtdMoveValue {
    /// Extract values from MoveValue without type information in json format
    pub fn to_json_value(self) -> Value {
        match self {
            RtdMoveValue::Struct(move_struct) => move_struct.to_json_value(),
            RtdMoveValue::Vector(values) => RtdMoveStruct::Runtime(values).to_json_value(),
            RtdMoveValue::Number(v) => json!(v),
            RtdMoveValue::Bool(v) => json!(v),
            RtdMoveValue::Address(v) => json!(v),
            RtdMoveValue::String(v) => json!(v),
            RtdMoveValue::UID { id } => json!({ "id": id }),
            RtdMoveValue::Option(v) => json!(v),
            RtdMoveValue::Variant(v) => v.to_json_value(),
        }
    }
}

impl Display for RtdMoveValue {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let mut writer = String::new();
        match self {
            RtdMoveValue::Number(value) => write!(writer, "{}", value)?,
            RtdMoveValue::Bool(value) => write!(writer, "{}", value)?,
            RtdMoveValue::Address(value) => write!(writer, "{}", value)?,
            RtdMoveValue::String(value) => write!(writer, "{}", value)?,
            RtdMoveValue::UID { id } => write!(writer, "{id}")?,
            RtdMoveValue::Struct(value) => write!(writer, "{}", value)?,
            RtdMoveValue::Option(value) => write!(writer, "{:?}", value)?,
            RtdMoveValue::Vector(vec) => {
                write!(
                    writer,
                    "{}",
                    vec.iter().map(|value| format!("{value}")).join(",\n")
                )?;
            }
            RtdMoveValue::Variant(value) => write!(writer, "{}", value)?,
        }
        write!(f, "{}", writer.trim_end_matches('\n'))
    }
}

impl From<MoveValue> for RtdMoveValue {
    fn from(value: MoveValue) -> Self {
        match value {
            MoveValue::U8(value) => RtdMoveValue::Number(value.into()),
            MoveValue::U16(value) => RtdMoveValue::Number(value.into()),
            MoveValue::U32(value) => RtdMoveValue::Number(value),
            MoveValue::U64(value) => RtdMoveValue::String(format!("{value}")),
            MoveValue::U128(value) => RtdMoveValue::String(format!("{value}")),
            MoveValue::U256(value) => RtdMoveValue::String(format!("{value}")),
            MoveValue::Bool(value) => RtdMoveValue::Bool(value),
            MoveValue::Vector(values) => {
                RtdMoveValue::Vector(values.into_iter().map(|value| value.into()).collect())
            }
            MoveValue::Struct(value) => {
                // Best effort Rtd core type conversion
                let MoveStruct { type_, fields } = &value;
                if let Some(value) = try_convert_type(type_, fields) {
                    return value;
                }
                RtdMoveValue::Struct(value.into())
            }
            MoveValue::Signer(value) | MoveValue::Address(value) => {
                RtdMoveValue::Address(RtdAddress::from(ObjectID::from(value)))
            }
            MoveValue::Variant(MoveVariant {
                type_,
                variant_name,
                tag: _,
                fields,
            }) => RtdMoveValue::Variant(RtdMoveVariant {
                type_: type_.clone(),
                variant: variant_name.to_string(),
                fields: fields
                    .into_iter()
                    .map(|(id, value)| (id.into_string(), value.into()))
                    .collect::<BTreeMap<_, _>>(),
            }),
        }
    }
}

fn to_bytearray(value: &[MoveValue]) -> Option<Vec<u8>> {
    if value.iter().all(|value| matches!(value, MoveValue::U8(_))) {
        let bytearray = value
            .iter()
            .flat_map(|value| {
                if let MoveValue::U8(u8) = value {
                    Some(*u8)
                } else {
                    None
                }
            })
            .collect::<Vec<_>>();
        Some(bytearray)
    } else {
        None
    }
}

#[serde_as]
#[derive(Debug, Deserialize, Serialize, JsonSchema, Clone, Eq, PartialEq)]
#[serde(rename = "MoveVariant")]
pub struct RtdMoveVariant {
    #[schemars(with = "String")]
    #[serde(rename = "type")]
    #[serde_as(as = "RtdStructTag")]
    pub type_: StructTag,
    pub variant: String,
    pub fields: BTreeMap<String, RtdMoveValue>,
}

impl RtdMoveVariant {
    pub fn to_json_value(self) -> Value {
        // We only care about values here, assuming type information is known at the client side.
        let fields = self
            .fields
            .into_iter()
            .map(|(key, value)| (key, value.to_json_value()))
            .collect::<BTreeMap<_, _>>();
        json!({
            "variant": self.variant,
            "fields": fields,
        })
    }
}

impl Display for RtdMoveVariant {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let mut writer = String::new();
        let RtdMoveVariant {
            type_,
            variant,
            fields,
        } = self;
        writeln!(writer)?;
        writeln!(writer, "  {}: {type_}", "type".bold().bright_black())?;
        writeln!(writer, "  {}: {variant}", "variant".bold().bright_black())?;
        for (name, value) in fields {
            let value = format!("{}", value);
            let value = if value.starts_with('\n') {
                indent(&value, 2)
            } else {
                value
            };
            writeln!(writer, "  {}: {value}", name.bold().bright_black())?;
        }

        write!(f, "{}", writer.trim_end_matches('\n'))
    }
}

#[serde_as]
#[derive(Debug, Deserialize, Serialize, JsonSchema, Clone, Eq, PartialEq, EnumVariantOrder)]
#[serde(untagged, rename = "MoveStruct")]
pub enum RtdMoveStruct {
    Runtime(Vec<RtdMoveValue>),
    WithTypes {
        #[schemars(with = "String")]
        #[serde(rename = "type")]
        #[serde_as(as = "RtdStructTag")]
        type_: StructTag,
        fields: BTreeMap<String, RtdMoveValue>,
    },
    WithFields(BTreeMap<String, RtdMoveValue>),
}

impl RtdMoveStruct {
    /// Extract values from MoveStruct without type information in json format
    pub fn to_json_value(self) -> Value {
        // Unwrap MoveStructs
        match self {
            RtdMoveStruct::Runtime(values) => {
                let values = values
                    .into_iter()
                    .map(|value| value.to_json_value())
                    .collect::<Vec<_>>();
                json!(values)
            }
            // We only care about values here, assuming struct type information is known at the client side.
            RtdMoveStruct::WithTypes { type_: _, fields } | RtdMoveStruct::WithFields(fields) => {
                let fields = fields
                    .into_iter()
                    .map(|(key, value)| (key, value.to_json_value()))
                    .collect::<BTreeMap<_, _>>();
                json!(fields)
            }
        }
    }

    pub fn field_value(&self, field_name: &str) -> Option<RtdMoveValue> {
        match self {
            RtdMoveStruct::WithFields(fields) => fields.get(field_name).cloned(),
            RtdMoveStruct::WithTypes { type_: _, fields } => fields.get(field_name).cloned(),
            _ => None,
        }
    }
}

impl Display for RtdMoveStruct {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let mut writer = String::new();
        match self {
            RtdMoveStruct::Runtime(_) => {}
            RtdMoveStruct::WithFields(fields) => {
                for (name, value) in fields {
                    writeln!(writer, "{}: {value}", name.bold().bright_black())?;
                }
            }
            RtdMoveStruct::WithTypes { type_, fields } => {
                writeln!(writer)?;
                writeln!(writer, "  {}: {type_}", "type".bold().bright_black())?;
                for (name, value) in fields {
                    let value = format!("{}", value);
                    let value = if value.starts_with('\n') {
                        indent(&value, 2)
                    } else {
                        value
                    };
                    writeln!(writer, "  {}: {value}", name.bold().bright_black())?;
                }
            }
        }
        write!(f, "{}", writer.trim_end_matches('\n'))
    }
}

#[derive(Eq, PartialEq, Clone, Debug, Serialize, Deserialize, JsonSchema)]
pub struct RtdMoveAbort {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub module_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub function: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub line: Option<u16>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error_code: Option<u64>,
}

impl RtdMoveAbort {
    pub fn new(move_location: MoveLocation, code: u64) -> Self {
        let module = move_location.module.to_canonical_string(true);
        let (error_code, line) = match ErrorBitset::from_u64(code) {
            Some(c) => (c.error_code().map(|c| c as u64), c.line_number()),
            None => (Some(code), None),
        };
        Self {
            module_id: Some(module),
            function: move_location.function_name.clone(),
            line,
            error_code,
        }
    }
}

impl Display for RtdMoveAbort {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let mut writer = String::new();
        if let Some(module_id) = &self.module_id {
            writeln!(writer, "Module ID: {module_id}")?;
        }
        if let Some(function) = &self.function {
            writeln!(writer, "Function: {function}")?;
        }
        if let Some(line) = &self.line {
            writeln!(writer, "Line: {line}")?;
        }
        if let Some(error_code) = &self.error_code {
            writeln!(writer, "Error code: {error_code}")?;
        }
        write!(f, "{}", writer.trim_end_matches('\n'))
    }
}

fn indent<T: Display>(d: &T, indent: usize) -> String {
    d.to_string()
        .lines()
        .map(|line| format!("{:indent$}{}", "", line))
        .join("\n")
}

fn try_convert_type(type_: &StructTag, fields: &[(Identifier, MoveValue)]) -> Option<RtdMoveValue> {
    let struct_name = format!(
        "0x{}::{}::{}",
        type_.address.short_str_lossless(),
        type_.module,
        type_.name
    );
    let mut values = fields
        .iter()
        .map(|(id, value)| (id.to_string(), value))
        .collect::<BTreeMap<_, _>>();
    match struct_name.as_str() {
        "0x1::string::String" | "0x1::ascii::String" => {
            if let Some(MoveValue::Vector(bytes)) = values.remove("bytes") {
                return to_bytearray(bytes)
                    .and_then(|bytes| String::from_utf8(bytes).ok())
                    .map(RtdMoveValue::String);
            }
        }
        "0x2::url::Url" => {
            return values.remove("url").cloned().map(RtdMoveValue::from);
        }
        "0x2::object::ID" => {
            return values.remove("bytes").cloned().map(RtdMoveValue::from);
        }
        "0x2::object::UID" => {
            let id = values.remove("id").cloned().map(RtdMoveValue::from);
            if let Some(RtdMoveValue::Address(address)) = id {
                return Some(RtdMoveValue::UID {
                    id: ObjectID::from(address),
                });
            }
        }
        "0x2::balance::Balance" => {
            return values.remove("value").cloned().map(RtdMoveValue::from);
        }
        "0x1::option::Option" => {
            if let Some(MoveValue::Vector(values)) = values.remove("vec") {
                return Some(RtdMoveValue::Option(Box::new(
                    // in Move option is modeled as vec of 1 element
                    values.first().cloned().map(RtdMoveValue::from),
                )));
            }
        }
        _ => return None,
    }
    warn!(
        fields =? fields,
        "Failed to convert {struct_name} to RtdMoveValue"
    );
    None
}

impl From<MoveStruct> for RtdMoveStruct {
    fn from(move_struct: MoveStruct) -> Self {
        RtdMoveStruct::WithTypes {
            type_: move_struct.type_,
            fields: move_struct
                .fields
                .into_iter()
                .map(|(id, value)| (id.into_string(), value.into()))
                .collect(),
        }
    }
}

#[test]
fn enum_size() {
    assert_eq!(std::mem::size_of::<RtdMoveNormalizedType>(), 16);
}
