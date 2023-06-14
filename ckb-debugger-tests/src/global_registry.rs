#![allow(dead_code)]

use anyhow;
use ckb_hash::blake2b_256;
use ckb_jsonrpc_types::{CellInput, CellOutput, JsonBytes, OutPoint, Script};
use ckb_mock_tx_types::{ReprMockInput, ReprMockTransaction};
use ckb_types::packed;
use molecule::{bytes::Bytes, prelude::Entity};

use crate::{
    combine_lock_mol::ChildScriptConfig, create_child_script_config, create_script_from_cell_dep,
    create_witness_args, read_tx_template,
};

// Use always success script. Repeat with `SimpleChildScriptConfig` times. Note,
// same count means same child script config hash. It's quite important that
// in global registry, every config cell has unique child script config hash.
type SimpleChildScriptConfig = usize;

#[derive(Clone)]
pub struct AssetCell {
    pub config: SimpleChildScriptConfig,
}

//
// the next hash is derived from child script config. It's uncontrollable. So we
// need fake next hash to test some cases.
//
#[derive(Clone)]
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

pub struct BatchTransforming {
    pub template_file_name: String,
    // some predefined cell template in json file. represented as cell_dep index
    pub global_registry_index: usize,
    pub always_success_index: usize,
    pub combine_lock_index: usize,
    pub transforming: Vec<Transforming>,
    pub tx: ReprMockTransaction,

    // private part
    global_registry_script: Script,
    always_success_script: Script,
    combine_lock_script: Script,
    global_registry_id: [u8; 32],
}

pub fn create_input(lock: Script, type_: Option<Script>, data: JsonBytes) -> ReprMockInput {
    let r: packed::Script = lock.clone().into();
    let random = blake2b_256(r.as_slice());
    let dummy_outpoint = OutPoint {
        tx_hash: random.into(),
        index: 0.into(),
    };
    let input = CellInput {
        since: 0.into(),
        previous_output: dummy_outpoint,
    };
    let output = CellOutput {
        capacity: 9000.into(),
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

impl BatchTransforming {
    pub fn new(
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

    // this is a fake config cell. Can be used as input config cell for inserting
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
    fn create_config(&self, config: SimpleChildScriptConfig) -> ChildScriptConfig {
        let cell_dep_index = vec![self.always_success_index];
        let args = vec![Bytes::new()];
        let vec = vec![0u8; config];
        let vec_vec = vec![vec.as_slice()];
        create_child_script_config(&self.tx, &cell_dep_index, &args, vec_vec.as_slice(), true).unwrap()
    }
    // current hash or next hash
    pub fn create_hash(&self, config: SimpleChildScriptConfig) -> [u8; 32] {
        let config = self.create_config(config);
        blake2b_256(config.as_slice())
    }
    fn append_witness(&mut self, config: SimpleChildScriptConfig) {
        let inner_witness = vec![Bytes::new(); config];
        let config = self.create_config(config);
        let args = create_witness_args(&config, 0, &inner_witness).unwrap();
        self.tx
            .tx
            .witnesses
            .push(JsonBytes::from_bytes(args.as_bytes()));
    }
    fn append_fake_witness(&mut self) {
        self.tx
            .tx
            .witnesses
            .push(JsonBytes::from_bytes(Bytes::new()));
    }
    pub fn generate(&mut self) -> Result<(), anyhow::Error> {
        // input config cells
        let mut inputs = vec![];
        for trans in &self.transforming {
            let hashes: Vec<([u8; 32], [u8; 32], Option<ChildScriptConfig>)> = trans
                .input_config_cells
                .iter()
                .map(|c| {
                    let (hash, data) = match c.type_ {
                        ConfigCellType::Fake(h) => (h, None),
                        ConfigCellType::Real(config) => {
                            let c = self.create_config(config);
                            (blake2b_256(c.as_slice()), Some(c))
                        }
                    };
                    (hash, c.next_hash, data)
                })
                .collect::<Vec<_>>();
            inputs.extend(hashes);
        }
        for i in inputs {
            if let Some(config) = i.2 {
                self.append_input_config_cell(config, i.1);
            } else {
                self.append_fake_input_config_cell(i.0, i.1);
            }
        }
        // witness
        let mut types = vec![];
        for trans in &self.transforming {
            for i in &trans.input_config_cells {
                types.push(i.type_.clone());
            }
        }
        for t in types {
            match t {
                ConfigCellType::Fake(_) => {
                    self.append_fake_witness();
                }
                ConfigCellType::Real(c) => {
                    self.append_witness(c);
                }
            }
        }

        // input asset cells
        let mut input_hashes = vec![];
        for trans in &self.transforming {
            let hashes: Vec<[u8; 32]> = trans
                .input_asset_cells
                .iter()
                .map(|c| blake2b_256(self.create_config(c.config).as_slice()))
                .collect::<Vec<_>>();
            input_hashes.extend(hashes);
        }
        for hash in input_hashes {
            self.append_input_asset_cell(hash);
        }
        // witness
        let mut assets = vec![];
        for trans in &self.transforming {
            for i in &trans.input_asset_cells {
                assets.push(i.config);
            }
        }
        for c in assets {
            self.append_witness(c);
        }

        // output config cells
        let mut outputs = vec![];
        for trans in &self.transforming {
            let hashes: Vec<([u8; 32], [u8; 32], Option<ChildScriptConfig>)> = trans
                .output_config_cells
                .iter()
                .map(|c| {
                    let (hash, data) = match c.type_ {
                        ConfigCellType::Fake(h) => (h, None),
                        ConfigCellType::Real(config) => {
                            let c = self.create_config(config);
                            (blake2b_256(c.as_slice()), Some(c))
                        }
                    };
                    (hash, c.next_hash, data)
                })
                .collect::<Vec<_>>();
            outputs.extend(hashes);
        }
        for i in outputs {
            if let Some(config) = i.2 {
                self.append_output_config_cell(config, i.1);
            } else {
                self.append_fake_output_config_cell(i.0, i.1);
            }
        }

        // auto fill
        if self.tx.tx.cell_deps.len() == 0 {
            self.tx.tx.cell_deps = self
                .tx
                .mock_info
                .cell_deps
                .iter()
                .map(|c| c.cell_dep.clone())
                .collect::<Vec<_>>();
        }
        if self.tx.tx.inputs.len() == 0 {
            self.tx.tx.inputs = self
                .tx
                .mock_info
                .inputs
                .iter()
                .map(|c| c.input.clone())
                .collect::<Vec<_>>();
        }

        Ok(())
    }
}

pub fn find_middle(a: [u8; 32], b: [u8; 32]) -> [u8; 32] {
    let mut result = [0; 32];
    for i in 0..32 {
        let x = a[i] as u16;
        let y = b[i] as u16;
        result[i] = ((x + y) / 2) as u8;
    }
    result
}

pub fn find_smaller(a: &[u8; 32]) -> [u8; 32] {
    let mut part = u128::from_be_bytes(a[16..32].try_into().unwrap());
    assert!(part != 0);
    part -= 1;
    let part = part.to_be_bytes();
    let mut result = a.clone();
    result[16..32].copy_from_slice(&part);
    result
}

pub fn find_larger(a: &[u8; 32]) -> [u8; 32] {
    let mut part = u128::from_be_bytes(a[16..32].try_into().unwrap());
    part += 1;
    let part = part.to_be_bytes();
    let mut result = a.clone();
    result[16..32].copy_from_slice(&part);
    result
}
