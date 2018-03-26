# rust-mozjs

Rust bindings to SpiderMonkey

[Documentation](http://doc.servo.org/mozjs/)

## Setting up your environment

### Rust

This project requires Rust nightly-2018-03-25 or greater. You can install this with Rustup.rs:

### Rustup.rs

To install on Windows, download and run [`rustup-init.exe`](https://win.rustup.rs/)
then follow the onscreen instructions.

To install on other systems, run:

```sh
curl https://sh.rustup.rs -sSf | sh
```

This will also download the current stable version of Rust.
To default to nightly, run instead:

```sh
curl https://sh.rustup.rs -sSf | sh -s -- --default-toolchain nightly
```

If you already have Rustup.rs installed, run:

```sh
rustup toolchain install nightly
rustup default nightly
```

### Other dependencies

#### OS X
#### On OS X (homebrew)

```sh
brew install automake pkg-config python2 cmake
```

## Building the project

```sh
cargo build
```

## Testing the project

```sh
cargo test
```
