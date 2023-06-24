## libairx - AirX Core Library

[![Release Build & Test](https://github.com/hatsune-miku/libairx/actions/workflows/rust.yml/badge.svg)](https://github.com/hatsune-miku/libairx/actions/workflows/rust.yml)

- Fast & Lightweight
    - Written in pure Rust.
    - Plaintext transmission [1].
- Reliable
    - Low-cost discovery service.
    - Hash-verified network packets.
    - TCP-based text transmission.

[1] It's possible to implement some end-to-end encryption on top of this library to achieve
better security.

[API Documentation](https://github.com/hatsune-miku/libairx/wiki)

---

### Finished

- LAN Discovery
- Text sync over LAN (TCP)

### WIP

- Text sync over Internet (TCP)
- Image sync over LAN
- File sharing over LAN
- File sharing over Internet (Upload & share link)

### Usage

- Build

```shell
cargo build --release --lib
```

- Test

```shell
cargo test
```

### Credits

| Contributor |     #     |
|:-----------:|:---------:|
|    G, Z     | 202191382 |
|    L, S     | 201714987 |
|    G, J     | 202096888 |
|    G, C     | 202194431 |
