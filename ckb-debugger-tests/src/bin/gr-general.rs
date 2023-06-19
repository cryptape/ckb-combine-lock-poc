use ckb_debugger_tests::global_registry::{
    find_middle, find_smaller, AssetCell, BatchTransforming, ConfigCell, ConfigCellType,
    Transforming,
};
use ckb_jsonrpc_types::JsonBytes;
use clap::Parser;

#[derive(Parser)]
struct Args {
    #[arg(long)]
    insert: bool,
    #[arg(long)]
    update: bool,
    #[arg(long)]
    batch_insert: bool,
    #[arg(long)]
    batch_transforming: bool,
    #[arg(long)]
    insert_fail_modify: bool,
    #[arg(long)]
    insert_fail_gap: bool,
}

pub fn main() -> Result<(), Box<dyn std::error::Error>> {
    drop(env_logger::init());
    let args = Args::parse();
    if args.insert {
        return insert();
    } else if args.update {
        return update();
    } else if args.batch_insert {
        return batch_insert();
    } else if args.batch_transforming {
        return batch_transforming();
    } else if args.insert_fail_modify {
        return insert_fail_modify();
    } else if args.insert_fail_gap {
        return insert_fail_gap();
    }
    unreachable!();
}

pub fn insert() -> Result<(), Box<dyn std::error::Error>> {
    let mut batch = BatchTransforming::new(
        "../ckb-debugger-tests/templates/gr-general.json",
        0,
        1,
        2,
        3,
    );
    let next_hash = batch.create_hash(1);
    batch.transforming.push(Transforming {
        input_config_cells: vec![ConfigCell {
            type_: ConfigCellType::Fake([0u8; 32]),
            next_hash: [0xFF; 32],
        }],
        input_asset_cells: vec![AssetCell { config: 1 }],
        output_config_cells: vec![
            ConfigCell {
                type_: ConfigCellType::Fake([0u8; 32]),
                next_hash,
            },
            ConfigCell {
                type_: ConfigCellType::Real(1),
                next_hash: [0xFF; 32],
            },
        ],
    });

    batch.generate()?;

    let json = serde_json::to_string_pretty(&batch.tx).unwrap();
    println!("{}", json);
    Ok(())
}

pub fn update() -> Result<(), Box<dyn std::error::Error>> {
    let mut batch = BatchTransforming::new(
        "../ckb-debugger-tests/templates/gr-general.json",
        0,
        1,
        2,
        3,
    );
    batch.transforming.push(Transforming {
        input_asset_cells: vec![],
        input_config_cells: vec![ConfigCell {
            type_: ConfigCellType::Real(1),
            next_hash: [0xFF; 32],
        }],
        output_config_cells: vec![ConfigCell {
            type_: ConfigCellType::Real(1),
            next_hash: [0xFF; 32],
        }],
    });

    batch.generate()?;

    let json = serde_json::to_string_pretty(&batch.tx).unwrap();
    println!("{}", json);
    Ok(())
}

pub fn batch_insert() -> Result<(), Box<dyn std::error::Error>> {
    let mut batch = BatchTransforming::new(
        "../ckb-debugger-tests/templates/gr-general.json",
        0,
        1,
        2,
        3,
    );
    let next_hash = batch.create_hash(1);
    let next_hash2 = batch.create_hash(2);
    assert!(next_hash < next_hash2);

    batch.transforming.push(Transforming {
        input_config_cells: vec![ConfigCell {
            type_: ConfigCellType::Fake([0u8; 32]),
            next_hash: [0xFF; 32],
        }],
        input_asset_cells: vec![AssetCell { config: 1 }, AssetCell { config: 2 }],
        output_config_cells: vec![
            ConfigCell {
                type_: ConfigCellType::Fake([0u8; 32]),
                next_hash: next_hash,
            },
            ConfigCell {
                type_: ConfigCellType::Real(1),
                next_hash: next_hash2,
            },
            ConfigCell {
                type_: ConfigCellType::Real(2),
                next_hash: [0xFF; 32],
            },
        ],
    });

    batch.generate()?;

    let json = serde_json::to_string_pretty(&batch.tx).unwrap();
    println!("{}", json);
    Ok(())
}

