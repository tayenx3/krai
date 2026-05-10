<div align="center">
    <img src="icon.png" width=200vw>
    <h1>Krai</h1>
    <img src="https://img.shields.io/badge/license-Apache--2.0-blue">
    <img src="https://img.shields.io/badge/built_with-Rust-orange">
</div>

Krai is a modern, general-purpose low-level programming language built in Rust that aims to be:

- A safer C
- A friendlier Rust
- A more documented Odin

Krai's philosophy is "explicit > clever." It only hurts when you're actually stupid. Here is a quick taste:

```rust
const std = import("std");

$parse_json() Tree ! Error -> {
    # ...
}

$main!() -> {
    # memory allocation
    let buf = std.mem.alloc<u8>(1024);  # uses global allocator
    defer std.mem.free(buf);            # remember to free!
    
    # error handling that scales
    let data = std.io.read_file_to_string("config.json")?;
    defer std.mem.free(data);
    let parsed = parse_json(data) ?? default;
    defer parsed.free();
}
```

## Installing Krai

### 1. Install Cargo (if you haven't)

Go to [rust-lang.org](https://rust-lang.org/) and install the Rust toolchain.

### 2. Install Krai using Cargo

```bash
git clone https://github.com/tayenx3/krai.git
cd krai
cargo install --path . --release
```

## The Krai Book

You can learn Krai right now using [the Krai Book.](./docs/guide.md)
