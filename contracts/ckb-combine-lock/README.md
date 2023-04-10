

### Build
There are 2 options:
1. use capsule at root folder:
```
capsule build
```
Suggested for production usage for sake of reproducible build.

2. use native compiler
```
rustup target add riscv64imac-unknown-none-elf
sudo apt install gcc-riscv64-unknown-elf
cargo build --release
```
Suggested for developing phase.

