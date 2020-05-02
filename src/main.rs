use std::error::Error;
use std::fs::File;
use std::io::BufWriter;
use png::OutputInfo;
use std::path::Path;

struct Frame {
    pixels: Vec<u8>,
}

impl Frame {
    fn load(path: &Path) -> Result<(OutputInfo, Self), Box<dyn Error>> {
        let decoder = png::Decoder::new(File::open(path)?);
        let (info, mut reader) = decoder.read_info()?;
        let mut frame = Frame::new(&info);
        reader.next_frame(&mut frame.pixels).unwrap();
        Ok((info, frame))
    }

    fn save(&self, info: &OutputInfo, path: &Path) -> Result<(), Box<dyn Error>> {
        let ref mut w = BufWriter::new(File::create(path)?);
        let mut encoder = png::Encoder::new(w, info.width, info.height);
        encoder.set_color(info.color_type);
        encoder.set_depth(info.bit_depth);
        let mut writer = encoder.write_header()?;
        writer.write_image_data(&self.pixels)?;
        Ok(())
    }

    fn new(info: &OutputInfo) -> Self {
        Self {
            pixels: vec![0; info.buffer_size()],
        }
    }

    fn pixel_index(&self, info: &OutputInfo, row: usize, col: usize) -> usize {
        row * info.line_size + col * info.color_type.samples()
    }

    fn copy_pixel(&mut self, info: &OutputInfo, src_frame: &Frame, row: usize, col: usize) {
        let dst_index = self.pixel_index(info, row, col);
        let src_index = src_frame.pixel_index(info, row, col);
        // TODO: I'm not sure if this approach works for Indexed colors
        for i in 0..info.color_type.samples() {
            self.pixels[dst_index + i] = src_frame.pixels[src_index + i];
        }
    }

    fn swap_pixels(&mut self, info: &OutputInfo,
                   dst_row: usize, dst_col: usize,
                   src_row: usize, src_col: usize) {
        let dst_index = self.pixel_index(info, dst_row, dst_col);
        let src_index = self.pixel_index(info, src_row, src_col);
        for i in 0..info.color_type.samples() {
            self.pixels.swap(dst_index + i, src_index + i);
        }
    }

    #[allow(dead_code)]
    fn rotate(&mut self, info: &OutputInfo) {
        let w = info.width as usize;
        let h = info.height as usize;
        for row in 0..(h / 2) {
            for col in 0..w {
                self.swap_pixels(info, row, col, h - row - 1, col);
            }
        }
    }

    fn copy_row(&mut self, info: &OutputInfo, frame: &Frame, row: usize) {
        let w = info.width as usize;
        for col in 0..w {
            self.copy_pixel(info, frame, row, col);
        }
    }

    fn copy_col(&mut self, info: &OutputInfo, frame: &Frame, col: usize) {
        let h = info.height as usize;
        for row in 0..h {
            self.copy_pixel(info, frame, row, col);
        }
    }
}

const DISPLACEMENT_STEP: usize = 2;


struct Movie {
    frames: Vec<Frame>,
    info: OutputInfo,
}

impl Movie {
    // TODO: Movie::load doesn't tell where exactly things failed
    fn load(input_folder: &str, frame_count: usize) -> Result<Self, Box<dyn Error>> {
        let mut result = {
            assert!(frame_count > 0);
            let (info, first_frame) = Frame::load(Path::new(&format!("{}/{:04}.png", input_folder, 1)))?;
            let mut result = Self {
                frames: Vec::new(),
                info: info
            };
            result.frames.push(first_frame);
            result
        };

        // TODO: multi-threading?
        for i in 2..=frame_count {
            let (info, frame) = Frame::load(Path::new(&format!("{}/{:04}.png", input_folder, i)))?;
            assert!(info.width == result.info.width);
            assert!(info.height == result.info.height);
            assert!(info.color_type == result.info.color_type);
            assert!(info.bit_depth == result.info.bit_depth);
            assert!(info.line_size == result.info.line_size);
            result.frames.push(frame);
        }

        Ok(result)
    }

    // TODO: we need more interesting displacement algorithms
    fn displace_frame_by_row(&self, index: usize, output_frame: &mut Frame) {
        let h = self.info.height as usize;

        for row in 0..h {
            let displaced_index = index + row / DISPLACEMENT_STEP;
            // TODO: wrap around?
            if displaced_index < self.frames.len() {
                output_frame.copy_row(&self.info, &self.frames[displaced_index], row);
            }
        }
    }

    fn displace_frame_by_col(&self, index: usize, output_frame: &mut Frame) {
        let w = self.info.width as usize;

        for col in 0..w {
            let displaced_index = index + col / DISPLACEMENT_STEP;
            // TODO: wrap around?
            if displaced_index < self.frames.len() {
                output_frame.copy_col(&self.info, &self.frames[displaced_index], col);
            }
        }
    }

    fn displace_frame_by_rowcol(&self, frame_index: usize, output_frame: &mut Frame) {
        let w = self.info.width as usize;
        let h = self.info.height as usize;

        for row in 0..h {
            for col in 0..w {
                let displaced_frame_index = (frame_index + (row + col) / DISPLACEMENT_STEP) % self.frames.len();
                output_frame.copy_pixel(&self.info, &self.frames[displaced_frame_index], row, col);
            }
        }
    }
}

fn main() {
    let input_folder = "./input";
    let output_folder = "./output";
    // TODO: amount of frames should not be hardcoded
    let frame_count = 300;

    println!("Loading frames...");
    let movie = Movie::load(input_folder, frame_count).unwrap();

    assert!(frame_count > 0);
    let mut output_frame = Frame::new(&movie.info);

    std::fs::create_dir_all(output_folder).unwrap();
    // TODO: multi-threading?
    for i in 0..frame_count {
        let output_path = format!("{}/{:04}.png", output_folder, i + 1);
        print!("\rDisplacing frame {} out of {}", i + 1, frame_count);
        movie.displace_frame_by_row(i, &mut output_frame);
        output_frame.save(&movie.info, Path::new(&output_path)).unwrap();
    }
    print!("\n");
}
