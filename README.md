# CKB Combine Lock PoC

Design draft https://cryptape.notion.site/Combine-Lock-Script-e49f1f2de7504e8da1c9e6961309b317

Note: This is a draft design and PoC implementation. The design could change drastically in future iterations.

### Requirements
- Rust 1.69 or above
- Clang 16 or above
- ckb-debugger

### How to Build
Install build tools:
``` sh
make install
```
It will install ckb-debugger, clang-16 and Rust target(riscv64imac-unknown-none-elf). 

Build contracts:
``` sh
cargo build --release --target=riscv64imac-unknown-none-elf
```

Run tests:
``` sh
make ci
```
