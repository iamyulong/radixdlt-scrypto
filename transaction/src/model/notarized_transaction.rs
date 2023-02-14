use radix_engine_interface::crypto::*;
use radix_engine_interface::data::{scrypto_decode, scrypto_encode};
use radix_engine_interface::network::NetworkDefinition;
use radix_engine_interface::*;
use sbor::*;

use crate::manifest::{compile, CompileError};
use crate::model::TransactionManifest;

// TODO: add versioning of transaction schema

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, PartialEq, Eq, ScryptoCategorize, ScryptoEncode, ScryptoDecode)]
pub struct TransactionHeader {
    pub version: u8,
    pub network_id: u8,
    pub start_epoch_inclusive: u64,
    pub end_epoch_exclusive: u64,
    pub nonce: u64,
    pub notary_public_key: PublicKey,
    pub notary_as_signatory: bool,
    pub cost_unit_limit: u32,
    pub tip_percentage: u16,
}

#[derive(Debug, Clone, PartialEq, Eq, ScryptoCategorize, ScryptoEncode, ScryptoDecode)]
pub struct TransactionIntent {
    pub header: TransactionHeader,
    pub manifest: TransactionManifest,
}

#[derive(Debug, Clone, PartialEq, Eq, ScryptoCategorize, ScryptoEncode, ScryptoDecode)]
pub struct SignedTransactionIntent {
    pub intent: TransactionIntent,
    pub intent_signatures: Vec<SignatureWithPublicKey>,
}

#[derive(Debug, Clone, PartialEq, Eq, ScryptoCategorize, ScryptoEncode, ScryptoDecode)]
pub struct NotarizedTransaction {
    pub signed_intent: SignedTransactionIntent,
    pub notary_signature: Signature,
}

/// Represents any natively supported signature.
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(tag = "type", content = "signature")
)]
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Hash, ScryptoCategorize, ScryptoEncode, ScryptoDecode,
)]
pub enum Signature {
    EcdsaSecp256k1(EcdsaSecp256k1Signature),
    EddsaEd25519(EddsaEd25519Signature),
}

/// Represents any natively supported signature, including public key.
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(tag = "type")
)]
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Hash, ScryptoCategorize, ScryptoEncode, ScryptoDecode,
)]
pub enum SignatureWithPublicKey {
    EcdsaSecp256k1 {
        signature: EcdsaSecp256k1Signature,
    },
    EddsaEd25519 {
        public_key: EddsaEd25519PublicKey,
        signature: EddsaEd25519Signature,
    },
}

impl SignatureWithPublicKey {
    pub fn signature(&self) -> Signature {
        match &self {
            SignatureWithPublicKey::EcdsaSecp256k1 { signature } => signature.clone().into(),
            SignatureWithPublicKey::EddsaEd25519 { signature, .. } => signature.clone().into(),
        }
    }
}

impl From<EcdsaSecp256k1Signature> for Signature {
    fn from(signature: EcdsaSecp256k1Signature) -> Self {
        Self::EcdsaSecp256k1(signature)
    }
}

impl From<EddsaEd25519Signature> for Signature {
    fn from(signature: EddsaEd25519Signature) -> Self {
        Self::EddsaEd25519(signature)
    }
}

impl From<EcdsaSecp256k1Signature> for SignatureWithPublicKey {
    fn from(signature: EcdsaSecp256k1Signature) -> Self {
        Self::EcdsaSecp256k1 { signature }
    }
}

