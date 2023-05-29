
CKB_DEBUGGER ?= ckb-debugger-2023


## cl: combine lock
## gr: combine lock with global registry
all: \
	cl-always-success \
	cl-always-success-3i3c \
	cl-cl-always-success \
	cl-child-script \
	gr-child-script \
	gr-child-script-no-config-cell\
	child-script-success \
	child-script-multi-inputs \
	auth-script-exec-success \
	negative-cl-always-failure \
	negative-cl-child-script-config-hash-error \
	negative-cl-child-script-sig-error \
	negative-cl-cl-always-failure \
	negative-cl-index-error \
	negative-cl-vec-index-error \
	negative-cl-witness-length-wrong

cl-always-success:
	cargo run --bin $@ | ${CKB_DEBUGGER} --tx-file=- -s lock

cl-always-success-3i3c:
	cargo run --bin $@ | ${CKB_DEBUGGER} --tx-file=- -s lock --cell-index=0
	cargo run --bin $@ | ${CKB_DEBUGGER} --tx-file=- -s lock --cell-index=1
	cargo run --bin $@ | ${CKB_DEBUGGER} --tx-file=- -s lock --cell-index=2

cl-cl-always-success:
	cargo run --bin $@ | ${CKB_DEBUGGER} --tx-file=- -s lock

cl-child-script:
	cargo run --bin $@ | ${CKB_DEBUGGER} --tx-file=- -s lock

gr-child-script:
	cargo run --bin gr-child-script has-config-cell | ${CKB_DEBUGGER} --tx-file=- -s lock

gr-child-script-no-config-cell:
	cargo run --bin gr-child-script | ${CKB_DEBUGGER} --tx-file=- -s lock

child-script-success:
	cargo run --bin $@ | ${CKB_DEBUGGER} --tx-file=- -s lock

child-script-multi-inputs:
	cargo run --bin $@ | ${CKB_DEBUGGER} --tx-file=- -s lock

auth-script-exec-success:
	cargo run --bin $@ | ${CKB_DEBUGGER} --tx-file=- -s lock

negative-cl-always-failure:
	cargo run --bin $@ | ${CKB_DEBUGGER} --tx-file=- -s lock | grep "Run result: 85"

negative-cl-child-script-config-hash-error:
	cargo run --bin $@ | ${CKB_DEBUGGER} --tx-file=- -s lock | grep "Run result: 81"

negative-cl-child-script-sig-error:
	cargo run --bin $@ | ${CKB_DEBUGGER} --tx-file=- -s lock | grep "Run result: 85"

negative-cl-cl-always-failure:
	cargo run --bin $@ | ${CKB_DEBUGGER} --tx-file=- -s lock | grep "Run result: 85"

negative-cl-index-error:
	cargo run --bin $@ | ${CKB_DEBUGGER} --tx-file=- -s lock | grep "Run result: 84"

negative-cl-vec-index-error:
	cargo run --bin $@ | ${CKB_DEBUGGER} --tx-file=- -s lock | grep "Run result: 83"

negative-cl-witness-length-wrong:
	cargo run --bin $@ | ${CKB_DEBUGGER} --tx-file=- -s lock | grep "Run result: 88"
