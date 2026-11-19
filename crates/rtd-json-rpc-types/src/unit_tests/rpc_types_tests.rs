// Copyright (c) LinkU Labs, Inc.
// SPDX-License-Identifier: Apache-2.0

use std::str::FromStr;

use anyhow::anyhow;
use move_core_types::annotated_value::{MoveStruct, MoveValue};
use move_core_types::ident_str;
use move_core_types::identifier::Identifier;
use move_core_types::language_storage::{StructTag, TypeTag};
use serde_json::json;

use rtd_types::base_types::{ObjectDigest, SequenceNumber};
use rtd_types::base_types::{ObjectID, RtdAddress};
use rtd_types::gas_coin::GasCoin;
use rtd_types::object::{MoveObject, Owner};
use rtd_types::{MOVE_STDLIB_ADDRESS, RTD_FRAMEWORK_ADDRESS, parse_rtd_struct_tag};

use crate::{ObjectChange, RtdMoveStruct, RtdMoveValue};

#[test]
fn test_move_value_to_rtd_coin() {
    let id = ObjectID::random();
    let value = 10000;
    let coin = GasCoin::new(id, value);

    let move_object = MoveObject::new_gas_coin(SequenceNumber::new(), id, value);
    let layout = GasCoin::layout();

    let move_struct = move_object.to_move_struct(&layout).unwrap();
    let rtd_struct = RtdMoveStruct::from(move_struct);
    let gas_coin = GasCoin::try_from(&rtd_struct).unwrap();
    assert_eq!(coin.value(), gas_coin.value());
    assert_eq!(coin.id(), gas_coin.id());
}

#[test]
fn test_move_value_to_string() {
    let test_string = "Some test string";
    let bytes = test_string.as_bytes();
    let values = bytes
        .iter()
        .map(|u8| MoveValue::U8(*u8))
        .collect::<Vec<_>>();

    let move_value = MoveValue::Struct(MoveStruct {
        type_: StructTag {
            address: MOVE_STDLIB_ADDRESS,
            module: ident_str!("string").to_owned(),
            name: ident_str!("String").to_owned(),
            type_params: vec![],
        },
        fields: vec![(ident_str!("bytes").to_owned(), MoveValue::Vector(values))],
    });

    let rtd_value = RtdMoveValue::from(move_value);

    assert!(matches!(rtd_value, RtdMoveValue::String(s) if s == test_string));
}

#[test]
fn test_option() {
    // bugfix for https://github.com/LinkUVerse/rtd/issues/4995
    let option = MoveValue::Struct(MoveStruct {
        type_: StructTag {
            address: MOVE_STDLIB_ADDRESS,
            module: Identifier::from_str("option").unwrap(),
            name: Identifier::from_str("Option").unwrap(),
            type_params: vec![TypeTag::U8],
        },
        fields: vec![(
            Identifier::from_str("vec").unwrap(),
            MoveValue::Vector(vec![MoveValue::U8(5)]),
        )],
    });
    let rtd_value = RtdMoveValue::from(option);
    assert!(matches!(
        rtd_value,
        RtdMoveValue::Option(value) if *value == Some(RtdMoveValue::Number(5))
    ));
}

#[test]
fn test_move_value_to_url() {
    let test_url = "http://testing.com";
    let bytes = test_url.as_bytes();
    let values = bytes
        .iter()
        .map(|u8| MoveValue::U8(*u8))
        .collect::<Vec<_>>();

    let string_move_value = MoveValue::Struct(MoveStruct {
        type_: StructTag {
            address: MOVE_STDLIB_ADDRESS,
            module: ident_str!("string").to_owned(),
            name: ident_str!("String").to_owned(),
            type_params: vec![],
        },
        fields: vec![(ident_str!("bytes").to_owned(), MoveValue::Vector(values))],
    });

    let url_move_value = MoveValue::Struct(MoveStruct {
        type_: StructTag {
            address: RTD_FRAMEWORK_ADDRESS,
            module: ident_str!("url").to_owned(),
            name: ident_str!("Url").to_owned(),
            type_params: vec![],
        },
        fields: vec![(ident_str!("url").to_owned(), string_move_value)],
    });

    let rtd_value = RtdMoveValue::from(url_move_value);

    assert!(matches!(rtd_value, RtdMoveValue::String(s) if s == test_url));
}

#[test]
fn test_serde() {
    let test_values = [
        RtdMoveValue::Number(u32::MAX),
        RtdMoveValue::UID {
            id: ObjectID::random(),
        },
        RtdMoveValue::String("some test string".to_string()),
        RtdMoveValue::Address(RtdAddress::random_for_testing_only()),
        RtdMoveValue::Bool(true),
        RtdMoveValue::Option(Box::new(None)),
        RtdMoveValue::Vector(vec![
            RtdMoveValue::Number(1000000),
            RtdMoveValue::Number(2000000),
            RtdMoveValue::Number(3000000),
        ]),
    ];

    for value in test_values {
        let json = serde_json::to_string(&value).unwrap();
        let serde_value: RtdMoveValue = serde_json::from_str(&json)
            .map_err(|e| anyhow!("Serde failed for [{:?}], Error msg : {}", value, e))
            .unwrap();
        assert_eq!(
            value, serde_value,
            "Error converting {:?} [{json}], got {:?}",
            value, serde_value
        )
    }
}

