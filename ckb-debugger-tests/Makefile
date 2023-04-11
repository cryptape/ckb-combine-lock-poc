

always-success:
	../target/debug/always-success > tx.json
	RUST_LOG=debug ckb-debugger --tx-file=tx.json --script-group-type=lock --cell-index=0 --cell-type=input
