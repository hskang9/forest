// Copyright 2020 ChainSafe Systems
// SPDX-License-Identifier: Apache-2.0, MIT

use encoding::{de, ser, serde_bytes, to_vec, Error as EncodingError};
use serde::{Deserialize, Serialize};
use std::ops::Deref;

/// Method number indicator for calling actor methods
#[derive(Default, Clone, Copy, PartialEq, Debug, Serialize, Deserialize)]
pub struct MethodNum(u64); // TODO: add constraints to this

// TODO verify format or implement custom serialize/deserialize function (if necessary):
// https://github.com/ChainSafe/forest/issues/143

impl MethodNum {
    /// Constructor for new MethodNum
    pub fn new(num: u64) -> Self {
        Self(num)
    }
}

impl From<MethodNum> for u64 {
    fn from(method_num: MethodNum) -> u64 {
        method_num.0
    }
}

/// Base actor send method
pub const METHOD_SEND: isize = 0;
/// Base actor constructor method
pub const METHOD_CONSTRUCTOR: isize = 1;
/// Base actor cron method
pub const METHOD_CRON: isize = 2;

/// Placeholder for non base methods for actors
// TODO revisit on complete spec
pub const METHOD_PLACEHOLDER: isize = 3;

/// Serialized bytes to be used as parameters into actor methods
#[derive(Default, Clone, PartialEq, Debug)]
pub struct Serialized {
    bytes: Vec<u8>,
}

impl ser::Serialize for Serialized {
    fn serialize<S>(&self, s: S) -> Result<S::Ok, S::Error>
    where
        S: ser::Serializer,
    {
        let value = serde_bytes::Bytes::new(&self.bytes);
        serde_bytes::Serialize::serialize(value, s)
    }
}

impl<'de> de::Deserialize<'de> for Serialized {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: de::Deserializer<'de>,
    {
        let bz: Vec<u8> = serde_bytes::Deserialize::deserialize(deserializer)?;
        Ok(Serialized::new(bz))
    }
}

impl Deref for Serialized {
    type Target = Vec<u8>;
    fn deref(&self) -> &Self::Target {
        &self.bytes
    }
}

impl Serialized {
    /// Constructor if data is encoded already
    pub fn new(bytes: Vec<u8>) -> Self {
        Self { bytes }
    }

    /// Contructor for encoding Cbor encodable structure
    pub fn serialize<O: ser::Serialize>(obj: O) -> Result<Self, EncodingError> {
        Ok(Self {
            bytes: to_vec(&obj)?,
        })
    }

    /// Returns serialized bytes
    pub fn bytes(&self) -> Vec<u8> {
        self.bytes.clone()
    }
}
