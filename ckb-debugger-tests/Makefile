

all: always-success cl-always-success cl-always-success-info-cell


always-success:
	cargo run --bin always-success > tx.json
	RUST_LOG=debug ckb-debugger --tx-file=tx.json --script-group-type=lock --cell-index=0 --cell-type=input

cl-always-success:
	cargo run --bin cl-always-success > tx.json
	RUST_LOG=debug ckb-debugger --tx-file=tx.json --script-group-type=lock --cell-index=0 --cell-type=input

cl-always-success-info-cell:
	cargo run --bin cl-always-success-info-cell > tx.json
	RUST_LOG=debug ckb-debugger --tx-file=tx.json --script-group-type=lock --cell-index=0 --cell-type=input
