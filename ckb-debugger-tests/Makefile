
CKB_DEBUGGER ?= ckb-debugger-2023

all: \
	always-success \
	cl-always-success \
	cl-always-success-info-cell \
	cl-child-script \
	child-script-success \
	auth-script-exec-success


always-success:
	cargo run --bin always-success |  ${CKB_DEBUGGER} --tx-file=- -s lock

cl-always-success:
	cargo run --bin cl-always-success |  ${CKB_DEBUGGER} --tx-file=- -s lock

cl-always-success-info-cell:
	cargo run --bin cl-always-success-info-cell |  ${CKB_DEBUGGER} --tx-file=- -s lock

cl-child-script:
	cargo run --bin cl-child-script |  ${CKB_DEBUGGER} --tx-file=- -s lock

child-script-success:
	cargo run --bin child-script-success |  ${CKB_DEBUGGER} --tx-file=- -s lock

child-script-multisig-success:
	cargo run --bin child-script-multisig-success |  ${CKB_DEBUGGER} --tx-file=- -s lock

child-script-multi-inputs:
	cargo run --bin child-script-multi-inputs |  ${CKB_DEBUGGER} --tx-file=- -s lock

auth-script-exec-success:
	cargo run --bin auth-script-exec-success |  ${CKB_DEBUGGER} --tx-file=- -s lock
