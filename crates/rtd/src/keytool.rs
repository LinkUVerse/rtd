// Copyright (c) LinkU Labs, Inc.
// SPDX-License-Identifier: Apache-2.0
use crate::zklogin_commands_util::perform_zk_login_test_tx;
use anyhow::anyhow;
use bip32::DerivationPath;
use clap::*;
use fastcrypto::ed25519::Ed25519KeyPair;
use fastcrypto::encoding::{Base64, Encoding, Hex};
use fastcrypto::hash::HashFunction;
use fastcrypto::jwt_utils::parse_and_validate_jwt;
use fastcrypto::traits::{KeyPair, ToFromBytes};
use fastcrypto_zkp::bn254::utils::{
    gen_address_seed, get_nonce, get_proof, get_test_issuer_jwt_token,
};
use fastcrypto_zkp::bn254::zk_login::{JWK, JwkId};
use fastcrypto_zkp::bn254::zk_login::{OIDCProvider, ZkLoginInputs, fetch_jwks};
use fastcrypto_zkp::bn254::zk_login_api::ZkLoginEnv;
use imbl::hashmap::HashMap as ImHashMap;
use json_to_table::{Orientation, json_to_table};
use linku_common::ZipDebugEqIteratorExt;
use num_bigint::BigUint;
use rand::SeedableRng;
use rand::rngs::StdRng;
use rtd_keys::key_derive::generate_new_key;
use rtd_keys::key_identity::KeyIdentity;
use rtd_keys::keypair_file::{
    read_authority_keypair_from_file, read_keypair_from_file, write_authority_keypair_to_file,
    write_keypair_to_file,
};
use rtd_keys::keystore::{AccountKeystore, Keystore};
use rtd_sdk::wallet_context::WalletContext;
use rtd_types::base_types::RtdAddress;
use rtd_types::committee::EpochId;
use rtd_types::crypto::{DefaultHash, PublicKey};
use rtd_types::crypto::{
    EncodeDecodeBase64, RtdKeyPair, Signature, SignatureScheme, ZkLoginPublicIdentifier,
    get_authority_key_pair,
};
use rtd_types::error::RtdResult;
use rtd_types::multisig::{MultiSig, MultiSigPublicKey, ThresholdUnit, WeightUnit};
use rtd_types::multisig_legacy::{MultiSigLegacy, MultiSigPublicKeyLegacy};
use rtd_types::signature::{GenericSignature, VerifyParams};
use rtd_types::signature_verification::VerifiedDigestCache;
use rtd_types::transaction::{TransactionData, TransactionDataAPI};
use rtd_types::zk_login_authenticator::ZkLoginAuthenticator;
use serde::Serialize;
use serde_json::json;
use shared_crypto::intent::{Intent, IntentMessage, IntentScope, PersonalMessage};
use std::fmt::{Debug, Display, Formatter};
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tabled::builder::Builder;
use tabled::settings::Rotate;
use tabled::settings::{Modify, Width, object::Rows};
use tracing::info;
#[cfg(test)]
#[path = "unit_tests/keytool_tests.rs"]
mod keytool_tests;

#[allow(clippy::large_enum_variant)]
#[derive(Subcommand)]
#[clap(rename_all = "kebab-case")]
pub enum KeyToolCommand {
    /// Update an old alias to a new one.
    /// If a new alias is not provided, a random one will be generated.
    #[clap(name = "update-alias")]
    Alias {
        old_alias: String,
        /// The alias must start with a letter and can contain only letters, digits, dots, hyphens (-), or underscores (_).
        new_alias: Option<String>,
    },
    /// Convert private key in Hex or Base64 to new format (Bech32
    /// encoded 33 byte flag || private key starting with "rtdprivkey").
    /// Hex private key format import and export are both deprecated in
    /// Rtd Wallet and Rtd CLI Keystore. Use `rtd keytool import` if you
    /// wish to import a key to Rtd Keystore.
    Convert { value: String },
    /// Given a Base64 encoded transaction bytes, decode its components. If a signature is provided,
    /// verify the signature against the transaction and output the result.
    DecodeOrVerifyTx {
        #[clap(long)]
        tx_bytes: String,
        #[clap(long)]
        sig: Option<GenericSignature>,
        #[clap(long, default_value = "0")]
        cur_epoch: u64,
    },
    /// Given a Base64 encoded MultiSig signature, decode its components.
    /// If tx_bytes is passed in, verify the multisig.
    DecodeMultiSig {
        #[clap(long)]
        multisig: MultiSig,
        #[clap(long)]
        tx_bytes: Option<String>,
        #[clap(long, default_value = "0")]
        cur_epoch: u64,
    },
    /// Generate a new keypair with key scheme flag {ed25519 | secp256k1 | secp256r1}
    /// with optional derivation path, default to m/44'/784'/0'/0'/0' for ed25519 or
    /// m/54'/784'/0'/0/0 for secp256k1 or m/74'/784'/0'/0/0 for secp256r1. Word
    /// length can be { word12 | word15 | word18 | word21 | word24} default to word12
    /// if not specified.
    ///
    /// The keypair file is output to the current directory. The content of the file is
    /// a Base64 encoded string of 33-byte `flag || privkey`.
    ///
    /// Use `rtd client new-address` if you want to generate and save the key into rtd.keystore.
    Generate {
        key_scheme: SignatureScheme,
        derivation_path: Option<DerivationPath>,
        word_length: Option<String>,
    },

