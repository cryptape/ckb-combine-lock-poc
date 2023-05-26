use super::error::Error;
use alloc::vec::Vec;

#[derive(Clone, Default)]
pub struct HashPair {
    pub current_hash: [u8; 32],
    pub next_hash: [u8; 32],
}

impl HashPair {
    pub fn new(current_hash: [u8; 32], next_hash: [u8; 32]) -> Self {
        assert!(current_hash < next_hash);
        Self {
            current_hash,
            next_hash,
        }
    }
    pub fn in_range(&self, outer: &Self) -> bool {
        outer.current_hash <= self.current_hash
            && self.current_hash < self.next_hash
            && self.next_hash <= outer.next_hash
    }
    pub fn no_overlap(&self, other: &Self) -> bool {
        if self.next_hash <= other.current_hash {
            return true;
        }
        if other.next_hash <= self.current_hash {
            return true;
        }
        return false;
    }
}

/// CL = Combine Lock, CC = Config Cell
/// Some transforming below. Left on input and right on output.
/// insert 1 config cell: CL + CC -> CC + CC
/// insert 2 config cells: CL + CL + CC -> CC + CC + CC
/// insert N config cells: CL + ... + CL + CC -> CC + CC + ... + CC
/// update config cell: CC -> CC
/// Import notes:
/// 1. Updating config cell is actually inserting 0 config cell
/// 2. There is always one config cell on left
/// 3. There can be many config cells on right
/// 4. There are many transforming in one transaction.
pub struct TransformingStatus {
    pub input: HashPair,
    pub outputs: Vec<HashPair>,
}

impl TransformingStatus {
    pub fn new(input: HashPair) -> Self {
        Self {
            input,
            outputs: Vec::new(),
        }
    }
    pub fn try_push(&mut self, pair: &HashPair) -> bool {
        if pair.in_range(&self.input) {
            self.outputs.push(pair.clone());
            return true;
        } else {
            return false;
        }
    }
    pub fn validate(&mut self) -> bool {
        if self.outputs.len() == 0 {
            return false;
        }
        self.outputs
            .sort_by(|a, b| a.current_hash.cmp(&b.current_hash));
        if self.input.current_hash != self.outputs[0].current_hash {
            return false;
        }
        for i in 1..self.outputs.len() {
            if self.outputs[i - 1].next_hash != self.outputs[i].current_hash {
                return false;
            }
        }
        if self.input.next_hash != self.outputs.last().unwrap().next_hash {
            return false;
        }
        true
    }
}

pub struct BatchTransformingStatus {
    pub transforming: Vec<TransformingStatus>,
}

impl BatchTransformingStatus {
    pub fn new() -> Self {
        Self {
            transforming: Default::default(),
        }
    }
    pub fn set_input(&mut self, input: HashPair) -> Result<(), Error> {
        for s in &self.transforming {
            if !s.input.no_overlap(&input) {
                return Err(Error::OverlapPair);
            }
        }
        self.transforming.push(TransformingStatus::new(input));
        Ok(())
    }
    pub fn set_output(&mut self, output: HashPair) -> Result<(), Error> {
        for s in &mut self.transforming {
            if s.try_push(&output) {
                return Ok(());
            }
        }
        Err(Error::DanglingPair)
    }
    pub fn validate(&mut self) -> bool {
        for s in &mut self.transforming {
            if !s.validate() {
                return false;
            }
        }
        return true;
    }
}
