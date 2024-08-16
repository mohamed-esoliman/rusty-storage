use crate::error::{Result, RustyStorageError};
use image::{ImageBuffer, Rgb};
use std::fs::File;
use std::io::Read;
use std::path::Path;
use ffmpeg_next as ffmpeg;
use ffmpeg::format::Pixel;
use ffmpeg::frame;
use ffmpeg::codec;
use ffmpeg::Dictionary;

const PIXELS_PER_BYTE: usize = 1;
const FRAME_WIDTH: u32 = 1920;
const FRAME_HEIGHT: u32 = 1080;
const FPS: i32 = 30;

pub struct Encoder {
    output_path: String,
}

impl Encoder {
    pub fn new(output_path: &str) -> Self {
        Self {
            output_path: output_path.to_string(),
        }
    }

    pub fn encode_file<P: AsRef<Path>>(&self, input_path: P) -> Result<()> {
        // Read input file
        let mut file = File::open(input_path)?;
        let mut buffer = Vec::new();
        file.read_to_end(&mut buffer)?;

        // Initialize FFmpeg
        ffmpeg::init().map_err(|e| RustyStorageError::FFmpeg(e.to_string()))?;

        // Create output context
        let mut output_context = ffmpeg::format::output(&self.output_path)
            .map_err(|e| RustyStorageError::FFmpeg(e.to_string()))?;

        // Find H264 encoder
        let encoder = codec::encoder::find(codec::Id::H264)
            .ok_or_else(|| RustyStorageError::FFmpeg("H264 encoder not found".to_string()))?;

        // Create video stream
        let mut stream = output_context
            .add_stream(encoder)
            .map_err(|e| RustyStorageError::FFmpeg(e.to_string()))?;

        // Configure stream parameters
        {
            let mut params = stream.parameters();
            params.set_width(FRAME_WIDTH as i32);
            params.set_height(FRAME_HEIGHT as i32);
            params.set_format(Pixel::RGB24);
            params.set_bit_rate(2_000_000);
            
            // Set time base and frame rate
            let time_base = ffmpeg::util::rational::Rational::new(1, FPS);
            stream.set_time_base(time_base);
            
            let frame_rate = ffmpeg::util::rational::Rational::new(FPS, 1);
            stream.set_rate(frame_rate);
        }

        // Create encoder
        let mut encoder = codec::encoder::video::Video::open(encoder, Dictionary::new())
            .map_err(|e| RustyStorageError::FFmpeg(e.to_string()))?;

        // Configure encoder
        encoder.set_height(FRAME_HEIGHT as i32);
        encoder.set_width(FRAME_WIDTH as i32);
        encoder.set_format(Pixel::RGB24);
        encoder.set_rate(ffmpeg::util::rational::Rational::new(FPS, 1));
        encoder.set_time_base(ffmpeg::util::rational::Rational::new(1, FPS));

        // Write header
        output_context
            .write_header()
            .map_err(|e| RustyStorageError::FFmpeg(e.to_string()))?;

        // Create frames from data
        let frames = self.create_frames_from_data(&buffer)?;

        // Encode and write frames
        for (i, frame_data) in frames.iter().enumerate() {
            let mut frame = frame::Video::new(
                Pixel::RGB24,
                FRAME_WIDTH,
                FRAME_HEIGHT,
            );
            
            // Set frame timing
            frame.set_pts(Some(i as i64));
            frame.set_time_base(ffmpeg::util::rational::Rational::new(1, FPS));

            // Copy frame data
            frame.data_mut(0).copy_from_slice(frame_data);

            // Encode frame
            match encoder.encode(&frame) {
                Ok(packets) => {
                    for mut packet in packets {
                        packet.set_position(i as i64);
                        packet.set_stream(stream.index());
                        packet.rescale_ts(frame.time_base(), stream.time_base());
                        
                        output_context
                            .write(&packet)
                            .map_err(|e| RustyStorageError::FFmpeg(e.to_string()))?;
                    }
                },
                Err(e) => return Err(RustyStorageError::FFmpeg(e.to_string())),
            }
        }

        // Flush encoder
        match encoder.flush() {
            Ok(packets) => {
                for mut packet in packets {
                    packet.set_stream(stream.index());
                    output_context
                        .write(&packet)
                        .map_err(|e| RustyStorageError::FFmpeg(e.to_string()))?;
                }
            },
            Err(e) => return Err(RustyStorageError::FFmpeg(e.to_string())),
        }

        // Write trailer
        output_context
            .write_trailer()
            .map_err(|e| RustyStorageError::FFmpeg(e.to_string()))?;

        Ok(())
    }

    fn create_frames_from_data(&self, data: &[u8]) -> Result<Vec<Vec<u8>>> {
        let pixels_per_frame = (FRAME_WIDTH * FRAME_HEIGHT) as usize;
        let bytes_per_frame = pixels_per_frame * PIXELS_PER_BYTE;
        let mut frames = Vec::new();

        for chunk in data.chunks(bytes_per_frame) {
            let mut frame = ImageBuffer::new(FRAME_WIDTH, FRAME_HEIGHT);
            
            for (i, byte) in chunk.iter().enumerate() {
                let x = (i % FRAME_WIDTH as usize) as u32;
                let y = (i / FRAME_WIDTH as usize) as u32;
                
                if y < FRAME_HEIGHT {
                    frame.put_pixel(x, y, Rgb([*byte, *byte, *byte]));
                }
            }

            // Fill remaining pixels if chunk is smaller than frame size
            if chunk.len() < bytes_per_frame {
                for i in chunk.len()..bytes_per_frame {
                    let x = (i % FRAME_WIDTH as usize) as u32;
                    let y = (i / FRAME_WIDTH as usize) as u32;
                    frame.put_pixel(x, y, Rgb([0, 0, 0]));
                }
            }

            frames.push(frame.into_raw());
        }

        Ok(frames)
    }
} 