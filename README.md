# Rusty Storage

Ever wanted unlimited storage without paying a dime? Here's a clever hack: many video platforms offer unlimited video storage, so why not turn your files into videos? Rusty Storage is a crafty Rust-based tool that does exactly that - it transforms any file into a video that you can upload to your favorite video platform, effectively giving you unlimited storage. When you need your files back, just download the video and decode it back to its original form.

## Features

- Encode any file into a video format
- Decode videos back to their original files
- Command-line interface for easy use
- Progress indicators for all operations
- Efficient encoding using H.264 codec
- Support for any file type

## Prerequisites

- Rust (latest stable version)
- FFmpeg installed on your system

## Installation

1. Clone the repository:

```bash
git clone https://github.com/yourusername/rusty-storage.git
cd rusty-storage
```

2. Build the project:

```bash
cargo build --release
```

The binary will be available at `target/release/rusty-storage`.

## Usage

### Encode a file to video

```bash
rusty-storage encode -i path/to/file -o output.mp4
```

### Decode a video back to file

```bash
rusty-storage decode -i video.mp4 -o output_file
```

## How it Works

1. **Encoding Process**:

   - The input file is read as a stream of bytes
   - Each byte is converted into a pixel value in the video
   - The pixels are arranged into frames of 1920x1080 resolution
   - The frames are encoded into an H.264 video at 30fps

2. **Decoding Process**:
   - The video file is read frame by frame
   - Each pixel's value is converted back into a byte
   - The bytes are assembled in order
   - The original file is reconstructed

## Technical Details

- Video Resolution: 1920x1080 (Full HD)
- Frame Rate: 30 FPS
- Codec: H.264
- Each pixel stores one byte of data
- Maximum file size per video: ~2GB (varies by platform)

## Limitations

- Video quality settings may affect data integrity
- Large files will result in longer videos
- Some video platforms may compress uploads, potentially corrupting the data

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

## License

This project is licensed under the MIT License - see the LICENSE file for details.
