#![allow(dead_code)]

extern crate alloc;

mod error;
#[path = "../../../ckb-combine-lock-common/src/transforming.rs"]
mod transforming;

use transforming::{BatchTransformingStatus, Cell};

fn test(inputs: &[(u8, u8)], outputs: &[(u8, u8)], result: bool) {
    let mut batch = BatchTransformingStatus::new();

    for i in inputs {
        batch.set_input(Cell::new(0, [i.0; 32], [i.1; 32])).unwrap();
    }
    for o in outputs {
        batch
            .set_output(Cell::new(0, [o.0; 32], [o.1; 32]))
            .unwrap();
    }
    assert_eq!(batch.validate(), result);
}

#[test]
fn test_single() {
    test(&[(0, 9)], &[(0, 9)], true);
    test(&[(0, 9)], &[(0, 1), (1, 9)], true);
    test(
        &[(0, 255)],
        &[(30, 50), (0, 30), (50, 100), (100, 255)],
        true,
    );

    test(&[(0, 9)], &[(0, 1), (1, 8)], false);
    test(&[(0, 9)], &[(1, 2), (2, 9)], false);
    test(&[(0, 9)], &[(0, 2), (3, 9)], false);
}

#[test]
fn test_batch() {
    // updating mixed with inserting
    test(&[(0, 9), (50, 60)], &[(0, 1), (1, 9), (50, 60)], true);
    // 2 inserting
    test(
        &[(0, 9), (50, 60)],
        &[(1, 9), (0, 1), (50, 55), (55, 60)],
        true,
    );
    // fail
    test(
        &[(0, 9), (50, 60)],
        &[(0, 1), (1, 9), (50, 55), (55, 56)],
        false,
    );
}

#[test]
#[should_panic]
fn test_wrong_pair() {
    let _ = Cell::new(0, [1; 32], [0; 32]);
}

#[test]
fn test_output_is_empty() {
    test(&[(0, 9)], &[], false);
}

#[test]
fn test_dangling_pair() {
    let mut batch = BatchTransformingStatus::new();
    let input = Cell::new(0, [4; 32], [6; 32]);
    batch.set_input(input.clone()).unwrap();

    let output = Cell::new(0, [5; 32], [7; 32]);
    let result = batch.set_output(output);
    assert!(result.is_err());

    let output = Cell::new(0, [3; 32], [5; 32]);
    let result = batch.set_output(output);
    assert!(result.is_err());
}

#[test]
fn test_input_overlap() {
    let mut batch = BatchTransformingStatus::new();
    let input = Cell::new(0, [0; 32], [1; 32]);
    batch.set_input(input.clone()).unwrap();
    let result = batch.set_input(input.clone());
    assert!(result.is_err());

    let mut batch = BatchTransformingStatus::new();
    let input = Cell::new(0, [0; 32], [3; 32]);
    batch.set_input(input).unwrap();
    let input = Cell::new(0, [2; 32], [4; 32]);
    let result = batch.set_input(input);
    assert!(result.is_err());
}

fn main() {}
