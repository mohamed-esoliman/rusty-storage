use std::process::Command;
use std::path::Path;

pub fn file_to_video(input_path: &str, output_path: &str) {
    let status = Command::new("ffmpeg")
        .args(&[
            "-loop", "1",
            "-i", input_path,
            "-c:v", "libx264",
            "-t", "10",
            "-pix_fmt", "yuv420p",
            output_path,
        ])
        .status()
        .expect("Failed to execute ffmpeg command");

    if !status.success() {
        panic!("Failed to convert file to video");
    }
}