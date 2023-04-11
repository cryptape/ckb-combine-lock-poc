mod hash;
mod smt;

use std::{fs::read_to_string, path::PathBuf};

use ckb_debugger_api::embed::Embed;
use ckb_mock_tx_types::{MockTransaction, ReprMockTransaction};
use serde_json::from_str as from_json_str;
use sparse_merkle_tree::H256;
use ckb_combine_lock_common::combine_lock_mol::{CombineLockWitness, ChildScript, ChildScriptVec, Uint16};
use hash::hash;
use smt::build_tree;
use ckb_combine_lock_common::molecule::prelude::*;
use ckb_combine_lock_common::molecule::bytes::Bytes;
use ckb_combine_lock_common::blockchain::Bytes as BlockchainBytes;
use ckb_combine_lock_common::ckb_std::ckb_types::prelude::*;

pub fn read_tx_template(file_name: &str) -> Result<MockTransaction, Box<dyn std::error::Error>> {
    let mock_tx = read_to_string(file_name)?;
    let mut mock_tx_embed = Embed::new(PathBuf::from(file_name), mock_tx.clone());
    let mock_tx = mock_tx_embed.replace_all();
    let repr_mock_tx: ReprMockTransaction = from_json_str(&mock_tx)?;
    Ok(repr_mock_tx.into())
}

// return smt root, witness args
pub fn create_simple_case(scripts: Vec<ChildScript>) -> (H256, Bytes) {
    let builder = ChildScriptVec::new_builder();
    let child_scripts = builder.extend(scripts).build();

    let h = hash(child_scripts.as_slice());
    let (root, proof) = build_tree(&Vec::from([h]));

    let index = Uint16::new_builder().nth0(1u8.into()).build();
    let proof: Bytes = proof.into();
    let proof2: BlockchainBytes = proof.pack();
    let combine_lock_witness = CombineLockWitness::new_builder().scripts(child_scripts).proof(proof2).witness_base_index(index).build();

    (root, combine_lock_witness.as_bytes())
}

