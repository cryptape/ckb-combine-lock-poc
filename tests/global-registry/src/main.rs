#![allow(dead_code)]

extern crate alloc;

mod error;
#[path = "../../../contracts/global-registry/src/transforming.rs"]
mod transforming;

use transforming::{BatchTransformingStatus, HashPair};

fn test(input: (u8, u8), outputs: &[(u8, u8)], result: bool) {
    let mut batch = BatchTransformingStatus::new();

    let input = HashPair::new([input.0; 32], [input.1; 32]);
    batch.set_input(input).unwrap();
    for o in outputs {
        batch
            .set_output(HashPair::new([o.0; 32], [o.1; 32]))
            .unwrap();
    }
    assert_eq!(batch.validate(), result);
}

#[test]
fn test_global_registry() {
    test((0, 9), &[(0, 9)], true);
    test((0, 9), &[(0, 1), (1, 9)], true);
    test((0, 255), &[(30, 50), (0, 30), (50, 100), (100, 255)], true);

    test((0, 9), &[(0, 1), (1, 8)], false);
}

#[test]
#[should_panic]
fn test_wrong_case() {
    test((0, 9), &[(0, 100)], false);
}

#[test]
fn test_fail() {
    let mut batch = BatchTransformingStatus::new();
    let input = HashPair::new([0; 32], [1; 32]);
    batch.set_input(input.clone()).unwrap();
    let result = batch.set_input(input.clone());
    assert!(result.is_err());
}

fn main() {}