    /// Add a new key to Rtd CLI Keystore using either the input mnemonic phrase or a Bech32 encoded 33-byte
    /// `flag || privkey` starting with "rtdprivkey", the key scheme flag {ed25519 | secp256k1 | secp256r1}
    /// and an optional derivation path, default to m/44'/784'/0'/0'/0' for ed25519 or m/54'/784'/0'/0/0
    /// for secp256k1 or m/74'/784'/0'/0/0 for secp256r1. Supports mnemonic phrase of word length 12, 15,
    /// 18, 21, 24. Set an alias for the key with the --alias flag. If no alias is provided, the tool will
    /// automatically generate one.
    Import {
        /// Sets an alias for this address. The alias must start with a letter and can contain only letters, digits, hyphens (-), or underscores (_).
        #[clap(long)]
        alias: Option<String>,
        input_string: String,
        key_scheme: SignatureScheme,
        derivation_path: Option<DerivationPath>,
    },
    /// Output the private key of the given key identity in Rtd CLI Keystore as Bech32
    /// encoded string starting with `rtdprivkey`.
    Export {
        #[clap(long)]
        key_identity: KeyIdentity,
    },
    /// List all keys by its Rtd address, Base64 encoded public key, key scheme name in
    /// rtd.keystore.
    List {
        /// Sort by alias
        #[clap(long, short = 's')]
        sort_by_alias: bool,
    },
    /// This reads the content at the provided file path. The accepted format can be
    /// [enum RtdKeyPair] (Base64 encoded of 33-byte `flag || privkey`) or `type AuthorityKeyPair`
    /// (Base64 encoded `privkey`). This prints out the account keypair as Base64 encoded `flag || privkey`,
    /// the network keypair, worker keypair, protocol keypair as Base64 encoded `privkey`.
    LoadKeypair { file: PathBuf },
    /// To MultiSig Rtd Address. Pass in a list of all public keys `flag || pk` in Base64.
    /// See `keytool list` for example public keys.
    MultiSigAddress {
        #[clap(long)]
        threshold: ThresholdUnit,
        #[clap(long, num_args(1..))]
        pks: Vec<PublicKey>,
        #[clap(long, num_args(1..))]
        weights: Vec<WeightUnit>,
    },
    /// Provides a list of participating signatures (`flag || sig || pk` encoded in Base64),
    /// threshold, a list of all public keys and a list of their weights that define the
    /// MultiSig address. Returns a valid MultiSig signature and its sender address. The
    /// result can be used as signature field for `rtd client execute-signed-tx`. The sum
    /// of weights of all signatures must be >= the threshold.
    ///
    /// The order of `sigs` must be the same as the order of `pks`.
    /// e.g. for [pk1, pk2, pk3, pk4, pk5], [sig1, sig2, sig5] is valid, but
    /// [sig2, sig1, sig5] is invalid.
    MultiSigCombinePartialSig {
        #[clap(long, num_args(1..))]
        sigs: Vec<GenericSignature>,
        #[clap(long, num_args(1..))]
        pks: Vec<PublicKey>,
        #[clap(long, num_args(1..))]
        weights: Vec<WeightUnit>,
        #[clap(long)]
        threshold: ThresholdUnit,
    },
    MultiSigCombinePartialSigLegacy {
        #[clap(long, num_args(1..))]
        sigs: Vec<GenericSignature>,
        #[clap(long, num_args(1..))]
        pks: Vec<PublicKey>,
        #[clap(long, num_args(1..))]
        weights: Vec<WeightUnit>,
        #[clap(long)]
        threshold: ThresholdUnit,
    },

    /// Read the content at the provided file path. The accepted format can be
    /// [enum RtdKeyPair] (Base64 encoded of 33-byte `flag || privkey`) or `type AuthorityKeyPair`
    /// (Base64 encoded `privkey`). It prints its Base64 encoded public key and the key scheme flag.
    Show { file: PathBuf },
    /// Create signature using the private key for the given address (or its alias) in rtd keystore.
    /// Any signature commits to a [struct IntentMessage] consisting of the Base64 encoded
    /// of the BCS serialized transaction bytes itself and its intent. If intent is absent,
    /// default will be used.
    Sign {
        #[clap(long)]
        address: KeyIdentity,
        #[clap(long)]
        data: String,
        #[clap(long)]
        intent: Option<Intent>,
    },
    /// This takes [enum RtdKeyPair] of Base64 encoded of 33-byte `flag || privkey`). It
    /// outputs the keypair into a file at the current directory where the address is the filename,
    /// and prints out its Rtd address, Base64 encoded public key, the key scheme, and the key scheme flag.
    Unpack { keypair: String },

    /// Disabled until RTD OAuth clients, redirect URLs, and zkLogin services have been configured.
    ZkLoginSignAndExecuteTx {
        #[clap(long)]
        max_epoch: EpochId,
        #[clap(long, default_value = "devnet")]
        network: String,
        #[clap(long, default_value = "false")]
        fixed: bool, // if true, use a fixed kp generated from [0; 32] seed.
        #[clap(long, default_value = "false")]
        test_multisig: bool, // if true, use a multisig address with zklogin and a traditional kp.
        #[clap(long, default_value = "false")]
        sign_with_sk: bool, // if true, execute tx with the traditional sig (in the multisig), otherwise with the zklogin sig.
    },

