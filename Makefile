

MOLC := moleculec

all:

all-via-docker:

mol:
	${MOLC} --language rust --schema-file ckb-combine-lock-common/blockchain.mol | rustfmt > ckb-combine-lock-common/src/blockchain.rs
	${MOLC} --language rust --schema-file ckb-combine-lock-common/combine_lock.mol | rustfmt > ckb-combine-lock-common/src/combine_lock_mol.rs

ci:
	capsule build --release

install:
	wget 'https://github.com/nervosnetwork/capsule/releases/download/v0.9.0/capsule_v0.9.0_x86_64-linux.tar.gz'
	tar xzvf capsule_v0.9.0_x86_64-linux.tar.gz
	mv capsule_v0.9.0_x86_64-linux/capsule ~/.cargo/bin
	cargo install moleculec --locked
