use scrap::{Capturer, Display};
use std::{
    fs::{self},
    io::ErrorKind::WouldBlock,
    thread,
    time::{Duration, Instant},
    process::Command,
};
use image::{ImageBuffer, Rgba};

fn main() {
    let display = Display::primary().expect("Couldn't find primary display.");
    let mut capturer = Capturer::new(display).expect("Couldn't begin capture.");

    let (width, height) = (capturer.width(), capturer.height());
    println!("Capturing video at resolution {}x{}", width, height);

    // Create output folder for frames
    let _ = fs::create_dir_all("frames");

    let start = Instant::now();
    let mut frame_count = 0;
    let target_duration = Duration::from_secs(3); // 10 seconds
    let fps = 30;
    let frame_delay = Duration::from_millis(1000 / fps);

    while start.elapsed() < target_duration {
        match capturer.frame() {
            Ok(frame) => {
                let mut img_buf = ImageBuffer::<Rgba<u8>, Vec<u8>>::new(width as u32, height as u32);
                for (i, pixel) in img_buf.pixels_mut().enumerate() {
                    let i = i * 4;
                    let b = frame[i];
                    let g = frame[i + 1];
                    let r = frame[i + 2];
                    *pixel = Rgba([r, g, b, 255]);
                }

                let filename = format!("frames/frame_{:05}.bmp", frame_count);
                img_buf.save(&filename).expect("Failed to save frame");
                frame_count += 1;

                thread::sleep(frame_delay);
            }
            Err(ref e) if e.kind() == WouldBlock => {
                thread::sleep(Duration::from_millis(1));
                continue;
            }
            Err(e) => panic!("Capture error: {}", e),
        }
    }

    println!("Finished capturing {} frames. Encoding video...", frame_count);

    // Call ffmpeg to generate video
    let output = Command::new("ffmpeg")
        .args([
            "-y", // Overwrite output
            "-framerate", &fps.to_string(),
            "-i", "frames/frame_%05d.bmp",
            "-c:v", "libx264",
            "-pix_fmt", "yuv420p",
            "output.mp4",
        ])
        .output()
        .expect("Failed to execute ffmpeg");

    if output.status.success() {
        println!("Video saved as output.mp4");
    } else {
        eprintln!(
            "ffmpeg failed:\n{}",
            String::from_utf8_lossy(&output.stderr)
        );
    }

    // Optional: Clean up frames
    let _ = fs::remove_dir_all("frames");
}