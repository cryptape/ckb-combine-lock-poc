use ckb_debugger_tests::global_registry::{
    find_middle, AssetCell, BatchTransforming, ConfigCell, ConfigCellType, Transforming,
};
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
    }
    unreachable!();
}

pub fn insert() -> Result<(), Box<dyn std::error::Error>> {
    let mut batch =
        BatchTransforming::new("../ckb-debugger-tests/templates/gr-general.json", 0, 1, 2);
    let next_hash = batch.create_next_hash(1);
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
    let mut batch =
        BatchTransforming::new("../ckb-debugger-tests/templates/gr-general.json", 0, 1, 2);
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
    let mut batch =
        BatchTransforming::new("../ckb-debugger-tests/templates/gr-general.json", 0, 1, 2);
    let next_hash = batch.create_next_hash(1);
    let next_hash2 = batch.create_next_hash(2);
    assert!(next_hash2 < next_hash);

    batch.transforming.push(Transforming {
        input_config_cells: vec![ConfigCell {
            type_: ConfigCellType::Fake([0u8; 32]),
            next_hash: [0xFF; 32],
        }],
        input_asset_cells: vec![AssetCell { config: 1 }, AssetCell { config: 2 }],
        output_config_cells: vec![
            ConfigCell {
                type_: ConfigCellType::Fake([0u8; 32]),
                next_hash: next_hash2,
            },
            ConfigCell {
                type_: ConfigCellType::Real(2),
                next_hash: next_hash,
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

pub fn batch_transforming() -> Result<(), Box<dyn std::error::Error>> {
    let mut batch =
        BatchTransforming::new("../ckb-debugger-tests/templates/gr-general.json", 0, 1, 2);
    let next_hash = batch.create_next_hash(1);
    let next_hash2 = batch.create_next_hash(2);
    assert!(next_hash2 < next_hash);
    let middle = find_middle(next_hash, next_hash2);

    // insert next_hash in [middle, 0xFF..FF]
    batch.transforming.push(Transforming {
        input_config_cells: vec![ConfigCell {
            type_: ConfigCellType::Fake(middle),
            next_hash: [0xFF; 32],
        }],
        input_asset_cells: vec![AssetCell { config: 1 }],
        output_config_cells: vec![
            ConfigCell {
                type_: ConfigCellType::Fake(middle),
                next_hash,
            },
            ConfigCell {
                type_: ConfigCellType::Real(1),
                next_hash: [0xFF; 32],
            },
        ],
    });

    // update next_hash2's cell
    batch.transforming.push(Transforming {
        input_asset_cells: vec![],
        input_config_cells: vec![ConfigCell {
            type_: ConfigCellType::Real(2),
            next_hash: middle,
        }],
        output_config_cells: vec![ConfigCell {
            type_: ConfigCellType::Real(2),
            next_hash: middle,
        }],
    });

    batch.generate()?;

    let json = serde_json::to_string_pretty(&batch.tx).unwrap();
    println!("{}", json);
    Ok(())
}
