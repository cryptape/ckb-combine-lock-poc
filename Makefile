

MOLC := moleculec

all:
	capsule build --release

mol:
	${MOLC} --language rust --schema-file crates/molecule-types/combine_lock.mol | rustfmt > crates/molecule-types/src/combine_lock.rs
	${MOLC} --language rust --schema-file crates/molecule-types/lock_wrapper.mol | rustfmt > crates/molecule-types/src/lock_wrapper.rs

ci:
	cd tests/global-registry && cargo test && cd ../..
	capsule build --release
	make -C ckb-debugger-tests all

dev:
	capsule build --release -- --features log

install:
	wget 'https://github.com/XuJiandong/ckb-standalone-debugger/releases/download/ckb2023-0621/ckb-debugger-linux-x64.tar.gz'
	tar zxvf ckb-debugger-linux-x64.tar.gz
	mv ckb-debugger ~/.cargo/bin/ckb-debugger-2023
	cargo install cross --git https://github.com/cross-rs/cross
	wget 'https://github.com/nervosnetwork/capsule/releases/download/v0.10.0/capsule_v0.10.0_x86_64-linux.tar.gz'
	tar xzvf capsule_v0.10.0_x86_64-linux.tar.gz
	mv capsule_v0.10.0_x86_64-linux/capsule ~/.cargo/bin
	cargo install moleculec --git https://github.com/nervosnetwork/molecule.git --rev 1306c29c529ab375e0368ffeb691bd8c7bbf0403
