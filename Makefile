

MOLC := moleculec

all:
	cargo build --release --target=riscv64imac-unknown-none-elf

mol: ckb-lock-common/src/generated/blockchain.rs
	${MOLC} --language rust --schema-file crates/types/combine_lock.mol | rustfmt > crates/types/src/combine_lock.rs
	${MOLC} --language rust --schema-file crates/types/lock_wrapper.mol | rustfmt > crates/types/src/lock_wrapper.rs
	${MOLC} --language - --schema-file crates/types/blockchain.mol --format json | moleculec-c2 --rust --input - | rustfmt > ckb-lock-common/src/generated/blockchain.rs
	${MOLC} --language - --schema-file crates/types/lock_wrapper.mol --format json | moleculec-c2 --rust --input - | rustfmt > ckb-lock-common/src/generated/lock_wrapper.rs
	${MOLC} --language - --schema-file crates/types/combine_lock.mol --format json | moleculec-c2 --rust --input - | rustfmt > ckb-lock-common/src/generated/combine_lock.rs

ci:
	cd tests/global-registry && cargo test && cd ../..
	cargo build --release --target=riscv64imac-unknown-none-elf
	make -C ckb-debugger-tests all

# this is optional
install-moleculec:
	cargo install --git https://github.com/XuJiandong/moleculec-c2.git --rev 4f1bd3c moleculec-c2
	cargo install --force --version "0.7.3" "moleculec"

install:
	rustup target add riscv64imac-unknown-none-elf
	wget 'https://github.com/XuJiandong/ckb-standalone-debugger/releases/download/ckb2023-0621/ckb-debugger-linux-x64.tar.gz'
	tar zxvf ckb-debugger-linux-x64.tar.gz
	mv ckb-debugger ~/.cargo/bin/ckb-debugger-2023
	wget https://apt.llvm.org/llvm.sh && chmod +x llvm.sh && sudo ./llvm.sh 16 && rm llvm.sh