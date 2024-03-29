## 👋 Hello hiring manager!

This project (libairx) is part of AirX, the cross-platform text and file sharing system.

`libairx` is a Rust library that provides AirX with LAN discovery and LAN data transmission.
You can find other AirX-related projects here:

- [AirX Demo Video](https://hatsune-miku.github.io/airx.html)
- [Windows Client (WinUI 3)](https://github.com/hatsune-miku/AirX-win)
- [Android Client (React Native)](https://github.com/hatsune-miku/airx4a)
- [macOS Client (SwiftUI)](https://github.com/Lsjy44/airX_mac)
- [Netdisk Frontend (Vue.js)](https://github.com/hatsune-miku/airx-cloud)
- [Backend (SpringBoot)](https://github.com/hatsune-miku/airx-backend)

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

- Build Native

```shell
cargo build --release --lib
```

- Build Android

Install Android NDK 25.1.8937393 and SDK API 33 (Tiramisu)

Edit `Makefile` to adjust your paths

```shell
make
make install-android
```

### Credits

| Contributor |     #     |
|:-----------:|:---------:|
|    G, Z     | 202191382 |
|    L, S     | 201714987 |
|    G, J     | 202096888 |
|    G, C     | 202194431 |
