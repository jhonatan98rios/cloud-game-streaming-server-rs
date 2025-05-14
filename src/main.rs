use scrap::{Capturer, Display};
use std::io::Write;
use std::process::{Command, Stdio};
use std::thread;
use std::time::{Duration, Instant};

fn main() {
    let display = Display::primary().expect("Couldn't find primary display.");
    let mut capturer = Capturer::new(display).expect("Couldn't begin capture.");

    let width = capturer.width();
    let height = capturer.height();
    let fps = 30;
    let duration_seconds = 5;

    println!("Capturing {}x{} at {} FPS for {} seconds...", width, height, fps, duration_seconds);

    let frame_size = width * height * 3; // 3 bytes per pixel (RGB)

    // Spawn ffmpeg subprocess
    let mut ffmpeg = Command::new("ffmpeg")
        .args([
            "-y",
            "-f", "rawvideo",
            "-pix_fmt", "rgb24",
            "-s", &format!("{}x{}", width, height),
            "-r", &fps.to_string(),
            "-i", "-",
            "-c:v", "libx264",
            "-pix_fmt", "yuv420p",
            "output.mp4",
        ])
        .stdin(Stdio::piped())
        .spawn()
        .expect("Failed to start ffmpeg");

    let mut stdin = ffmpeg.stdin.take().expect("Failed to open ffmpeg stdin");

    let start = Instant::now();
    let frame_duration = Duration::from_secs_f32(1.0 / fps as f32);

    while start.elapsed().as_secs_f32() < duration_seconds as f32 {
        match capturer.frame() {
            Ok(frame) => {
                // Convert from BGRA to RGB
                let mut rgb = Vec::with_capacity(frame_size);
                for i in 0..(width * height) {
                    let i = i * 4;
                    let b = frame[i];
                    let g = frame[i + 1];
                    let r = frame[i + 2];
                    rgb.push(r);
                    rgb.push(g);
                    rgb.push(b);
                }

                // Write to ffmpeg stdin
                if let Err(e) = stdin.write_all(&rgb) {
                    eprintln!("Failed to write frame to ffmpeg: {}", e);
                    break;
                }

                thread::sleep(frame_duration);
            }
            Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                thread::sleep(Duration::from_millis(1));
                continue;
            }
            Err(e) => {
                eprintln!("Capture error: {}", e);
                break;
            }
        }
    }

    // Close stdin so ffmpeg knows we're done
    drop(stdin);

    let status = ffmpeg.wait().expect("Failed to wait on ffmpeg");
    if status.success() {
        println!("Video saved to output.mp4");
    } else {
        eprintln!("ffmpeg exited with status: {:?}", status.code());
    }
}