#[test]
fn test_move_type_serde() {
    use crate::rtd_move as SM;
    use crate::rtd_move::RtdMoveNormalizedType as SNT;
    let test_types = vec![
        SNT::Bool,
        SNT::U8,
        SNT::U16,
        SNT::U32,
        SNT::U64,
        SNT::U128,
        SNT::U256,
        SNT::Address,
        SNT::Signer,
        SNT::Vector(Box::new(SNT::U8)),
        SNT::Struct {
            inner: Box::new(SM::RtdMoveNormalizedStructType {
                address: RTD_FRAMEWORK_ADDRESS.to_string(),
                module: "coin".to_owned(),
                name: "Coin".to_owned(),
                type_arguments: vec![SNT::Address],
            }),
        },
        SNT::Vector(Box::new(SNT::U16)),
        SNT::Vector(Box::new(SNT::Vector(Box::new(SNT::U8)))),
        SNT::TypeParameter(0),
        SNT::Reference(Box::new(SNT::U8)),
        SNT::MutableReference(Box::new(SNT::Struct {
            inner: Box::new(SM::RtdMoveNormalizedStructType {
                address: RTD_FRAMEWORK_ADDRESS.to_string(),
                module: "coin".to_owned(),
                name: "Coin".to_owned(),
                type_arguments: vec![SNT::Address],
            }),
        })),
    ];

    let mut acc = vec![];

    for value in test_types {
        let json = serde_json::to_string(&value).unwrap();
        acc.push(json);
    }

    let s = SM::RtdMoveNormalizedStruct {
        abilities: SM::RtdMoveAbilitySet {
            abilities: vec![SM::RtdMoveAbility::Copy],
        },
        type_parameters: vec![SM::RtdMoveStructTypeParameter {
            constraints: SM::RtdMoveAbilitySet {
                abilities: vec![SM::RtdMoveAbility::Drop],
            },
            is_phantom: false,
        }],
        fields: vec![
            SM::RtdMoveNormalizedField {
                name: "field1".to_string(),
                type_: SNT::U8,
            },
            SM::RtdMoveNormalizedField {
                name: "field2".to_string(),
                type_: SNT::U16,
            },
        ],
    };

    let json = serde_json::to_string(&s).unwrap();
    acc.push(json);

    // NB: variants declaration and lexicographic ordering are different here
    let variants = vec![
        ("b", vec![SNT::U16]),
        ("a", vec![]),
        (
            "c",
            vec![
                SNT::U32,
                SNT::Struct {
                    inner: Box::new(SM::RtdMoveNormalizedStructType {
                        address: RTD_FRAMEWORK_ADDRESS.to_string(),
                        module: "coin".to_owned(),
                        name: "Coin".to_owned(),
                        type_arguments: vec![SNT::Address],
                    }),
                },
            ],
        ),
    ];
    let variant_declaration_order = variants
        .iter()
        .map(|(name, _)| name.to_string())
        .collect::<Vec<_>>();
    let variants = variants
        .into_iter()
        .map(|(name, type_)| {
            (
                name.to_string(),
                type_
                    .into_iter()
                    .enumerate()
                    .map(|(i, t)| SM::RtdMoveNormalizedField {
                        name: format!("field{}", i),
                        type_: t,
                    })
                    .collect(),
            )
        })
        .collect();

    let e = SM::RtdMoveNormalizedEnum {
        abilities: SM::RtdMoveAbilitySet {
            abilities: vec![SM::RtdMoveAbility::Copy],
        },
        type_parameters: vec![],
        variants,
        variant_declaration_order: Some(variant_declaration_order),
    };

    acc.push(serde_json::to_string(&e).unwrap());

    insta::assert_snapshot!(acc.join("\n"));
}

#[test]
fn test_serde_bytearray() {
    // ensure that we serialize byte arrays as number array
    let test_values = MoveValue::Vector(vec![MoveValue::U8(1), MoveValue::U8(2), MoveValue::U8(3)]);
    let rtd_move_value = RtdMoveValue::from(test_values);
    let json = serde_json::to_value(&rtd_move_value).unwrap();
    assert_eq!(json, json!([1, 2, 3]));
}

#[test]
fn test_serde_number() {
    // ensure that we serialize byte arrays as number array
    let test_values = MoveValue::U8(1);
    let rtd_move_value = RtdMoveValue::from(test_values);
    let json = serde_json::to_value(&rtd_move_value).unwrap();
    assert_eq!(json, json!(1));
    let test_values = MoveValue::U16(1);
    let rtd_move_value = RtdMoveValue::from(test_values);
    let json = serde_json::to_value(&rtd_move_value).unwrap();
    assert_eq!(json, json!(1));
    let test_values = MoveValue::U32(1);
    let rtd_move_value = RtdMoveValue::from(test_values);
    let json = serde_json::to_value(&rtd_move_value).unwrap();
    assert_eq!(json, json!(1));
}

#[test]
fn test_type_tag_struct_tag_devnet_inc_222() {
    let offending_tags = [
        "0x1::address::MyType",
        "0x1::vector::MyType",
        "0x1::address::MyType<0x1::address::OtherType>",
        "0x1::address::MyType<0x1::address::OtherType, 0x1::vector::VecTyper>",
        "0x1::address::address<0x1::vector::address, 0x1::vector::vector>",
    ];

    for tag in offending_tags {
        let oc = ObjectChange::Created {
            sender: Default::default(),
            owner: Owner::Immutable,
            object_type: parse_rtd_struct_tag(tag).unwrap(),
            object_id: ObjectID::random(),
            version: Default::default(),
            digest: ObjectDigest::random(),
        };

        let serde_json = serde_json::to_string(&oc).unwrap();
        let deser: ObjectChange = serde_json::from_str(&serde_json).unwrap();
        assert_eq!(oc, deser);
    }
}
