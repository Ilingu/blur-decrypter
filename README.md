# Blur decrypter

#### ➡️ Program to crack/decrypt blurred text 🌫 (previously blurred by the same program)

## 🥅 Purpose

- Having Fun 🙂
- Implementing a basic blur alogrithm 🫥
- Trying to come up with my own ideas to unblur the text 💀

> As always this is a side project

## Limitations

It doesn't work very well, it work well up to a certain level of blurring, which happens to be quite low (𝞼~3).

This project also learn me that it's pratically impossible to unblur a image when you have 0 information about it,
here it works because I know the font, font_size, colors... (and like I said, it works up to a certain limit)

That was a fun and quick project to make but it has have 0 use case.

## Usage

If you want to test it go check the linux binary in the **release page**.

or build it yourself,

```bash
cargo build --release # should create a binary for your os in './target/release/blur-decrypter'
```

then:

```bash
blur-decrypter --help
```

## 🧰 Made with

1. **Elegance** ✅
2. `RUST` ✨🦀
3. go see [Cargo.toml](/Cargo.toml)
