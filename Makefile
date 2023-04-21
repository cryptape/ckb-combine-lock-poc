

MOLC := moleculec

all:
	capsule build --release

mol:
	${MOLC} --language rust --schema-file ckb-combine-lock-common/combine_lock.mol | rustfmt > ckb-combine-lock-common/src/combine_lock_mol.rs
	cp ckb-combine-lock-common/src/combine_lock_mol.rs ckb-debugger-tests/src
ci:
	capsule build --release
	make -C ckb-debugger-tests all

dev:
	capsule build --release -- --features log

install:
	cargo install --rev c6bd322 --git https://github.com/nervosnetwork/ckb-standalone-debugger ckb-debugger
	mv ~/.cargo/bin/ckb-debugger ~/.cargo/bin/ckb-debugger-2023
	wget 'https://github.com/nervosnetwork/capsule/releases/download/v0.9.0/capsule_v0.9.0_x86_64-linux.tar.gz'
	tar xzvf capsule_v0.9.0_x86_64-linux.tar.gz
	mv capsule_v0.9.0_x86_64-linux/capsule ~/.cargo/bin
	cargo install moleculec --git https://github.com/nervosnetwork/molecule.git --rev 1306c29c529ab375e0368ffeb691bd8c7bbf0403


