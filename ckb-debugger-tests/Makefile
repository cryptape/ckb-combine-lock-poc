

all: always-success cl-always-success cl-always-success-info-cell cl-child-script child-script-success child-script-multisig-success


always-success:
	cargo run --bin always-success | RUST_LOG=debug ckb-debugger --tx-file=- -s lock

cl-always-success:
	cargo run --bin cl-always-success | RUST_LOG=debug ckb-debugger --tx-file=- -s lock

cl-always-success-info-cell:
	cargo run --bin cl-always-success-info-cell | RUST_LOG=debug ckb-debugger --tx-file=- -s lock

cl-child-script:
	cargo run --bin cl-child-script | RUST_LOG=debug ckb-debugger --tx-file=- -s lock

child-script-success:
	cargo run --bin child-script-success > tx.json
	RUST_LOG=debug ckb-debugger --tx-file=tx.json --script-group-type=lock --cell-index=0 --cell-type=input

child-script-multisig-success:
	cargo run --bin child-script-multisig-success > tx.json
	RUST_LOG=debug ckb-debugger --tx-file=tx.json --script-group-type=lock --cell-index=0 --cell-type=input
