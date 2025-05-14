use anyhow::{Context, Result};
use log::{error, info};
use scrap::{Capturer, Display};
use std::{
    io::Write,
    process::{Child, Command, Stdio},
    thread,
    time::{Duration, Instant},
};

const DURATION_SECONDS: u64 = 10;
const FPS: u32 = 30;

fn main() -> Result<()> {
    env_logger::init();
    info!("🎬 Starting screen and audio capture...");

    let mut capturer = init_capturer()?;
    let (width, height) = (capturer.width(), capturer.height());

    let mut ffmpeg = start_ffmpeg(width, height)?;

    let stdin = ffmpeg.stdin.take().context("Failed to open ffmpeg stdin")?;

    capture_loop(&mut capturer, stdin, width, height)?;

    let status = ffmpeg.wait().context("Failed to wait on ffmpeg")?;

    if status.success() {
        info!("✅ Recording finished successfully: output.mp4");
    } else {
        error!("❌ ffmpeg exited with status: {:?}", status.code());
    }

    Ok(())
}

fn init_capturer() -> Result<Capturer> {
    let display = Display::primary().context("Couldn't find primary display")?;
    let capturer = Capturer::new(display).context("Couldn't begin screen capture")?;
    info!("🖥️ Capturing display: {}x{}", capturer.width(), capturer.height());
    Ok(capturer)
}

fn start_ffmpeg(width: usize, height: usize) -> Result<Child> {
    let audio_device = "Microfone (Steam Streaming Microphone)";

    let ffmpeg = Command::new("ffmpeg")
        .args([
            "-y",
            "-f", "rawvideo",
            "-pix_fmt", "rgb24",
            "-s", &format!("{}x{}", width, height),
            "-r", &FPS.to_string(),
            "-i", "-",
            "-f", "dshow",
            "-i", &format!("audio={}", audio_device),
            "-c:v", "libx264",
            "-pix_fmt", "yuv420p",
            "-c:a", "aac",
            "-t", &DURATION_SECONDS.to_string(),
            // "-shortest",
            "output.mp4",
        ])
        .stdin(Stdio::piped())
        .stdout(Stdio::null())
        .stderr(Stdio::piped())
        .spawn()
        .context("Failed to start ffmpeg")?;

    info!("📽️ ffmpeg process started with PID {}", ffmpeg.id());
    Ok(ffmpeg)
}

fn capture_loop(capturer: &mut Capturer, mut stdin: impl Write, width: usize, height: usize) -> Result<()> {
    let frame_size = width * height * 3;
    let frame_delay = Duration::from_secs_f32(1.0 / FPS as f32);
    let start_time = Instant::now();

    while start_time.elapsed().as_secs() < DURATION_SECONDS {
        match capturer.frame() {
            Ok(frame) => {
                let mut rgb = Vec::with_capacity(frame_size);
                for i in 0..(width * height) {
                    let offset = i * 4;
                    let b = frame[offset];
                    let g = frame[offset + 1];
                    let r = frame[offset + 2];
                    rgb.extend_from_slice(&[r, g, b]);
                }

                if let Err(e) = stdin.write_all(&rgb) {
                    error!("Failed to write frame to ffmpeg: {}", e);
                    break;
                }

                thread::sleep(frame_delay);
            }
            Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                thread::sleep(Duration::from_millis(1));
                continue;
            }
            Err(e) => {
                error!("Capture error: {}", e);
                break;
            }
        }
    }

    drop(stdin); // Finish ffmpeg input
    Ok(())
}
