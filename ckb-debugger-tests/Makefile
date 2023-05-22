
CKB_DEBUGGER ?= ckb-debugger-2023


## cl: combine lock
## gr: combine lock with global registry
all: \
	cl-always-success \
	cl-child-script \
	gr-child-script \
	gr-child-script-no-config-cell\
	child-script-success \
	child-script-multi-inputs \
	auth-script-exec-success

cl-always-success:
	cargo run --bin cl-always-success |  ${CKB_DEBUGGER} --tx-file=- -s lock

cl-child-script:
	cargo run --bin cl-child-script |  ${CKB_DEBUGGER} --tx-file=- -s lock

gr-child-script:
	cargo run --bin gr-child-script has-config-cell |  ${CKB_DEBUGGER} --tx-file=- -s lock

gr-child-script-no-config-cell:
	cargo run --bin gr-child-script |  ${CKB_DEBUGGER} --tx-file=- -s lock

child-script-success:
	cargo run --bin child-script-success |  ${CKB_DEBUGGER} --tx-file=- -s lock

child-script-multi-inputs:
	cargo run --bin child-script-multi-inputs |  ${CKB_DEBUGGER} --tx-file=- -s lock

auth-script-exec-success:
	cargo run --bin auth-script-exec-success |  ${CKB_DEBUGGER} --tx-file=- -s lock
