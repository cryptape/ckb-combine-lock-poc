#![allow(dead_code)]

use anyhow;
use ckb_hash::blake2b_256;
use ckb_jsonrpc_types::{CellInput, CellOutput, JsonBytes, OutPoint, Script};
use ckb_mock_tx_types::{ReprMockInput, ReprMockTransaction};
use ckb_types::packed;
use molecule::prelude::Entity;

use crate::{combine_lock_mol::ChildScriptConfig, create_script_from_cell_dep, read_tx_template};

// simplify: use only always success, repeat with `SimpleChildScriptConfig`
// times. note, same count means same child script config hash.
type SimpleChildScriptConfig = usize;

pub struct AssetCell {
    pub config: SimpleChildScriptConfig,
}

pub enum ConfigCellType {
    Fake([u8; 32]),
    Real(SimpleChildScriptConfig),
}

pub struct ConfigCell {
    pub type_: ConfigCellType,
    pub next_hash: [u8; 32],
}

pub struct Transforming {
    pub input_asset_cells: Vec<AssetCell>,
    pub input_config_cells: Vec<ConfigCell>,
    pub output_config_cells: Vec<ConfigCell>,
}

pub struct Transaction {
    pub template_file_name: String,
    // some predefined cell template in json file. represented as cell_dep index
    pub global_registry_index: usize,
    pub always_success_index: usize,
    pub combine_lock_index: usize,

    pub transforming: Vec<Transforming>,

    // private part
    tx: ReprMockTransaction,
    global_registry_script: Script,
    always_success_script: Script,
    combine_lock_script: Script,
    global_registry_id: [u8; 32],
}

pub fn create_input(lock: Script, type_: Option<Script>, data: JsonBytes) -> ReprMockInput {
    let dummy_outpoint = OutPoint {
        tx_hash: [0u8; 32].into(),
        index: 0.into(),
    };
    let input = CellInput {
        since: 0.into(),
        previous_output: dummy_outpoint,
    };
    let output = CellOutput {
        capacity: 10000.into(),
        lock,
        type_,
    };
    ReprMockInput {
        input,
        output,
        data,
        header: None,
    }
}

pub fn create_output(lock: Script, type_: Option<Script>) -> CellOutput {
    CellOutput {
        capacity: 9000.into(),
        lock,
        type_,
    }
}

impl Transaction {
    fn new(
        template_file_name: &str,
        always_success_index: usize,
        global_registry_index: usize,
        combine_lock_index: usize,
    ) -> Self {
        let tx = read_tx_template(template_file_name).unwrap();
        let always_success_script = create_script_from_cell_dep(&tx, always_success_index, false)
            .unwrap()
            .into();
        let combine_lock_script = create_script_from_cell_dep(&tx, combine_lock_index, false)
            .unwrap()
            .into();
        let mut global_registry_script: Script =
            create_script_from_cell_dep(&tx, global_registry_index, true)
                .unwrap()
                .into();
        // a fake args
        global_registry_script.args = JsonBytes::from_vec(vec![0u8; 32]);
        let global_registry_id = {
            let script: packed::Script = global_registry_script.clone().into();
            let hash = script.calc_script_hash();
            hash.as_slice().try_into().unwrap()
        };
        Self {
            template_file_name: template_file_name.into(),
            always_success_index,
            global_registry_index,
            combine_lock_index,
            transforming: vec![],

            tx,
            always_success_script,
            global_registry_script,
            combine_lock_script,
            global_registry_id,
        }
    }
    fn create_combine_lock(&self, child_script_config_hash: [u8; 32]) -> Script {
        let mut lock = self.combine_lock_script.clone();
        let mut args = vec![1u8];
        args.extend(self.global_registry_id.clone());
        args.extend(child_script_config_hash);
        lock.args = JsonBytes::from_vec(args);
        lock
    }

    // this is a fake config cell. Can used as input config cell for inserting
    fn append_fake_input_config_cell(
        &mut self,
        fake_current_hash: [u8; 32],
        fake_next_hash: [u8; 32],
    ) {
        let mut data = fake_next_hash.to_vec();
        data.extend(vec![0u8; 32]);
        let input = create_input(
            self.create_combine_lock(fake_current_hash),
            Some(self.global_registry_script.clone()),
            JsonBytes::from_vec(data),
        );
        self.tx.mock_info.inputs.push(input);
    }
    // this is a true config cell. Can be used as input config cell for updating
    fn append_input_config_cell(&mut self, config: ChildScriptConfig, next_hash: [u8; 32]) {
        let hash = blake2b_256(config.as_slice());
        let lock = self.create_combine_lock(hash);
        let mut data = next_hash.to_vec();
        data.extend(config.as_slice());
        let input = create_input(
            lock,
            Some(self.global_registry_script.clone()),
            JsonBytes::from_vec(data),
        );
        self.tx.mock_info.inputs.push(input);
    }

    fn append_input_asset_cell(&mut self, child_script_config_hash: [u8; 32]) {
        let input = create_input(
            self.create_combine_lock(child_script_config_hash),
            None,
            JsonBytes::from_vec(vec![]),
        );
        self.tx.mock_info.inputs.push(input);
    }

    fn append_output_asset_cell(&mut self, child_script_config_hash: [u8; 32]) {
        let lock = self.create_combine_lock(child_script_config_hash);
        let output = create_output(lock, None);
        self.tx.tx.outputs.push(output);
        self.tx.tx.outputs_data.push(JsonBytes::from_vec(vec![]));
    }
    fn append_output_config_cell(&mut self, config: ChildScriptConfig, next_hash: [u8; 32]) {
        let hash = blake2b_256(config.as_slice());
        let lock = self.create_combine_lock(hash);
        let mut data = next_hash.to_vec();
        data.extend(config.as_slice());
        let output = create_output(lock, Some(self.global_registry_script.clone()));
        self.tx.tx.outputs.push(output);
        self.tx.tx.outputs_data.push(JsonBytes::from_vec(data));
    }
    fn append_fake_output_config_cell(&mut self, fake_current_hash: [u8; 32], next_hash: [u8; 32]) {
        let lock = self.create_combine_lock(fake_current_hash);
        let mut data = next_hash.to_vec();
        data.extend(vec![0u8; 32]);
        let output = create_output(lock, Some(self.global_registry_script.clone()));
        self.tx.tx.outputs.push(output);
        self.tx.tx.outputs_data.push(JsonBytes::from_vec(data));
    }
    pub fn generate(&self) -> Result<(), anyhow::Error> {
        Ok(())
    }
}
