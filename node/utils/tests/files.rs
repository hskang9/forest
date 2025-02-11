// Copyright 2020 ChainSafe Systems
// SPDX-License-Identifier: Apache-2.0, MIT

use utils::{count_files, read_file_to_string, read_file_to_vec, read_toml, write_to_file};

use serde_derive::Deserialize;
use std::fs::remove_dir_all;

// Please use with caution, remove_dir_all will completely delete a directory
fn cleanup_file(path: &str) {
    if let Err(e) = remove_dir_all(path) {
        panic!("cleanup_file() path: {:?} failed: {:?}", path, e);
    }
}

#[test]
fn write_to_file_to_path() {
    let msg = "Hello World!".as_bytes().to_vec();
    let path = "./test-write/";
    let file = "test.txt";
    match write_to_file(&msg, &path, file) {
        Ok(_) => cleanup_file(path),
        Err(e) => {
            cleanup_file(path);
            panic!("{:?}", e);
        }
    }
}

#[test]
fn write_to_file_nested_dir() {
    let msg = "Hello World!".as_bytes().to_vec();
    let root = "./test_missing";
    let path = format!("{}{}", root, "/test_write_string/");
    match write_to_file(&msg, &path, "test-file") {
        Ok(_) => cleanup_file(root),
        Err(e) => {
            cleanup_file(root);
            panic!("{:?}", e);
        }
    }
}

#[test]
fn read_from_file_vec() {
    let msg = "Hello World!".as_bytes().to_vec();
    let path = "./test_read_file/";
    let file_name = "out.keystore";
    if let Err(e) = write_to_file(&msg, &path, file_name) {
        panic!(e);
    }
    match read_file_to_vec(&format!("{}{}", path, file_name)) {
        Ok(contents) => {
            cleanup_file(path);
            assert_eq!(contents, msg)
        }
        Err(e) => {
            cleanup_file(path);
            panic!("{:?}", e);
        }
    }
}

#[test]
fn read_from_file_string() {
    let msg = "Hello World!";
    let path = "./test_string_read_file/";
    let file_name = "out.keystore";
    if let Err(e) = write_to_file(&msg.as_bytes().to_vec(), &path, file_name) {
        panic!(e);
    }
    match read_file_to_string(&format!("{}{}", path, file_name)) {
        Ok(contents) => {
            cleanup_file(path);
            assert_eq!(contents, msg)
        }
        Err(e) => {
            cleanup_file(path);
            panic!("{:?}", e);
        }
    }
}

// For test_read_toml()
#[derive(Deserialize)]
struct Config {
    ip: String,
    port: Option<u16>,
    keys: Keys,
}

// For test_read_toml()
#[derive(Deserialize)]
struct Keys {
    github: String,
    travis: Option<String>,
}

// Test taken from https://docs.rs/toml/0.5.5/toml/
#[test]
fn read_from_toml() {
    let toml_str = r#"
        ip = '127.0.0.1'

        [keys]
        github = 'xxxxxxxxxxxxxxxxx'
        travis = 'yyyyyyyyyyyyyyyyy'
    "#;
    let config: Config = match read_toml(toml_str) {
        Ok(contents) => contents,
        Err(e) => panic!(e),
    };

    assert_eq!(config.ip, "127.0.0.1");
    assert_eq!(config.port, None);
    assert_eq!(config.keys.github, "xxxxxxxxxxxxxxxxx");
    assert_eq!(config.keys.travis.as_ref().unwrap(), "yyyyyyyyyyyyyyyyy");
}

#[test]
fn count_files_in_a_dir() {
    let msg = "Hello World!";
    let path = "./test_string_read_file/";
    let file_name = "out.keystore";
    write_to_file(msg.as_bytes(), &path, file_name).unwrap();
    match count_files(path) {
        Ok(file_count) => {
            cleanup_file(path);
            assert_eq!(1, file_count);
        }
        Err(e) => {
            cleanup_file(path);
            panic!("{:?}", e);
        }
    }
}