impl From<(EddsaEd25519PublicKey, EddsaEd25519Signature)> for SignatureWithPublicKey {
    fn from((public_key, signature): (EddsaEd25519PublicKey, EddsaEd25519Signature)) -> Self {
        Self::EddsaEd25519 {
            public_key,
            signature,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum IntentCreationError {
    CompileErr(CompileError),
    ConfigErr(IntentConfigError),
}

impl From<CompileError> for IntentCreationError {
    fn from(compile_error: CompileError) -> Self {
        IntentCreationError::CompileErr(compile_error)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum IntentConfigError {
    MismatchedNetwork { expected: u8, actual: u8 },
}

impl TransactionIntent {
    pub fn new(
        network: &NetworkDefinition,
        header: TransactionHeader,
        manifest: &str,
        blobs: Vec<Vec<u8>>,
    ) -> Result<Self, IntentCreationError> {
        if network.id != header.network_id {
            return Err(IntentCreationError::ConfigErr(
                IntentConfigError::MismatchedNetwork {
                    expected: network.id,
                    actual: header.network_id,
                },
            ));
        }
        Ok(Self {
            header,
            manifest: compile(manifest, &network, blobs)?,
        })
    }

    pub fn from_slice(slice: &[u8]) -> Result<Self, DecodeError> {
        scrypto_decode(slice)
    }

    pub fn hash(&self) -> Result<Hash, EncodeError> {
        Ok(hash(self.to_bytes()?))
    }

    pub fn to_bytes(&self) -> Result<Vec<u8>, EncodeError> {
        scrypto_encode(self)
    }
}

impl SignedTransactionIntent {
    pub fn from_slice(slice: &[u8]) -> Result<Self, DecodeError> {
        scrypto_decode(slice)
    }

    pub fn hash(&self) -> Result<Hash, EncodeError> {
        Ok(hash(self.to_bytes()?))
    }

    pub fn to_bytes(&self) -> Result<Vec<u8>, EncodeError> {
        scrypto_encode(self)
    }
}

impl NotarizedTransaction {
    pub fn from_slice(slice: &[u8]) -> Result<Self, DecodeError> {
        scrypto_decode(slice)
    }

    pub fn hash(&self) -> Result<Hash, EncodeError> {
        Ok(hash(self.to_bytes()?))
    }

    pub fn to_bytes(&self) -> Result<Vec<u8>, EncodeError> {
        scrypto_encode(self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::signing::*;

    #[test]
    fn construct_sign_and_notarize_ecdsa_secp256k1() {
        // create a key pair
        let sk1 = EcdsaSecp256k1PrivateKey::from_u64(1).unwrap();
        let sk2 = EcdsaSecp256k1PrivateKey::from_u64(2).unwrap();
        let sk_notary = EcdsaSecp256k1PrivateKey::from_u64(3).unwrap();

        // construct
        let intent = TransactionIntent::new(
            &NetworkDefinition::simulator(),
            TransactionHeader {
                version: 1,
                network_id: NetworkDefinition::simulator().id,
                start_epoch_inclusive: 0,
                end_epoch_exclusive: 100,
                nonce: 5,
                notary_public_key: sk_notary.public_key().into(),
                notary_as_signatory: false,
                cost_unit_limit: 1_000_000,
                tip_percentage: 5,
            },
            "CLEAR_AUTH_ZONE;",
            Vec::new(),
        )
        .unwrap();

        // sign
        let signature1 = sk1.sign(&intent.to_bytes().unwrap());
        let signature2 = sk2.sign(&intent.to_bytes().unwrap());
        let signed_intent = SignedTransactionIntent {
            intent,
            intent_signatures: vec![signature1.into(), signature2.into()],
        };

        // notarize
        let signature3 = sk_notary.sign(&signed_intent.to_bytes().unwrap());
        let transaction = NotarizedTransaction {
            signed_intent,
            notary_signature: signature3.into(),
        };

        assert_eq!(
            "641c247aa64c7f8f6706f365efcb2898b43893f006b748c2de46929756c08f5e",
            transaction.signed_intent.intent.hash().unwrap().to_string()
        );
        assert_eq!(
            "6815ed0fc10d5d5561ef894b32022ff3e360a8e9a55b17692118e8a833f254f4",
            transaction.signed_intent.hash().unwrap().to_string()
        );
        assert_eq!(
            "f49ed28a70c90e2c2394f40bc65b9a28a2eda812865002179b7369a2a50d5384",
            transaction.hash().unwrap().to_string()
        );
        assert_eq!("5c2102210221022109070107f20a00000000000000000a64000000000000000a0500000000000000220001b102f9308a019258c31049344f85f89d5229b531c845836f99b08601f113bce036f901000940420f00080500210220220109002020002022020001b20093ce440fadbd89f53ec11e8f33a9f52073a4fb1447e4ca2fae5cc6951a8950a62d7cb46ee8b398796b630db258da0bf0bdb8a7d7dfad483e14ebf0eb2f6870da0001b201f6ab8f5364c763ad822790649738ee7ec9c69141bff73887e2124ffcd836c8ce703269f29484d8e7d2fbbd89c5f7960490b264de0ae1f3acdce3cc4dc4db9d6c220001b200b4fafc970acef2d0d63df34abac36e2f26e212d939c2d0aeeb14493b4e54ef0e27902de7e88e1864a56ce7f0cdcb797f1919d9e4f2c7c6262ef4958c2a777c08", hex::encode(scrypto_encode(&transaction).unwrap()));
    }

    #[test]
    fn construct_sign_and_notarize_eddsa_ed25519() {
        // create a key pair
        let sk1 = EddsaEd25519PrivateKey::from_u64(1).unwrap();
        let sk2 = EddsaEd25519PrivateKey::from_u64(2).unwrap();
        let sk_notary = EddsaEd25519PrivateKey::from_u64(3).unwrap();

        // construct
        let intent = TransactionIntent::new(
            &NetworkDefinition::simulator(),
            TransactionHeader {
                version: 1,
                network_id: NetworkDefinition::simulator().id,
                start_epoch_inclusive: 0,
                end_epoch_exclusive: 100,
                nonce: 5,
                notary_public_key: sk_notary.public_key().into(),
                notary_as_signatory: false,
                cost_unit_limit: 1_000_000,
                tip_percentage: 5,
            },
            "CLEAR_AUTH_ZONE;",
            Vec::new(),
        )
        .unwrap();

        // sign
        let signature1 = (sk1.public_key(), sk1.sign(&intent.to_bytes().unwrap()));
        let signature2 = (sk2.public_key(), sk2.sign(&intent.to_bytes().unwrap()));
        let signed_intent = SignedTransactionIntent {
            intent,
            intent_signatures: vec![signature1.into(), signature2.into()],
        };

        // notarize
        let signature3 = sk_notary.sign(&signed_intent.to_bytes().unwrap());
        let transaction = NotarizedTransaction {
            signed_intent,
            notary_signature: signature3.into(),
        };

        assert_eq!(
            "9e6a155f408a445b7f5249bfc2df9dbc4f94b78a1b81f170d2dc0f91529cc212",
            transaction.signed_intent.intent.hash().unwrap().to_string()
        );
        assert_eq!(
            "d0e153166223c8d5501c28eca048b627a42c149971468740f9a75b73c778d8c1",
            transaction.signed_intent.hash().unwrap().to_string()
        );
        assert_eq!(
            "a634164a0713efc33d14fc60f610657c50b21f4c761308de00a5ee1312f496b0",
            transaction.hash().unwrap().to_string()
        );
        assert_eq!("5c2102210221022109070107f20a00000000000000000a64000000000000000a0500000000000000220101b3f381626e41e7027ea431bfe3009e94bdd25a746beec468948d6c3c7c5dc9a54b01000940420f00080500210220220109002020002022020102b34cb5abf6ad79fbf5abbccafcc269d85cd2651ed4b885b5869f241aedf0a5ba29b4e756712638dc7deeabdee71bddc156f84cb69b24ed2bac7a0806a25a9831c1d63e26dfdb402313a07c9f3f1c4d2862dfb97b968f1dfbacc532eb0ce65ba66d090102b37422b9887598068e32c4448a949adb290d0f4e35b9e01b0ee5f1a1e600fe2674b4400bf5f21b9427bd6d379ee9200804066cf219044ac7f2cb9c1e22dccb122befee9e513a63f6f56ca120d91c04a00d7f250d80afcaaf089b942e4e631ed8d804220101b48b0b9f9ebe27b6a16158a91413488b3c9718cd0280d505e79efed54fc0edd7dfc2e31a4707925a642dd8d00b61c46cda670c16e8750bab5755cfa9f0f4092e0b", hex::encode(scrypto_encode(&transaction).unwrap()));
    }
}
