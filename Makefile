

MOLC := moleculec

all:
	capsule build --release

mol: ckb-lock-common/src/generated/blockchain.rs
	${MOLC} --language rust --schema-file crates/types/combine_lock.mol | rustfmt > crates/types/src/combine_lock.rs
	${MOLC} --language rust --schema-file crates/types/lock_wrapper.mol | rustfmt > crates/types/src/lock_wrapper.rs

target/blockchain.json: crates/types/blockchain.mol
	${MOLC} --language - --schema-file $^ --format json > $@

ckb-lock-common/src/generated/blockchain.rs: target/blockchain.json
	moleculec-c2 --rust --input $< | rustfmt > $@

ci:
	cd tests/global-registry && cargo test && cd ../..
	capsule build --release
	make -C ckb-debugger-tests all

# this is optional
install-moleculec:
	cargo install --git https://github.com/XuJiandong/moleculec-c2.git --rev cece491 moleculec-c2
	cargo install --force --version "0.7.3" "moleculec"

install:
	wget 'https://github.com/XuJiandong/ckb-standalone-debugger/releases/download/ckb2023-0621/ckb-debugger-linux-x64.tar.gz'
	tar zxvf ckb-debugger-linux-x64.tar.gz
	mv ckb-debugger ~/.cargo/bin/ckb-debugger-2023
	cargo install cross --git https://github.com/cross-rs/cross
	wget 'https://github.com/nervosnetwork/capsule/releases/download/v0.10.0/capsule_v0.10.0_x86_64-linux.tar.gz'
	tar xzvf capsule_v0.10.0_x86_64-linux.tar.gz
	mv capsule_v0.10.0_x86_64-linux/capsule ~/.cargo/bin