    /// Execute a zkLogin test transaction with a token and parameters supplied by a separately configured RTD client.
    ZkLoginEnterToken {
        #[clap(long)]
        parsed_token: String,
        #[clap(long)]
        max_epoch: EpochId,
        #[clap(long)]
        jwt_randomness: String,
        #[clap(long)]
        kp_bigint: String,
        #[clap(long)]
        ephemeral_key_identifier: RtdAddress,
        #[clap(long, default_value = "devnet")]
        network: String,
        #[clap(long, default_value = "false")]
        test_multisig: bool,
        #[clap(long, default_value = "false")]
        sign_with_sk: bool,
    },

    /// Given a zkLogin signature, parse it if valid. If `bytes` provided,
    /// parse it as either as TransactionData or PersonalMessage based on `intent_scope`.
    /// It verifies the zkLogin signature based its latest JWK fetched.
    /// Example request: rtd keytool zk-login-sig-verify --sig $SERIALIZED_ZKLOGIN_SIG --bytes $BYTES --intent-scope 0 --network devnet --curr-epoch 10
    ZkLoginSigVerify {
        /// The Base64 of the serialized zkLogin signature.
        #[clap(long)]
        sig: String,
        /// The Base64 of the BCS encoded TransactionData or PersonalMessage.
        #[clap(long)]
        bytes: Option<String>,
        /// Either 0 for TransactionData or 3 for PersonalMessage.
        #[clap(long)]
        intent_scope: u8,
        /// The current epoch for the network to verify the signature's max_epoch against.
        #[clap(long)]
        cur_epoch: Option<EpochId>,
        /// The network to verify the signature for, determines ZkLoginEnv.
        #[clap(long, default_value = "devnet")]
        network: String,
    },

    /// TESTING ONLY: Generate a fixed ephemeral key and its JWT token with test issuer. Produce a zklogin signature for the given data and max epoch.
    /// e.g. rtd keytool zk-login-insecure-sign-personal-message --data "hello" --max-epoch 5
    ZkLoginInsecureSignPersonalMessage {
        /// The base64 encoded string of the message to sign, without the intent message wrapping.
        #[clap(long)]
        data: String,
        /// The max epoch used for the zklogin signature validity.
        #[clap(long)]
        max_epoch: EpochId,
    },
}

