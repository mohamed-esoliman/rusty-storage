use crate::error::{Result, RustyStorageError};
use ffmpeg_next as ffmpeg;
use ffmpeg::codec;
use ffmpeg::format::input;
use ffmpeg::media::Type;
use ffmpeg::software::scaling::{context::Context as ScalingContext, flag::Flags};
use ffmpeg::util::format::Pixel;
use ffmpeg::codec::decoder::video::Video;
use ffmpeg::Dictionary;
use image::RgbImage;
use std::fs::File;
use std::io::Write;
use std::path::Path;

pub struct Decoder {
    input_path: String,
}

impl Decoder {
    pub fn new(input_path: &str) -> Self {
        Self {
            input_path: input_path.to_string(),
        }
    }

    pub fn decode_file<P: AsRef<Path>>(&self, output_path: P) -> Result<()> {
        // Initialize FFmpeg
        ffmpeg::init().map_err(|e| RustyStorageError::FFmpeg(e.to_string()))?;

        // Open input context
        let input_context = input(&self.input_path)
            .map_err(|e| RustyStorageError::FFmpeg(e.to_string()))?;

        // Find best video stream
        let input = input_context
            .streams()
            .best(Type::Video)
            .ok_or_else(|| RustyStorageError::FFmpeg("No video stream found".to_string()))?;

        let video_stream_index = input.index();

        // Get decoder parameters
        let decoder_params = input_context
            .stream(video_stream_index)
            .ok_or_else(|| RustyStorageError::FFmpeg("Failed to get stream".to_string()))?
            .parameters();

        // Find decoder
        let decoder = codec::decoder::find(decoder_params.codec_id())
            .ok_or_else(|| RustyStorageError::FFmpeg("Decoder not found".to_string()))?;

        // Create decoder
        let mut decoder = codec::decoder::video::Video::open(decoder, Dictionary::new())
            .map_err(|e| RustyStorageError::FFmpeg(e.to_string()))?;

        // Configure decoder
        decoder.set_height(decoder_params.height());
        decoder.set_width(decoder_params.width());
        decoder.set_format(decoder_params.format());

        let mut decoded_data = Vec::new();
        let mut scaler = None;

        // Process frames
        for (stream, packet) in input_context.packets() {
            if stream.index() == video_stream_index {
                // Decode packet
                match decoder.decode(&packet) {
                    Ok(frames) => {
                        for decoded in frames {
                            // Initialize scaler if needed
                            if scaler.is_none() {
                                let context = ScalingContext::get(
                                    decoded.format(),
                                    decoded.width(),
                                    decoded.height(),
                                    Pixel::RGB24,
                                    decoded.width(),
                                    decoded.height(),
                                    Flags::BILINEAR,
                                ).map_err(|e| RustyStorageError::FFmpeg(e.to_string()))?;
                                scaler = Some(context);
                            }

                            // Convert frame to RGB format
                            let mut rgb_frame = ffmpeg::frame::Video::empty();
                            if let Some(ref mut context) = scaler {
                                context.run(&decoded, &mut rgb_frame)
                                    .map_err(|e| RustyStorageError::FFmpeg(e.to_string()))?;
                                let data = self.process_frame(&rgb_frame)?;
                                decoded_data.extend_from_slice(&data);
                            }
                        }
                    },
                    Err(e) => return Err(RustyStorageError::FFmpeg(e.to_string())),
                }
            }
        }

        // Flush decoder
        match decoder.flush() {
            Ok(frames) => {
                for decoded in frames {
                    if let Some(ref mut context) = scaler {
                        let mut rgb_frame = ffmpeg::frame::Video::empty();
                        context.run(&decoded, &mut rgb_frame)
                            .map_err(|e| RustyStorageError::FFmpeg(e.to_string()))?;
                        let data = self.process_frame(&rgb_frame)?;
                        decoded_data.extend_from_slice(&data);
                    }
                }
            },
            Err(e) => return Err(RustyStorageError::FFmpeg(e.to_string())),
        }

        // Remove padding (zeros) from the end
        while let Some(&0) = decoded_data.last() {
            decoded_data.pop();
        }

        // Write to output file
        let mut file = File::create(output_path)?;
        file.write_all(&decoded_data)?;

        Ok(())
    }

    fn process_frame(&self, frame: &ffmpeg::frame::Video) -> Result<Vec<u8>> {
        let width = frame.width();
        let height = frame.height();
        let stride = frame.stride(0);
        let data = frame.data(0);

        // Convert frame data to RGB image
        let mut img = RgbImage::new(width, height);
        for y in 0..height {
            for x in 0..width {
                let offset = (y as usize * stride + x as usize * 3) as usize;
                let r = data[offset];
                let g = data[offset + 1];
                let b = data[offset + 2];
                
                // Since we stored the same value in all channels during encoding,
                // we just take the red channel
                img.put_pixel(x, y, image::Rgb([r, g, b]));
            }
        }

        // Extract data from image
        let mut result = Vec::new();
        for pixel in img.pixels() {
            // We only need one channel since they're all the same
            result.push(pixel[0]);
        }

        Ok(result)
    }
} 