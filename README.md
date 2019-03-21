# jobshop-rust
Solving the job-shop problem in rust

### Install on windows


1. Download and run [rustup](https://rustup.rs)
    1. Continue without installing Microsoft Build Tools if prompted
    2. Customize install
    3. Default host triple: `x86_64-pc-windows-gnu`
        a. `rustup toolchain add nightly-gnu`
    4. Default toolchain: `nightly`
        a. `rustup default nightly-gnu`
    5. Modify path variables: Up to you
    6. Proceed with installation
2. Install [MSYS2](https://www.msys2.org)
4. Open `MSYS2 MSYS` from the start menu
5. `pacman -S mingw-w64-x86_64-toolchain mingw-w64-x86_64-gtk3`
6. Add path variables to PATH environment
7. Build project via terminal: `cargo build`
