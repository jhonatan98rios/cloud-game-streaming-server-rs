use scrap::{Capturer, Display};
use std::io::ErrorKind::WouldBlock;
use std::{fs::File, thread, time::Duration};
use image::{ImageBuffer, Rgba};

fn main() {
    // Select primary display
    let display = Display::primary().expect("Couldn't find primary display.");
    let mut capturer = Capturer::new(display).expect("Couldn't begin capture.");

    let (width, height) = (capturer.width(), capturer.height());
    println!("Capturing {}x{}", width, height);

    // Allow time for things to render
    thread::sleep(Duration::from_secs(1));

    // Try capturing a frame
    loop {
        match capturer.frame() {
            Ok(frame) => {
                println!("Captured one frame!");

                // Convert BGRX to RGBA for image crate
                let mut img_buf = ImageBuffer::<Rgba<u8>, Vec<u8>>::new(width as u32, height as u32);
                for (i, pixel) in img_buf.pixels_mut().enumerate() {
                    let i = i * 4;
                    let b = frame[i];
                    let g = frame[i + 1];
                    let r = frame[i + 2];
                    *pixel = Rgba([r, g, b, 255]);
                }

                img_buf
                    .save("screenshot.png")
                    .expect("Failed to save image");
                break;
            }
            Err(ref e) if e.kind() == WouldBlock => {
                // Try again after a short delay
                thread::sleep(Duration::from_millis(10));
                continue;
            }
            Err(e) => panic!("Error: {}", e),
        }
    }
}