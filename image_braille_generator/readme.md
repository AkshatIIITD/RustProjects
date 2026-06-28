# Image Braiile Ascii Generator

Convert an image into a braille ascii text terminal output.
Great for adding custom images in fastfetch output

Note: Threshold is optional, an average threshold will be calculated
for the image provided.

Examples to get braille (cat2.jpg used with 40 threshold):

For dev mode:

```bash
cargo run images/cat2.jpg 40
```

For prod (windows example here):

1. create the release exe:

   ```cmd

   cargo build --release

   ```

2. run the exe:

   ```cmd

   .\target\release\image_braille_generator.exe images/cat2.jpg 40

   ```

