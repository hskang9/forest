// Copyright 2020 ChainSafe Systems
// SPDX-License-Identifier: Apache-2.0

use crate::signature::{self, verify_bls_sig, BLS_SIG_LEN};
use bls_signatures::{Serialize as BlsSerialize, Signature};
use serde::{Deserialize, Serialize};

use encoding::{ser, de, serde_bytes};

pub struct VRFPublicKey(Vec<u8>);

/// Contains some public key type to be used for VRF verification
impl VRFPublicKey {
    pub fn new(input: Vec<u8>) -> Self {
        Self(input)
    }
}

/// The output from running a VRF
#[derive(Clone, Debug, PartialEq, Eq, Ord, PartialOrd, Default, Serialize, Deserialize)]
pub struct VRFResult(
    #[serde(with = "serde_bytes")]
    Vec<u8>
);

//impl ser::Serialize for VRFResult {
//    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
//        where
//            S: ser::Serializer,
//    {
//        let value = serde_bytes::Bytes::new(&self.0);
//        serde_bytes::Serialize::serialize(&value, serializer)
//    }
//}
//
//impl<'de> de::Deserialize<'de> for VRFResult {
//    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
//        where
//            D: de::Deserializer<'de>,
//    {
//        let bz: Vec<u8> = serde_bytes::Deserialize::deserialize(deserializer)?;
//        Ok(VRFResult(bz))
//    }
//}

// TODO verify format or implement custom serialize/deserialize function (if necessary):
// https://github.com/ChainSafe/ferret/issues/143

impl VRFResult {
    /// Creates a VRFResult from a raw vector
    pub fn new(output: Vec<u8>) -> Self {
        Self(output)
    }
    /// Returns clone of underlying vector
    pub fn to_bytes(&self) -> Vec<u8> {
        self.0.clone()
    }
    /// Returns max value based on [BLS_SIG_LEN](constant.BLS_SIG_LEN.html)
    pub fn max_value() -> Self {
        Self::new([std::u8::MAX; BLS_SIG_LEN].to_vec())
    }
    /// Validates syntax...
    pub fn validate_syntax(&self) -> bool {
        unimplemented!()
    }
    /// Asserts whether `input` was used with `pk` to produce this VRFOutput
    pub fn verify(&self, input: Vec<u8>, pk: VRFPublicKey) -> bool {
        match Signature::from_bytes(&self.0) {
            Ok(sig) => verify_bls_sig(&input, pk.0, signature::Signature::new(sig.as_bytes())),
            Err(_) => false,
        }
    }
}

// TODO verify format or implement custom serialize/deserialize function (if necessary):
// https://github.com/ChainSafe/ferret/issues/143

#[cfg(test)]
mod tests {
    use super::*;
    use bls_signatures::{PrivateKey, Serialize};
    use rand::{Rng, SeedableRng, XorShiftRng};

    #[test]
    fn verify() {
        let rng = &mut XorShiftRng::from_seed([0x3dbe6259, 0x8d313d76, 0x3237db17, 0xe5bc0654]);
        let privk = PrivateKey::generate(rng);

        let init_input = (0..64).map(|_| rng.gen()).collect::<Vec<u8>>();
        let input = privk.sign(&init_input);

        let genesis = VRFResult::new(input.as_bytes());

        let sig = privk.sign(&genesis.to_bytes());
        let res = VRFResult::new(sig.as_bytes());

        let pubk = VRFPublicKey::new(privk.public_key().as_bytes());

        assert!(res.verify(genesis.to_bytes(), pubk));
    }
}
