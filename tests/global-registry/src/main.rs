#![allow(dead_code)]

extern crate alloc;

mod error;
#[path = "../../../ckb-lock-common/src/intersection.rs"]
mod intersection;
#[path = "../../../ckb-lock-common/src/transforming.rs"]
mod transforming;

use intersection::get_intersection;
use transforming::{BatchTransformingStatus, Cell};
use core::ops::Range;


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

fn test_intersection(
    chunk: (usize, usize),
    target: (usize, usize),
    result: Option<Range<usize>>,
) {
    let r = get_intersection(chunk.0 .. (chunk.0 + chunk.1), target.0 .. (target.0 + target.1));
    let r2 = if let Some(rr) = r {
        Some(rr.start - chunk.0..rr.end - chunk.0)
    } else {
        r
    };
    assert_eq!(r2, result);
}

#[test]
fn test_intersection_1() {
    let target = (0, 50);
    let chunk = (100, 50);
    test_intersection(chunk, target, None);
    let target = (0, 100);
    let chunk = (100, 50);
    test_intersection(chunk, target, None);
    let target = (0, 101);
    let chunk = (100, 50);
    test_intersection(chunk, target, Some(0..1));
    let target = (0, 130);
    let chunk = (100, 50);
    test_intersection(chunk, target, Some(0..30));
    let target = (0, 150);
    let chunk = (100, 50);
    test_intersection(chunk, target, Some(0..50));
    let target = (0, 151);
    let chunk = (100, 50);
    test_intersection(chunk, target, Some(0..50));
    let target = (0, 200);
    let chunk = (100, 50);
    test_intersection(chunk, target, Some(0..50));

    let target = (99, 1);
    let chunk = (100, 50);
    test_intersection(chunk, target, None);

    let target = (100, 1);
    let chunk = (100, 50);
    test_intersection(chunk, target, Some(0..1));

    let target = (100, 40);
    let chunk = (100, 50);
    test_intersection(chunk, target, Some(0..40));

    let target = (100, 50);
    let chunk = (100, 50);
    test_intersection(chunk, target, Some(0..50));

    let target = (101, 50);
    let chunk = (100, 50);
    test_intersection(chunk, target, Some(1..50));

    let target = (149, 50);
    let chunk = (100, 50);
    test_intersection(chunk, target, Some(49..50));

    let target = (150, 50);
    let chunk = (100, 50);
    test_intersection(chunk, target, None);

    let target = (200, 50);
    let chunk = (100, 50);
    test_intersection(chunk, target, None);
}

fn main() {}
