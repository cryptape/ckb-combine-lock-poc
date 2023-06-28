
CKB_DEBUGGER ?= ckb-debugger-2023
BUILD ?= release

## cl: combine lock
## gr: combine lock with global registry
all: \
	gr-general-insert\
	gr-general-batch-insert\
	gr-general-batch-transforming\
	gr-general-update\
	gr-general-insert-fail-gap\
	gr-general-insert-fail-modify\
	gr-init \
	cl-always-success \
	gr-insert \
	gr-child-script \
	gr-child-script-no-config-cell \
	cl-always-success-3i3c \
	cl-cl-always-success \
	cl-child-script \
	child-script-success \
	child-script-long-witness \
	child-script-multi-inputs \
	negative-cl-always-failure \
	negative-cl-child-script-config-hash-error \
	negative-cl-child-script-sig-error \
	negative-cl-cl-always-failure \
	negative-cl-index-error \
	negative-cl-vec-index-error \
	negative-cl-witness-length-wrong\
	gr-update \

cl-always-success:
	cargo run --$(BUILD) --bin $@ | ${CKB_DEBUGGER} --tx-file=- -s lock

cl-always-success-3i3c:
	cargo run --$(BUILD) --bin $@ | ${CKB_DEBUGGER} --tx-file=- -s lock --cell-index=0
	cargo run --$(BUILD) --bin $@ | ${CKB_DEBUGGER} --tx-file=- -s lock --cell-index=1
	cargo run --$(BUILD) --bin $@ | ${CKB_DEBUGGER} --tx-file=- -s lock --cell-index=2

cl-cl-always-success:
	cargo run --$(BUILD) --bin $@ | ${CKB_DEBUGGER} --tx-file=- -s lock

cl-child-script:
	cargo run --$(BUILD) --bin $@ | ${CKB_DEBUGGER} --tx-file=- -s lock

gr-child-script:
	cargo run --$(BUILD) --bin gr-child-script -- --has-config-cell |  ${CKB_DEBUGGER} --tx-file=- -s lock

gr-child-script-no-config-cell:
	cargo run --$(BUILD) --bin gr-child-script | ${CKB_DEBUGGER} --tx-file=- -s lock

gr-init:
	cargo run --$(BUILD) --bin gr-init | ${CKB_DEBUGGER} --tx-file=- -s type --cell-index=0 --cell-type output

gr-insert:
	cargo run --$(BUILD) --bin gr-insert | ${CKB_DEBUGGER} --tx-file=- -s type --cell-index=1 --cell-type input
	cargo run --$(BUILD) --bin gr-insert | ${CKB_DEBUGGER} --tx-file=- -s lock --cell-index=1
	cargo run --$(BUILD) --bin gr-insert | ${CKB_DEBUGGER} --tx-file=- -s lock

gr-general-insert:
	cargo run --$(BUILD) --bin gr-general -- --insert | ${CKB_DEBUGGER} --tx-file=- -s type --cell-index=0 --cell-type input
	cargo run --$(BUILD) --bin gr-general -- --insert | ${CKB_DEBUGGER} --tx-file=- -s lock
	cargo run --$(BUILD) --bin gr-general -- --insert | ${CKB_DEBUGGER} --tx-file=- -s lock --cell-index=1

gr-general-batch-insert:
	cargo run --$(BUILD) --bin gr-general -- --batch-insert | ${CKB_DEBUGGER} --tx-file=- -s type --cell-index=0 --cell-type input
	cargo run --$(BUILD) --bin gr-general -- --batch-insert | ${CKB_DEBUGGER} --tx-file=- -s lock
	cargo run --$(BUILD) --bin gr-general -- --batch-insert | ${CKB_DEBUGGER} --tx-file=- -s lock --cell-index=1

gr-general-batch-transforming:
	cargo run --$(BUILD) --bin gr-general -- --batch-transforming | ${CKB_DEBUGGER} --tx-file=- -s type --cell-index=0 --cell-type input
	cargo run --$(BUILD) --bin gr-general -- --batch-transforming | ${CKB_DEBUGGER} --tx-file=- -s lock
	cargo run --$(BUILD) --bin gr-general -- --batch-transforming | ${CKB_DEBUGGER} --tx-file=- -s lock --cell-index=1
	cargo run --$(BUILD) --bin gr-general -- --batch-transforming | ${CKB_DEBUGGER} --tx-file=- -s lock --cell-index=2

gr-general-update:
	cargo run --$(BUILD) --bin gr-general -- --update | ${CKB_DEBUGGER} --tx-file=- -s lock --cell-index=0
	cargo run --$(BUILD) --bin gr-general -- --update | ${CKB_DEBUGGER} --tx-file=- -s type --cell-index=0 --cell-type input

gr-general-insert-fail-modify:
	cargo run --$(BUILD) --bin gr-general -- --insert-fail-modify | \
	${CKB_DEBUGGER} --tx-file=- -s type --cell-index=0 --cell-type input | grep "Run result: 54"

gr-general-insert-fail-gap:
	cargo run --$(BUILD) --bin gr-general -- --insert-fail-gap | \
	${CKB_DEBUGGER} --tx-file=- -s type --cell-index=0 --cell-type input | grep "Run result: 53"

gr-update:
	cargo run --$(BUILD) --bin gr-update | ${CKB_DEBUGGER} --tx-file=- -s lock
	cargo run --$(BUILD) --bin gr-update | ${CKB_DEBUGGER} --tx-file=- -s type --cell-index=0 --cell-type input
	cargo run --$(BUILD) --bin gr-update | ${CKB_DEBUGGER} --tx-file=- -s type --cell-index=0 --cell-type output

child-script-success:
	cargo run --$(BUILD) --bin $@ | ${CKB_DEBUGGER} --tx-file=- -s lock

child-script-long-witness:
	cargo run --$(BUILD) --bin $@ | ${CKB_DEBUGGER} --tx-file=- -s lock

child-script-multi-inputs:
	cargo run --$(BUILD) --bin $@ | ${CKB_DEBUGGER} --tx-file=- -s lock

negative-cl-always-failure:
	cargo run --$(BUILD) --bin negative -- cl-always-failure | ${CKB_DEBUGGER} --tx-file=- -s lock | grep "Run result: 84"

negative-cl-child-script-config-hash-error:
	cargo run --$(BUILD) --bin negative -- cl-child-script-config-hash-error | ${CKB_DEBUGGER} --tx-file=- -s lock | grep "Run result: 86"

negative-cl-child-script-sig-error:
	cargo run --$(BUILD) --bin negative -- cl-child-script-sig-error | ${CKB_DEBUGGER} --tx-file=- -s lock | grep "Run result: 84"

negative-cl-cl-always-failure:
	cargo run --$(BUILD) --bin negative -- cl-cl-always-failure | ${CKB_DEBUGGER} --tx-file=- -s lock | grep "Run result: 84"

negative-cl-index-error:
	cargo run --$(BUILD) --bin negative -- cl-index-error | ${CKB_DEBUGGER} --tx-file=- -s lock | grep "Run result: 83"

negative-cl-vec-index-error:
	cargo run --$(BUILD) --bin negative -- cl-vec-index-error | ${CKB_DEBUGGER} --tx-file=- -s lock | grep "Run result: 82"

negative-cl-witness-length-wrong:
	cargo run --$(BUILD) --bin negative -- cl-witness-length-wrong | ${CKB_DEBUGGER} --tx-file=- -s lock | grep "Run result: 85"
