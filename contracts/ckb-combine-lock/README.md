

### Build
There are 3 options:
1. use capsule at root folder:
```
capsule build
```
Suggested for production usage for sake of reproducible build.

2. use native compiler
```
rustup target add riscv64imac-unknown-none-elf
sudo apt install gcc-riscv64-unknown-elf
cargo build --release --target riscv64imac-unknown-none-elf
```
3. use nix command
```
sh <(curl -L https://nixos.org/nix/install) --daemon --yes
rustup target add riscv64imac-unknown-none-elf
CC_riscv64imac_unknown_none_elf="$(nix build --print-out-paths --no-link "nixpkgs#pkgsCross.riscv64-embedded.stdenv.cc.out" 2>/dev/null)/bin/riscv64-none-elf-gcc" cargo build --release --target=riscv64imac-unknown-none-elf
```
Suggested for developing phase.

