

all: always-success cl-always-success cl-always-success-info-cell cl-child-script

always-success:
	cargo run --bin always-success | RUST_LOG=debug ckb-debugger --tx-file=- -s lock

cl-always-success:
	cargo run --bin cl-always-success | RUST_LOG=debug ckb-debugger --tx-file=- -s lock

cl-always-success-info-cell:
	cargo run --bin cl-always-success-info-cell | RUST_LOG=debug ckb-debugger --tx-file=- -s lock

child-script-example-success:
	cargo run --bin child-script-example-success | RUST_LOG=debug ckb-debugger --tx-file=- -s lock

cl-child-script:
	cargo run --bin cl-child-script | RUST_LOG=debug ckb-debugger --tx-file=- -s lock
