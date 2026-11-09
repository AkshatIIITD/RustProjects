use std::{error, char};
use image::{GenericImageView, ImageReader};

fn main() -> Result<(), Box<dyn error::Error>> {
    
    // Step 1: Fetching Image
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: {} <image_path> [threshold]", args[0]);
        std::process::exit(1);
    }
    let image_path = &args[1];
    let img = ImageReader::open(image_path)?.decode()?;
    let (original_width, original_height) = img.dimensions();


    // Step 2: Resizing the image such that my output looks clean :)
    let target_char_width = 60; // Character width of the terminal output
    let target_pixel_width = target_char_width * 2;
    let ratio = original_height as f64 / original_width as f64;
    let raw_height = target_pixel_width as f64 * ratio;
    let target_pixel_height = (raw_height / 4.0).round() as u32 * 4;
    let resized_img = img.resize_exact(
        target_pixel_width, 
        target_pixel_height, 
        image::imageops::FilterType::Nearest
    );


    // Step 3: Grayscaling the image
    let flat_color_vec: Vec<u8> = resized_img.to_rgb8().into_raw();
    let mut grayscale_vec: Vec<u8> = Vec::with_capacity((target_pixel_width * target_pixel_height) as usize);
    for chunk in flat_color_vec.chunks_exact(3) {
        let r = chunk[0] as f64;
        let g = chunk[1] as f64;
        let b = chunk[2] as f64;
        let luma = (0.299 * r) + (0.587 * g) + (0.114 * b);
        grayscale_vec.push(luma as u8);
    }


    // Step 4: Finding threshold if not given
    let threshold: u8 = if args.len() >= 3 {
        // If the user provided a 3rd argument, try to parse it as a u8
        args[2].parse().expect("Threshold must be a valid number between 0 and 255")
    } else {
        // Dynamic Average Calculation
        eprintln!("No threshold provided");
        let sum: u64 = grayscale_vec.iter().map(|&pixel| pixel as u64).sum();
        let average = sum / (grayscale_vec.len() as u64);
        eprintln!("Calculated average brightness: {}", average);
        average as u8
    };


    // Step 5: 2x4 Window sliding through the grayscale image vector for keeping dots
    for y in (0..target_pixel_height).step_by(4) {
        for x in (0..target_pixel_width).step_by(2) {
            let mut dots = [0u8; 8];
            // Row 1
            dots[0] = if grayscale_vec[((y + 0) * target_pixel_width + (x + 0)) as usize] > threshold { 1 } else { 0 };
            dots[1] = if grayscale_vec[((y + 0) * target_pixel_width + (x + 1)) as usize] > threshold { 1 } else { 0 };
            // Row 2
            dots[2] = if grayscale_vec[((y + 1) * target_pixel_width + (x + 0)) as usize] > threshold { 1 } else { 0 };
            dots[3] = if grayscale_vec[((y + 1) * target_pixel_width + (x + 1)) as usize] > threshold { 1 } else { 0 };
            // Row 3
            dots[4] = if grayscale_vec[((y + 2) * target_pixel_width + (x + 0)) as usize] > threshold { 1 } else { 0 };
            dots[5] = if grayscale_vec[((y + 2) * target_pixel_width + (x + 1)) as usize] > threshold { 1 } else { 0 };
            // Row 4
            dots[6] = if grayscale_vec[((y + 3) * target_pixel_width + (x + 0)) as usize] > threshold { 1 } else { 0 };
            dots[7] = if grayscale_vec[((y + 3) * target_pixel_width + (x + 1)) as usize] > threshold { 1 } else { 0 };
            let mut braille_offset = 0;
            braille_offset |= dots[0] << 0; // Top-left (Bit 0)
            braille_offset |= dots[1] << 3; // Top-right (Bit 3)
            braille_offset |= dots[2] << 1; // Mid-top-left (Bit 1)
            braille_offset |= dots[3] << 4; // Mid-top-right (Bit 4)
            braille_offset |= dots[4] << 2; // Mid-bot-left (Bit 2)
            braille_offset |= dots[5] << 5; // Mid-bot-right (Bit 5)
            braille_offset |= dots[6] << 6; // Bot-left (Bit 6)
            braille_offset |= dots[7] << 7; // Bot-right (Bit 7)
            // Cast the hex base to u32, add our offset, and convert to a Rust char
            if let Some(braille_char) = char::from_u32(0x2800 + braille_offset as u32) {
                print!("{}", braille_char);
            } else {
                print!(" "); // Fallback if something goes wildly wrong
            }
        }
        println!();
    }
    Ok(())
}