pub fn batch_transforming() -> Result<(), Box<dyn std::error::Error>> {
    let mut batch = BatchTransforming::new(
        "../ckb-debugger-tests/templates/gr-general.json",
        0,
        1,
        2,
        3,
    );
    let next_hash = batch.create_hash(1);
    let next_hash2 = batch.create_hash(2);
    assert!(next_hash < next_hash2);
    let middle = find_middle(next_hash, next_hash2);

    // insert next_hash2 in [middle, 0xFF..FF]
    batch.transforming.push(Transforming {
        input_config_cells: vec![ConfigCell {
            type_: ConfigCellType::Fake(middle),
            next_hash: [0xFF; 32],
        }],
        input_asset_cells: vec![AssetCell { config: 2 }],
        output_config_cells: vec![
            ConfigCell {
                type_: ConfigCellType::Fake(middle),
                next_hash: next_hash2,
            },
            ConfigCell {
                type_: ConfigCellType::Real(2),
                next_hash: [0xFF; 32],
            },
        ],
    });
    batch.transforming.push(Transforming {
        input_asset_cells: vec![],
        input_config_cells: vec![ConfigCell {
            type_: ConfigCellType::Real(1),
            next_hash: middle,
        }],
        output_config_cells: vec![ConfigCell {
            type_: ConfigCellType::Real(1),
            next_hash: middle,
        }],
    });

    batch.generate()?;

    let json = serde_json::to_string_pretty(&batch.tx).unwrap();
    println!("{}", json);
    Ok(())
}

pub fn insert_fail_modify() -> Result<(), Box<dyn std::error::Error>> {
    let mut batch = BatchTransforming::new(
        "../ckb-debugger-tests/templates/gr-general.json",
        0,
        1,
        2,
        3,
    );
    let next_hash = batch.create_hash(1);
    let next_hash2 = batch.create_hash(2);
    assert!(next_hash < next_hash2);

    batch.transforming.push(Transforming {
        input_config_cells: vec![ConfigCell {
            type_: ConfigCellType::Fake([0u8; 32]),
            next_hash: [0xFF; 32],
        }],
        input_asset_cells: vec![AssetCell { config: 1 }, AssetCell { config: 2 }],
        output_config_cells: vec![
            ConfigCell {
                type_: ConfigCellType::Fake([0u8; 32]),
                next_hash: next_hash,
            },
            ConfigCell {
                type_: ConfigCellType::Real(1),
                next_hash: next_hash2,
            },
            ConfigCell {
                type_: ConfigCellType::Real(2),
                next_hash: [0xFF; 32],
            },
        ],
    });

    batch.generate()?;
    // modify cell data after next hash, not allowed
    let data = batch.tx.tx.outputs_data[0].clone();
    let mut data = data.into_bytes().to_vec();
    data[32] += 1;
    batch.tx.tx.outputs_data[0] = JsonBytes::from_vec(data.into());

    let json = serde_json::to_string_pretty(&batch.tx).unwrap();
    println!("{}", json);
    Ok(())
}

pub fn insert_fail_gap() -> Result<(), Box<dyn std::error::Error>> {
    let mut batch = BatchTransforming::new(
        "../ckb-debugger-tests/templates/gr-general.json",
        0,
        1,
        2,
        3,
    );
    let next_hash = batch.create_hash(1);
    let next_hash2 = batch.create_hash(2);
    assert!(next_hash < next_hash2);
    let fake_hash2 = find_smaller(&next_hash);

    batch.transforming.push(Transforming {
        input_config_cells: vec![ConfigCell {
            type_: ConfigCellType::Fake([0u8; 32]),
            next_hash: [0xFF; 32],
        }],
        input_asset_cells: vec![AssetCell { config: 1 }, AssetCell { config: 2 }],
        output_config_cells: vec![
            ConfigCell {
                type_: ConfigCellType::Fake([0u8; 32]),
                // there is a gap, failed
                next_hash: fake_hash2,
            },
            ConfigCell {
                type_: ConfigCellType::Real(1),
                next_hash: next_hash2,
            },
            ConfigCell {
                type_: ConfigCellType::Real(2),
                next_hash: [0xFF; 32],
            },
        ],
    });

    batch.generate()?;

    let json = serde_json::to_string_pretty(&batch.tx).unwrap();
    println!("{}", json);
    Ok(())
}