// Command Output types
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AliasUpdate {
    old_alias: String,
    new_alias: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DecodedMultiSig {
    public_base64_key: String,
    sig_base64: String,
    weight: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DecodedMultiSigOutput {
    multisig_address: RtdAddress,
    participating_keys_signatures: Vec<DecodedMultiSig>,
    pub_keys: Vec<MultiSigOutput>,
    threshold: usize,
    sig_verify_result: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DecodeOrVerifyTxOutput {
    tx: TransactionData,
    result: Option<RtdResult>,
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Key {
    alias: Option<String>,
    rtd_address: RtdAddress,
    public_base64_key: String,
    key_scheme: String,
    flag: u8,
    #[serde(skip_serializing_if = "Option::is_none")]
    mnemonic: Option<String>,
    peer_id: Option<String>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ExportedKey {
    exported_private_key: String,
    key: Key,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct KeypairData {
    account_keypair: String,
    network_keypair: Option<String>,
    worker_keypair: Option<String>,
    key_scheme: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MultiSigAddress {
    multisig_address: String,
    multisig: Vec<MultiSigOutput>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MultiSigCombinePartialSig {
    multisig_address: RtdAddress,
    multisig_parsed: GenericSignature,
    multisig_serialized: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MultiSigCombinePartialSigLegacyOutput {
    multisig_address: RtdAddress,
    multisig_legacy_parsed: GenericSignature,
    multisig_legacy_serialized: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MultiSigOutput {
    address: RtdAddress,
    public_base64_key: String,
    weight: u8,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ConvertOutput {
    bech32_with_flag: String, // latest Rtd Keystore and Rtd Wallet import/export format
    base64_with_flag: String, // Rtd Keystore storage format
    hex_without_flag: String, // Legacy Rtd Wallet format
    scheme: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PrivateKeyBase64 {
    base64: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SignData {
    rtd_address: RtdAddress,
    // Base64 encoded string of serialized transaction data.
    raw_tx_data: String,
    // Intent struct used, see [struct Intent] for field definitions.
    intent: Intent,
    // Base64 encoded [struct IntentMessage] consisting of (intent || message)
    // where message can be `TransactionData` etc.
    raw_intent_msg: String,
    // Base64 encoded blake2b hash of the intent message, this is what the signature commits to.
    digest: String,
    // Base64 encoded `flag || signature || pubkey` for a complete
    // serialized Rtd signature to be send for executing the transaction.
    rtd_signature: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ZkLoginSignAndExecuteTx {
    tx_digest: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ZkLoginSigVerifyResponse {
    data: Option<String>,
    iss: String,
    address_seed: String,
    kid: String,
    parsed: String,
    jwks: Option<String>,
    res: Option<RtdResult>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ZkLoginInsecureSignPersonalMessage {
    sig: String,
    bytes: String,
    address: String,
}

#[derive(Serialize)]
#[serde(untagged)]
pub enum CommandOutput {
    Alias(AliasUpdate),
    Convert(ConvertOutput),
    DecodeMultiSig(DecodedMultiSigOutput),
    DecodeOrVerifyTx(DecodeOrVerifyTxOutput),
    Error(String),
    Generate(Key),
    Import(Key),
    Export(ExportedKey),
    List(Vec<Key>),
    LoadKeypair(KeypairData),
    MultiSigAddress(MultiSigAddress),
    MultiSigCombinePartialSig(MultiSigCombinePartialSig),
    MultiSigCombinePartialSigLegacy(MultiSigCombinePartialSigLegacyOutput),
    PrivateKeyBase64(PrivateKeyBase64),
    Show(Key),
    Sign(SignData),
    ZkLoginSignAndExecuteTx(ZkLoginSignAndExecuteTx),
    ZkLoginInsecureSignPersonalMessage(ZkLoginInsecureSignPersonalMessage),
    ZkLoginSigVerify(ZkLoginSigVerifyResponse),
}

impl KeyToolCommand {
    pub async fn execute(
        self,
        context: &mut WalletContext,
    ) -> Result<CommandOutput, anyhow::Error> {
        Ok(match self {
            KeyToolCommand::Alias {
                old_alias,
                new_alias,
            } => {
                let keystore: &mut Keystore =
                    context.get_keystore_by_identity_mut(&KeyIdentity::Alias(old_alias.clone()))?;
                let new_alias = keystore
                    .update_alias(&old_alias, new_alias.as_deref())
                    .await?;
                CommandOutput::Alias(AliasUpdate {
                    old_alias,
                    new_alias,
                })
            }
            KeyToolCommand::Convert { value } => {
                let result = convert_private_key_to_bech32(value)?;
                CommandOutput::Convert(result)
            }

            KeyToolCommand::DecodeMultiSig {
                multisig,
                tx_bytes,
                cur_epoch,
            } => {
                let pks = multisig.get_pk().pubkeys();
                let sigs = multisig.get_sigs();
                let bitmap = multisig.get_indices()?;
                let address = RtdAddress::from(multisig.get_pk());

                let pub_keys = pks
                    .iter()
                    .map(|(pk, w)| MultiSigOutput {
                        address: (pk).into(),
                        public_base64_key: pk.encode_base64(),
                        weight: *w,
                    })
                    .collect::<Vec<MultiSigOutput>>();

                let threshold = *multisig.get_pk().threshold() as usize;

                let mut output = DecodedMultiSigOutput {
                    multisig_address: address,
                    participating_keys_signatures: vec![],
                    pub_keys,
                    threshold,
                    sig_verify_result: "".to_string(),
                };

                for (sig, i) in sigs.iter().zip_debug_eq(bitmap) {
                    let (pk, w) = pks
                        .get(i as usize)
                        .ok_or(anyhow!("Invalid public keys index".to_string()))?;
                    output.participating_keys_signatures.push(DecodedMultiSig {
                        public_base64_key: pk.encode_base64().clone(),
                        sig_base64: Base64::encode(sig.as_ref()),
                        weight: w.to_string(),
                    })
                }

                if let Some(tx_bytes) = tx_bytes {
                    let tx_bytes = Base64::decode(&tx_bytes)
                        .map_err(|e| anyhow!("Invalid base64 tx bytes: {:?}", e))?;
                    let tx_data: TransactionData = bcs::from_bytes(&tx_bytes)?;
                    let s = GenericSignature::MultiSig(multisig);
                    let res = s.verify_authenticator(
                        &IntentMessage::new(Intent::rtd_transaction(), tx_data),
                        address,
                        cur_epoch,
                        &VerifyParams::default(),
                        Arc::new(VerifiedDigestCache::new_empty()),
                    );

                    match res {
                        Ok(()) => output.sig_verify_result = "OK".to_string(),
                        Err(e) => output.sig_verify_result = format!("{:?}", e),
                    };
                };

                CommandOutput::DecodeMultiSig(output)
            }

            KeyToolCommand::DecodeOrVerifyTx {
                tx_bytes,
                sig,
                cur_epoch,
            } => {
                let tx_bytes = Base64::decode(&tx_bytes)
                    .map_err(|e| anyhow!("Invalid base64 key: {:?}", e))?;
                let tx_data: TransactionData = bcs::from_bytes(&tx_bytes)?;
                match sig {
                    None => CommandOutput::DecodeOrVerifyTx(DecodeOrVerifyTxOutput {
                        tx: tx_data,
                        result: None,
                    }),
                    Some(s) => {
                        let res = s.verify_authenticator(
                            &IntentMessage::new(Intent::rtd_transaction(), tx_data.clone()),
                            tx_data.sender(),
                            cur_epoch,
                            &VerifyParams::default(),
                            Arc::new(VerifiedDigestCache::new_empty()),
                        );
                        CommandOutput::DecodeOrVerifyTx(DecodeOrVerifyTxOutput {
                            tx: tx_data,
                            result: Some(res),
                        })
                    }
                }
            }
            KeyToolCommand::Generate {
                key_scheme,
                derivation_path,
                word_length,
            } => match key_scheme {
                SignatureScheme::BLS12381 => {
                    let (rtd_address, kp) = get_authority_key_pair();
                    let file_name = format!("bls-{rtd_address}.key");
                    write_authority_keypair_to_file(&kp, file_name)?;
                    CommandOutput::Generate(Key {
                        alias: None,
                        rtd_address,
                        public_base64_key: kp.public().encode_base64(),
                        key_scheme: key_scheme.to_string(),
                        flag: SignatureScheme::BLS12381.flag(),
                        mnemonic: None,
                        peer_id: None,
                    })
                }
                _ => {
                    let (rtd_address, skp, _scheme, phrase) =
                        generate_new_key(key_scheme, derivation_path, word_length)?;
                    let file = format!("{rtd_address}.key");
                    write_keypair_to_file(&skp, file)?;
                    let mut key = Key::from(&skp);
                    key.mnemonic = Some(phrase);
                    CommandOutput::Generate(key)
                }
            },

            KeyToolCommand::Import {
                alias,
                input_string,
                key_scheme,
                derivation_path,
            } => {
                if Hex::decode(&input_string).is_ok() {
                    return Err(anyhow!(
                        "Rtd Keystore and Rtd Wallet no longer support importing
                    private key as Hex, if you are sure your private key is encoded in Hex, use
                    `rtd keytool convert $HEX` to convert first then import the Bech32 encoded
                    private key starting with `rtdprivkey`."
                    ));
                }

                match RtdKeyPair::decode(&input_string) {
                    Ok(skp) => {
                        info!("Importing Bech32 encoded private key to keystore");
                        let mut key = Key::from(&skp);
                        context.config.keystore.import(alias.clone(), skp).await?;

                        let alias = match alias {
                            Some(x) => x,
                            None => context.config.keystore.get_alias(&key.rtd_address)?,
                        };

                        key.alias = Some(alias);
                        CommandOutput::Import(key)
                    }
                    Err(_) => {
                        info!("Importing mneomonics to keystore");
                        let rtd_address = context
                            .config
                            .keystore
                            .import_from_mnemonic(
                                &input_string,
                                key_scheme,
                                derivation_path,
                                alias.clone(),
                            )
                            .await?;
                        let skp = context.config.keystore.export(&rtd_address)?;
                        let mut key = Key::from(skp);

                        let alias = match alias {
                            Some(x) => x,
                            None => context.config.keystore.get_alias(&key.rtd_address)?,
                        };

                        key.alias = Some(alias);
                        CommandOutput::Import(key)
                    }
                }
            }
            KeyToolCommand::Export { key_identity } => {
                let address = context.config.keystore.get_by_identity(&key_identity)?;
                let skp = context.config.keystore.export(&address)?;
                let mut key = Key::from(skp);
                key.alias = context.config.keystore.get_alias(&key.rtd_address).ok();
                let key = ExportedKey {
                    exported_private_key: skp
                        .encode()
                        .map_err(|_| anyhow!("Cannot decode keypair"))?,
                    key,
                };
                CommandOutput::Export(key)
            }
            KeyToolCommand::List { sort_by_alias } => {
                let external_keys = context
                    .config
                    .external_keys
                    .as_ref()
                    .map(|k| k.entries())
                    .unwrap_or_default()
                    .into_iter();

                let mut keys: Vec<Key> = context
                    .config
                    .keystore
                    .entries()
                    .into_iter()
                    .chain(external_keys)
                    .map(|pk| {
                        let mut key = Key::from(pk);
                        key.alias = context.config.keystore.get_alias(&key.rtd_address).ok();
                        key
                    })
                    .collect::<Vec<Key>>();
                if sort_by_alias {
                    keys.sort_unstable();
                }
                CommandOutput::List(keys)
            }

            KeyToolCommand::LoadKeypair { file } => {
                let output = match read_keypair_from_file(&file) {
                    Ok(keypair) => {
                        // Account keypair is encoded with the key scheme flag {},
                        // and network and worker keypair are not.
                        let network_worker_keypair = match &keypair {
                            RtdKeyPair::Ed25519(kp) => kp.encode_base64(),
                            RtdKeyPair::Secp256k1(kp) => kp.encode_base64(),
                            RtdKeyPair::Secp256r1(kp) => kp.encode_base64(),
                        };
                        KeypairData {
                            account_keypair: keypair.encode_base64(),
                            network_keypair: Some(network_worker_keypair.clone()),
                            worker_keypair: Some(network_worker_keypair),
                            key_scheme: keypair.public().scheme().to_string(),
                        }
                    }
                    Err(_) => {
                        // Authority keypair file is not stored with the flag, it will try read as BLS keypair..
                        match read_authority_keypair_from_file(&file) {
                            Ok(keypair) => KeypairData {
                                account_keypair: keypair.encode_base64(),
                                network_keypair: None,
                                worker_keypair: None,
                                key_scheme: SignatureScheme::BLS12381.to_string(),
                            },
                            Err(e) => {
                                return Err(anyhow!(format!(
                                    "Failed to read keypair at path {:?} err: {:?}",
                                    file, e
                                )));
                            }
                        }
                    }
                };
                CommandOutput::LoadKeypair(output)
            }

            KeyToolCommand::MultiSigAddress {
                threshold,
                pks,
                weights,
            } => {
                let multisig_pk = MultiSigPublicKey::new(pks.clone(), weights.clone(), threshold)?;
                let address: RtdAddress = (&multisig_pk).into();
                let mut output = MultiSigAddress {
                    multisig_address: address.to_string(),
                    multisig: vec![],
                };

                for (pk, w) in pks.into_iter().zip_debug_eq(weights) {
                    output.multisig.push(MultiSigOutput {
                        address: Into::<RtdAddress>::into(&pk),
                        public_base64_key: pk.encode_base64(),
                        weight: w,
                    });
                }
                CommandOutput::MultiSigAddress(output)
            }

            KeyToolCommand::MultiSigCombinePartialSig {
                sigs,
                pks,
                weights,
                threshold,
            } => {
                let multisig_pk = MultiSigPublicKey::new(pks, weights, threshold)?;
                let address: RtdAddress = (&multisig_pk).into();
                let multisig = MultiSig::combine(sigs, multisig_pk)?;
                let generic_sig: GenericSignature = multisig.into();
                let multisig_serialized = generic_sig.encode_base64();
                CommandOutput::MultiSigCombinePartialSig(MultiSigCombinePartialSig {
                    multisig_address: address,
                    multisig_parsed: generic_sig,
                    multisig_serialized,
                })
            }

            KeyToolCommand::MultiSigCombinePartialSigLegacy {
                sigs,
                pks,
                weights,
                threshold,
            } => {
                let multisig_pk_legacy =
                    MultiSigPublicKeyLegacy::new(pks.clone(), weights.clone(), threshold)?;
                let multisig_pk = MultiSigPublicKey::new(pks, weights, threshold)?;
                let address: RtdAddress = (&multisig_pk).into();
                let multisig = MultiSigLegacy::combine(sigs, multisig_pk_legacy)?;
                let generic_sig: GenericSignature = multisig.into();
                let multisig_legacy_serialized = generic_sig.encode_base64();

                CommandOutput::MultiSigCombinePartialSigLegacy(
                    MultiSigCombinePartialSigLegacyOutput {
                        multisig_address: address,
                        multisig_legacy_parsed: generic_sig,
                        multisig_legacy_serialized,
                    },
                )
            }

            KeyToolCommand::Show { file } => {
                let res = read_keypair_from_file(&file);
                match res {
                    Ok(skp) => {
                        let key = Key::from(&skp);
                        CommandOutput::Show(key)
                    }
                    Err(_) => match read_authority_keypair_from_file(&file) {
                        Ok(keypair) => {
                            let public_base64_key = keypair.public().encode_base64();
                            CommandOutput::Show(Key {
                                alias: None, // alias does not get stored in key files
                                rtd_address: (keypair.public()).into(),
                                public_base64_key,
                                key_scheme: SignatureScheme::BLS12381.to_string(),
                                flag: SignatureScheme::BLS12381.flag(),
                                peer_id: None,
                                mnemonic: None,
                            })
                        }
                        Err(e) => CommandOutput::Error(format!(
                            "Failed to read keypair at path {:?}, err: {e}",
                            file
                        )),
                    },
                }
            }

            KeyToolCommand::Sign {
                address,
                data,
                intent,
            } => {
                let address = context.get_identity_address(Some(address))?;
                let intent = intent.unwrap_or_else(Intent::rtd_transaction);
                let intent_clone = intent.clone();
                let msg: TransactionData =
                    bcs::from_bytes(&Base64::decode(&data).map_err(|e| {
                        anyhow!("Cannot deserialize data as TransactionData {:?}", e)
                    })?)?;
                let intent_msg = IntentMessage::new(intent, msg);
                let raw_intent_msg: String = Base64::encode(bcs::to_bytes(&intent_msg)?);
                let mut hasher = DefaultHash::default();
                bcs::serialize_into(&mut hasher, &intent_msg)?;
                let digest = hasher.finalize().digest;
                let rtd_signature = context
                    .sign_secure(&address.into(), &intent_msg.value, intent_msg.intent)
                    .await?;
                CommandOutput::Sign(SignData {
                    rtd_address: address,
                    raw_tx_data: data,
                    intent: intent_clone,
                    raw_intent_msg,
                    digest: Base64::encode(digest),
                    rtd_signature: rtd_signature.encode_base64(),
                })
            }

            KeyToolCommand::Unpack { keypair } => {
                let keypair = RtdKeyPair::decode_base64(&keypair)
                    .map_err(|_| anyhow!("Invalid Base64 encode keypair"))?;

                let key = Key::from(&keypair);
                let path_str = format!("{}.key", key.rtd_address).to_lowercase();
                let path = Path::new(&path_str);
                let out_str = format!(
                    "address: {}\nkeypair: {}\nflag: {}",
                    key.rtd_address,
                    keypair.encode_base64(),
                    key.flag
                );
                fs::write(path, out_str).unwrap();
                CommandOutput::Show(key)
            }

            KeyToolCommand::ZkLoginInsecureSignPersonalMessage { data, max_epoch } => {
                let msg = PersonalMessage {
                    message: data.as_bytes().to_vec(),
                };
                let sub = "1";
                let user_salt = "1";
                let intent_msg = IntentMessage::new(Intent::personal_message(), msg.clone());

                // set up keypair, nonce with max_epoch
                let skp =
                    RtdKeyPair::Ed25519(Ed25519KeyPair::generate(&mut StdRng::from_seed([0; 32])));
                let jwt_randomness = BigUint::from_bytes_be(&[0; 32]).to_string();
                let mut eph_pk_bytes = vec![0x00];
                eph_pk_bytes.extend(skp.public().as_ref());
                let kp_bigint = BigUint::from_bytes_be(&eph_pk_bytes).to_string();
                let nonce = get_nonce(&eph_pk_bytes, max_epoch, &jwt_randomness).unwrap();

                // call test issuer to get jwt token.
                let client = reqwest::Client::new();
                let parsed_token = get_test_issuer_jwt_token(
                    &client,
                    &nonce,
                    &OIDCProvider::TestIssuer.get_config().iss,
                    sub,
                )
                .await
                .unwrap()
                .jwt;

                // Use a prover that was deployed and verified for this RTD network.
                let prover_url = std::env::var("RTD_ZKLOGIN_PROVER_URL")
                    .map_err(|_| anyhow!("Set RTD_ZKLOGIN_PROVER_URL before signing"))?;
                let reader = get_proof(
                    &parsed_token,
                    max_epoch,
                    &jwt_randomness,
                    &kp_bigint,
                    user_salt,
                    &prover_url,
                )
                .await
                .unwrap();
                let (_, aud, _) = parse_and_validate_jwt(&parsed_token).unwrap();
                let address_seed = gen_address_seed(user_salt, "sub", sub, &aud).unwrap();
                let zk_login_inputs =
                    ZkLoginInputs::from_reader(reader, &address_seed.to_string()).unwrap();
                let pk = PublicKey::ZkLogin(
                    ZkLoginPublicIdentifier::new(
                        zk_login_inputs.get_iss(),
                        zk_login_inputs.get_address_seed(),
                    )
                    .unwrap(),
                );
                let address = RtdAddress::from(&pk);
                // sign with ephemeral key and combine with zklogin inputs to generic signature
                let s = Signature::new_secure(&intent_msg, &skp);
                let sig = GenericSignature::ZkLoginAuthenticator(ZkLoginAuthenticator::new(
                    zk_login_inputs,
                    max_epoch,
                    s,
                ));
                CommandOutput::ZkLoginInsecureSignPersonalMessage(
                    ZkLoginInsecureSignPersonalMessage {
                        sig: Base64::encode(sig.as_bytes()),
                        bytes: Base64::encode(data.as_bytes()),
                        address: address.to_string(),
                    },
                )
            }
            KeyToolCommand::ZkLoginSignAndExecuteTx { .. } => {
                return Err(anyhow!(
                    "The built-in OAuth test clients and redirect URLs are not configured for RTD. \
                     Use an independently configured RTD OAuth client and the zk-login-enter-token command."
                ));
            }
            KeyToolCommand::ZkLoginEnterToken {
                parsed_token,
                max_epoch,
                jwt_randomness,
                kp_bigint,
                ephemeral_key_identifier,
                network,
                test_multisig,
                sign_with_sk,
            } => {
                let tx_digest = perform_zk_login_test_tx(
                    &parsed_token,
                    max_epoch,
                    &jwt_randomness,
                    &kp_bigint,
                    ephemeral_key_identifier,
                    &mut context.config.keystore,
                    &network,
                    test_multisig,
                    sign_with_sk,
                )
                .await?;
                CommandOutput::ZkLoginSignAndExecuteTx(ZkLoginSignAndExecuteTx { tx_digest })
            }

            KeyToolCommand::ZkLoginSigVerify {
                sig,
                bytes,
                intent_scope,
                cur_epoch,
                network,
            } => {
                match GenericSignature::from_bytes(
                    &Base64::decode(&sig).map_err(|e| anyhow!("Invalid base64 sig: {:?}", e))?,
                )? {
                    GenericSignature::ZkLoginAuthenticator(zk) => {
                        if bytes.is_none() || cur_epoch.is_none() {
                            return Ok(CommandOutput::ZkLoginSigVerify(ZkLoginSigVerifyResponse {
                                data: None,
                                parsed: serde_json::to_string(&zk)?,
                                iss: zk.inputs.get_iss().to_owned(),
                                kid: zk.inputs.get_kid().to_owned(),
                                address_seed: zk.inputs.get_address_seed().to_string(),
                                res: None,
                                jwks: None,
                            }));
                        }

                        let client = reqwest::Client::new();
                        let provider = OIDCProvider::from_iss(zk.get_iss())
                            .map_err(|_| anyhow!("Invalid iss"))?;
                        let jwks = fetch_jwks(&provider, &client, true).await?;
                        let parsed: ImHashMap<JwkId, JWK> = jwks.clone().into_iter().collect();
                        let env = match network.as_str() {
                            "devnet" | "localnet" => ZkLoginEnv::Test,
                            "mainnet" | "testnet" => ZkLoginEnv::Prod,
                            _ => return Err(anyhow!("Invalid network")),
                        };
                        let verify_params = VerifyParams::new(
                            parsed,
                            vec![],
                            env,
                            1, // migration mode, try v2 then v1
                            true,
                            true,
                            true,
                            Some(2),
                            true,
                            true,
                        );

                        let (serialized, res) = match IntentScope::try_from(intent_scope)
                            .map_err(|_| anyhow!("Invalid scope"))?
                        {
                            IntentScope::TransactionData => {
                                let tx_data: TransactionData = bcs::from_bytes(
                                    &Base64::decode(&bytes.unwrap())
                                        .map_err(|e| anyhow!("Invalid base64 tx data: {:?}", e))?,
                                )?;

                                let sig = GenericSignature::ZkLoginAuthenticator(zk.clone());
                                let res = sig.verify_authenticator(
                                    &IntentMessage::new(Intent::rtd_transaction(), tx_data.clone()),
                                    tx_data.execution_parts().1,
                                    cur_epoch.unwrap(),
                                    &verify_params,
                                    Arc::new(VerifiedDigestCache::new_empty()),
                                );
                                (serde_json::to_string(&tx_data)?, res)
                            }
                            IntentScope::PersonalMessage => {
                                let data = PersonalMessage {
                                    message: Base64::decode(&bytes.unwrap()).map_err(|e| {
                                        anyhow!("Invalid base64 personal message data: {:?}", e)
                                    })?,
                                };

                                let sig = GenericSignature::ZkLoginAuthenticator(zk.clone());
                                let res = sig.verify_authenticator(
                                    &IntentMessage::new(Intent::personal_message(), data.clone()),
                                    (&zk).try_into()?,
                                    cur_epoch.unwrap(),
                                    &verify_params,
                                    Arc::new(VerifiedDigestCache::new_empty()),
                                );
                                (serde_json::to_string(&data)?, res)
                            }
                            _ => return Err(anyhow!("Invalid intent scope")),
                        };
                        CommandOutput::ZkLoginSigVerify(ZkLoginSigVerifyResponse {
                            data: Some(serialized),
                            parsed: serde_json::to_string(&zk)?,
                            iss: zk.inputs.get_iss().to_owned(),
                            kid: zk.inputs.get_kid().to_owned(),
                            address_seed: zk.inputs.get_address_seed().to_string(),
                            jwks: Some(serde_json::to_string(&jwks)?),
                            res: Some(res),
                        })
                    }
                    _ => CommandOutput::Error("Not a zkLogin signature".to_string()),
                }
            }
        })
    }
}

impl From<&RtdKeyPair> for Key {
    fn from(skp: &RtdKeyPair) -> Self {
        Key::from(skp.public())
    }
}

impl From<PublicKey> for Key {
    fn from(pk: PublicKey) -> Self {
        Key {
            alias: None, // this is retrieved later
            rtd_address: RtdAddress::from(&pk),
            public_base64_key: pk.encode_base64(),
            key_scheme: pk.scheme().to_string(),
            mnemonic: None,
            flag: pk.flag(),
            peer_id: anemo_styling(&pk),
        }
    }
}

impl Key {
    pub(crate) fn with_mnemonic(mut self, mnemonic: Option<String>) -> Self {
        self.mnemonic = mnemonic;
        self
    }
}

impl Display for CommandOutput {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            CommandOutput::Alias(update) => {
                write!(
                    formatter,
                    "Old alias {} was updated to {}",
                    update.old_alias, update.new_alias
                )
            }
            // Sign needs to be manually built because we need to wrap the very long
            // rawTxData string and rawIntentMsg strings into multiple rows due to
            // their lengths, which we cannot do with a JsonTable
            CommandOutput::Sign(data) => {
                let intent_table = json_to_table(&json!(&data.intent))
                    .with(tabled::settings::Style::rounded().horizontals([]))
                    .to_string();

                let mut builder = Builder::default();
                builder
                    .set_header([
                        "rtdSignature",
                        "digest",
                        "rawIntentMsg",
                        "intent",
                        "rawTxData",
                        "rtdAddress",
                    ])
                    .push_record([
                        &data.rtd_signature,
                        &data.digest,
                        &data.raw_intent_msg,
                        &intent_table,
                        &data.raw_tx_data,
                        &data.rtd_address.to_string(),
                    ]);
                let mut table = builder.build();
                table.with(Rotate::Left);
                table.with(tabled::settings::Style::rounded().horizontals([]));
                table.with(Modify::new(Rows::new(0..)).with(Width::wrap(160).keep_words()));
                write!(formatter, "{}", table)
            }
            _ => {
                let json_obj = json![self];
                let mut table = json_to_table(&json_obj);
                let style = tabled::settings::Style::rounded().horizontals([]);
                table.with(style);
                table.array_orientation(Orientation::Column);
                write!(formatter, "{}", table)
            }
        }
    }
}

impl CommandOutput {
    pub fn print(&self, pretty: bool) {
        let line = if pretty {
            format!("{self}")
        } else {
            format!("{:?}", self)
        };
        // Log line by line
        for line in line.lines() {
            // Logs write to a file on the side.  Print to stdout and also log to file, for tests to pass.
            println!("{line}");
            info!("{line}")
        }
    }
}

// when --json flag is used, any output result is transformed into a JSON pretty string and sent to std output
impl Debug for CommandOutput {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match serde_json::to_string_pretty(self) {
            Ok(json) => write!(f, "{json}"),
            Err(err) => write!(f, "Error serializing JSON: {err}"),
        }
    }
}

/// Converts legacy formatted private key to 33 bytes bech32 encoded private key or vice versa.
/// It can handle:
/// 1) Hex encoded 32 byte private key (assumes scheme is Ed25519), this is the legacy wallet format
/// 2) Base64 encoded 32 bytes private key (assumes scheme is Ed25519)
/// 3) Base64 encoded 33 bytes private key with flag.
/// 4) Bech32 encoded 33 bytes private key with flag.
fn convert_private_key_to_bech32(value: String) -> Result<ConvertOutput, anyhow::Error> {
    let skp = match RtdKeyPair::decode(&value) {
        Ok(s) => s,
        Err(_) => match Hex::decode(&value) {
            Ok(decoded) => {
                if decoded.len() != 32 {
                    return Err(anyhow!(format!(
                        "Invalid private key length, expected 32 but got {}",
                        decoded.len()
                    )));
                }
                RtdKeyPair::Ed25519(Ed25519KeyPair::from_bytes(&decoded)?)
            }
            Err(_) => match RtdKeyPair::decode_base64(&value) {
                Ok(skp) => skp,
                Err(_) => match Ed25519KeyPair::decode_base64(&value) {
                    Ok(kp) => RtdKeyPair::Ed25519(kp),
                    Err(_) => return Err(anyhow!("Invalid private key encoding")),
                },
            },
        },
    };

    Ok(ConvertOutput {
        bech32_with_flag: skp.encode().map_err(|_| anyhow!("Cannot encode keypair"))?,
        base64_with_flag: skp.encode_base64(),
        hex_without_flag: Hex::encode(&skp.to_bytes()[1..]),
        scheme: skp.public().scheme().to_string(),
    })
}

fn anemo_styling(pk: &PublicKey) -> Option<String> {
    if let PublicKey::Ed25519(public_key) = pk {
        Some(anemo::PeerId(public_key.0).to_string())
    } else {
        None
    }
}
