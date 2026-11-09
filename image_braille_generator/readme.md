# Image to Braille generator

Converts an image into Braille-character text art for the terminal. Handy for custom
logos in `fastfetch`.

[TODO: add a screenshot showing the original image next to the Braille output]

## Usage

```bash
cargo run --release -- images/cat2.jpg 40
```

| Argument | Meaning |
|----------|---------|
| `<image_path>` | Path to the image. Common formats such as JPEG and PNG work, via the [`image`](https://crates.io/crates/image) crate. |
| `[threshold]` | Brightness cutoff from 0 to 255. Optional. If omitted, the image's average brightness is used. |

The art goes to **stdout** and diagnostics go to **stderr**, so you can save it directly:

```bash
cargo run --release -- images/cat2.jpg 40 > output.txt
```

To build an executable:

```bash
cargo build --release
./target/release/image_braille_generator images/cat2.jpg 40       # Linux
.\target\release\image_braille_generator.exe images/cat2.jpg 40   # Windows
```

Requires Rust 1.85 or newer (2024 edition). Your terminal font needs Braille glyphs.

## Using it with fastfetch

[TODO: add the exact command you use, for example `fastfetch --logo output.txt`.
Check `fastfetch --help logo` for your version.]

## How it works

1. **Load** the image with the `image` crate.
2. **Resize** to 60 characters wide, which is 120 pixels, with the height scaled to
   keep the aspect ratio and rounded to a multiple of 4.
3. **Grayscale** every pixel using the luma formula `0.299 R + 0.587 G + 0.114 B`.
4. **Threshold**: use the value you passed, or the average brightness of the image.
5. **Convert**: slide a 2x4 pixel window over the image. Each pixel brighter than the
   threshold turns on one dot. The 8 dots form a bit pattern that is added to the
   Unicode Braille base character `U+2800`.

The bit for each dot position:

```
 dot 1 (0x01)   dot 4 (0x08)
 dot 2 (0x02)   dot 5 (0x10)
 dot 3 (0x04)   dot 6 (0x20)
 dot 7 (0x40)   dot 8 (0x80)
```

Bright pixels become dots, so the result looks right on a dark terminal. On a light
terminal the image appears inverted.

## Limitations

- The output width is fixed at 60 characters (`target_char_width` in `main.rs`).
- Images are resized with nearest-neighbor, which is fast but can look blocky.
- Very wide panoramas can round to a height of 0 and fail.
- Transparency is ignored.