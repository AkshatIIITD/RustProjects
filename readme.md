# RustProjects

Small Rust projects I build to learn systems programming and networking.
Each folder is an independent Cargo project with its own README.

| Project | What it does | Concepts practiced | Run |
|---------|--------------|--------------------|-----|
| [network_packet_sniffer](network_packet_sniffer/) | Live packet sniffer for Linux. Reads raw Ethernet frames, parses IPv4, TCP and UDP headers by hand, and prints plaintext HTTP payloads and UDP flows | Raw sockets (`AF_PACKET`), Ethernet/IPv4/TCP/UDP header layout, byte order, `unsafe` and FFI | `cargo build --release` then `sudo ./target/release/network_packet_sniffer` |
| [image_braille_generator](image_braille_generator/) | Converts an image to Braille-character art for the terminal | image decoding and resizing, grayscale conversion, thresholding, bit manipulation, Unicode | `cargo run --release -- <image> [threshold]` |

## Requirements

- Rust 1.85 or newer via [rustup](https://rustup.rs/) (the sniffer uses the 2024 edition)
- Linux for the packet sniffer, which uses Linux-only raw sockets
- The braille generator runs on Windows and Linux