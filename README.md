## libairx - AirX Core Library

Provides UTF-8 encoded text and binary data transmission over LAN for AirX.

[![Release Build & Test](https://github.com/hatsune-miku/libairx/actions/workflows/rust.yml/badge.svg)](https://github.com/hatsune-miku/libairx/actions/workflows/rust.yml)

- Fast & Lightweight
    - Written in pure Rust.
    - Plaintext transmission.
    - Low-cost discovery service.
- Reliable
    - TCP-based data transmission.
    - Hash-verified data packets.

[API Documentation](https://github.com/hatsune-miku/libairx/wiki)

---

### Features

- LAN Discovery with group ID
- Share text over LAN
- Share files of any size over LAN
- Cross-platform support

### Usage

- Test

```shell
cargo test
```

- Build

```shell
cargo build --release --lib
```

### Credits

| Contributor |     #     |
|:-----------:|:---------:|
|    G, Z     | 202191382 |
|    L, S     | 201714987 |
|    G, J     | 202096888 |
|    G, C     | 202194431 |
