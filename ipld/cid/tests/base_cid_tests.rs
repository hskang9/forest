// Copyright 2020 ChainSafe Systems
// SPDX-License-Identifier: Apache-2.0, MIT

use encoding::{from_slice, to_vec, Cbor};
use forest_cid::{Cid, Codec, Error, Prefix, Version};
use multihash;
use multihash::Hash::Blake2b512;
use serde::Serialize;
use std::collections::HashMap;

#[test]
fn basic_marshalling() {
    let h = multihash::encode(multihash::Hash::SHA2256, b"beep boop").unwrap();

    let cid = Cid::new(Codec::DagProtobuf, Version::V1, h);

    let data = cid.to_bytes();
    let out = Cid::from_raw_cid(data).unwrap();

    assert_eq!(cid, out);

    let s = cid.to_string();
    let out2 = Cid::from_raw_cid(&s[..]).unwrap();

    assert_eq!(cid, out2);
}

#[test]
fn empty_string() {
    assert_eq!(Cid::from_raw_cid(""), Err(Error::InputTooShort));
}

#[test]
fn v0_handling() {
    let old = "QmdfTbBqBPQ7VNxZEYEj14VmRuZBkqFbiwReogJgS1zR1n";
    let cid = Cid::from_raw_cid(old).unwrap();

    assert_eq!(cid.version, Version::V0);
    assert_eq!(cid.to_string(), old);
}

#[test]
fn from_str() {
    let cid: Cid = "QmdfTbBqBPQ7VNxZEYEj14VmRuZBkqFbiwReogJgS1zR1n"
        .parse()
        .unwrap();
    assert_eq!(cid.version, Version::V0);

    let bad = "QmdfTbBqBPQ7VNxZEYEj14VmRuZBkqFbiwReogJgS1zIII".parse::<Cid>();
    assert_eq!(bad, Err(Error::ParsingError));
}

#[test]
fn v0_error() {
    let bad = "QmdfTbBqBPQ7VNxZEYEj14VmRuZBkqFbiwReogJgS1zIII";
    assert_eq!(Cid::from_raw_cid(bad), Err(Error::ParsingError));
}

#[test]
fn prefix_roundtrip() {
    let data = b"awesome test content";
    let h = multihash::encode(multihash::Hash::SHA2256, data).unwrap();

    let cid = Cid::new(Codec::DagProtobuf, Version::V1, h);
    let prefix = cid.prefix();

    let cid2 = Cid::new_from_prefix(&prefix, data).unwrap();

    assert_eq!(cid, cid2);

    let prefix_bytes = prefix.as_bytes();
    let prefix2 = Prefix::new_from_bytes(&prefix_bytes).unwrap();

    assert_eq!(prefix, prefix2);
}

#[test]
fn from() {
    let the_hash = "QmdfTbBqBPQ7VNxZEYEj14VmRuZBkqFbiwReogJgS1zR1n";

    let cases = vec![
        format!("/ipfs/{:}", &the_hash),
        format!("https://ipfs.io/ipfs/{:}", &the_hash),
        format!("http://localhost:8080/ipfs/{:}", &the_hash),
    ];

    for case in cases {
        let cid = Cid::from_raw_cid(case).unwrap();
        assert_eq!(cid.version, Version::V0);
        assert_eq!(cid.to_string(), the_hash);
    }
}

#[test]
fn test_hash() {
    let data: Vec<u8> = vec![1, 2, 3];
    let prefix = Prefix {
        version: Version::V0,
        codec: Codec::DagProtobuf,
        mh_type: multihash::Hash::SHA2256,
        mh_len: 32,
    };
    let mut map = HashMap::new();
    let cid = Cid::new_from_prefix(&prefix, &data).unwrap();
    map.insert(cid.clone(), data.clone());
    assert_eq!(&data, map.get(&cid).unwrap());
}

#[test]
fn test_default() {
    let data: Vec<u8> = vec![1, 2, 3];

    let cid = Cid::from_bytes_default(&data).unwrap();

    let prefix = cid.prefix();
    assert_eq!(prefix.version, Version::V1);
    assert_eq!(prefix.codec, Codec::DagCBOR);
    assert_eq!(prefix.mh_type, Blake2b512);
    assert_eq!(
        prefix.mh_len,
        // 4 is Blake2b512 code length (3) + 1, change if default changes
        (Blake2b512.size() + 4) as usize
    );
}

#[derive(Serialize, Copy, Clone)]
struct TestCborStruct {
    name: &'static str,
}
impl Cbor for TestCborStruct {}

#[test]
fn test_cbor_to_cid() {
    let obj = TestCborStruct { name: "test" };

    let enc = Cid::from_cbor_default(obj).unwrap();
    let bz_enc = Cid::from_bytes_default(&obj.marshal_cbor().unwrap()).unwrap();
    assert_eq!(enc, bz_enc);
}

#[test]
fn vector_cid_serialize_round() {
    let cids = vec![
        Cid::from_bytes_default(&[0, 1]).unwrap(),
        Cid::from_bytes_default(&[1, 2]).unwrap(),
        Cid::from_bytes_default(&[3, 2]).unwrap(),
    ];

    // Serialize cids with cbor
    let enc = to_vec(&cids).unwrap();

    // decode cbor bytes to vector again
    let dec: Vec<Cid> = from_slice(&enc).unwrap();

    assert_eq!(cids, dec);
}
