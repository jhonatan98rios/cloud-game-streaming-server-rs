use scrap::{Capturer, Display};
use std::{
    io::Write,
    process::{Command, Stdio},
    thread,
    time::{Duration, Instant},
};

fn main() {
    let display = Display::primary().expect("Couldn't find primary display.");
    let mut capturer = Capturer::new(display).expect("Couldn't begin capture.");
    let (width, height) = (capturer.width(), capturer.height());

    let fps = 30;
    let duration_seconds = 10;
    let frame_size = width * height * 3;

    println!("Capturing screen + audio on Windows for {} seconds...", duration_seconds);

    // Replace with your actual device from `ffmpeg -list_devices true -f dshow -i dummy`
    let audio_device = "Microfone (Steam Streaming Microphone)";

    // Spawn ffmpeg
    let mut ffmpeg = Command::new("ffmpeg")
        .args([
            "-y",
            "-f", "rawvideo",
            "-pix_fmt", "rgb24",
            "-s", &format!("{}x{}", width, height),
            "-r", &fps.to_string(),
            "-i", "-",

            // Audio input via dshow
            "-f", "dshow",
            "-i", &format!("audio={}", audio_device),

            // Output
            "-c:v", "libx264",
            "-pix_fmt", "yuv420p",
            "-c:a", "aac",
            "-shortest", // Stop when shortest input ends
            "output.mp4",
        ])
        .stdin(Stdio::piped())
        .spawn()
        .expect("Failed to start ffmpeg");

    let mut stdin = ffmpeg.stdin.take().expect("Failed to open ffmpeg stdin");

    // Start writing frames
    let start = Instant::now();
    let frame_duration = Duration::from_secs_f32(1.0 / fps as f32);

    while start.elapsed().as_secs_f32() < duration_seconds as f32 {
        match capturer.frame() {
            Ok(frame) => {
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

                if let Err(e) = stdin.write_all(&rgb) {
                    eprintln!("Error writing to ffmpeg: {}", e);
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

    drop(stdin); // Close ffmpeg stdin so it finalizes
    let status = ffmpeg.wait().expect("Failed to wait on ffmpeg");

    if status.success() {
        println!("✅ Recording complete: output.mp4");
    } else {
        eprintln!("ffmpeg exited with error: {:?}", status.code());
    }
}
