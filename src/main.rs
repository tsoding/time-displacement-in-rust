use std::error::Error;
use std::fs::File;
use std::io::BufWriter;
use png::OutputInfo;
use std::path::Path;

struct Frame {
    pixels: Vec<u8>,
    // TODO: each frame storing its info is wasteful
    info: OutputInfo,
}

impl Frame {
    fn load(path: &Path) -> Result<Self, Box<dyn Error>> {
        let decoder = png::Decoder::new(File::open(path)?);
        let (info, mut reader) = decoder.read_info()?;
        let mut frame = Frame::new(&info);
        reader.next_frame(&mut frame.pixels).unwrap();
        Ok(frame)
    }

    fn save(&self, path: &Path) -> Result<(), Box<dyn Error>> {
        let ref mut w = BufWriter::new(File::create(path)?);
        let mut encoder = png::Encoder::new(w, self.info.width, self.info.height);
        encoder.set_color(self.info.color_type);
        encoder.set_depth(self.info.bit_depth);
        let mut writer = encoder.write_header()?;
        writer.write_image_data(&self.pixels)?;
        Ok(())
    }

    fn new(info: &OutputInfo) -> Self {
        Self {
            pixels: vec![0; info.buffer_size()],
            info: OutputInfo { ..*info },
        }
    }

    fn pixel_index(&self, row: usize, col: usize) -> usize {
        row * self.info.line_size + col * self.info.color_type.samples()
    }

    #[allow(dead_code)]
    fn rotate(&mut self) {
        assert!(self.info.color_type == png::ColorType::RGB);
        let w = self.info.width as usize;
        let h = self.info.height as usize;
        for row in 0..(h / 2) {
            for col in 0..w {
                let index = self.pixel_index(row, col);
                let oindex = self.pixel_index(h - row - 1, col);
                self.pixels.swap(index + 0, oindex + 0);
                self.pixels.swap(index + 1, oindex + 1);
                self.pixels.swap(index + 2, oindex + 2);
            }
        }
    }

    fn copy_row(&mut self, frame: &Frame, row: usize) {
        let w = self.info.width as usize;
        for col in 0..w {
            let dst_index = self.pixel_index(row, col);
            let src_index = frame.pixel_index(row, col);
            self.pixels[dst_index + 0] = frame.pixels[src_index + 0];
            self.pixels[dst_index + 1] = frame.pixels[src_index + 1];
            self.pixels[dst_index + 2] = frame.pixels[src_index + 2];
        }
    }

    fn copy_col(&mut self, frame: &Frame, col: usize) {
        let h = self.info.height as usize;
        for row in 0..h {
            let dst_index = self.pixel_index(row, col);
            let src_index = frame.pixel_index(row, col);
            self.pixels[dst_index + 0] = frame.pixels[src_index + 0];
            self.pixels[dst_index + 1] = frame.pixels[src_index + 1];
            self.pixels[dst_index + 2] = frame.pixels[src_index + 2];
        }
    }
}

const DISPLACEMENT_STEP: usize = 2;

// TODO: we need more interesting displacement algorithm
fn displace_frame_by_row(frames: &[Frame], index: usize, output_frame: &mut Frame) {
    let h = frames[index].info.height as usize;

    for row in 0..h {
        let displaced_index = index + row / DISPLACEMENT_STEP;
        if displaced_index < frames.len() {
            output_frame.copy_row(&frames[displaced_index], row);
        }
    }
}

fn displace_frame_by_col(frames: &[Frame], index: usize, output_frame: &mut Frame) {
    let w = frames[index].info.width as usize;

    for col in 0..w {
        let displaced_index = index + col / DISPLACEMENT_STEP;
        if displaced_index < frames.len() {
            output_frame.copy_col(&frames[displaced_index], col);
        }
    }
}

fn displace_frame_by_rowcol(frames: &[Frame],
                            frame_index: usize,
                            output_frame: &mut Frame) {
    let w = frames[frame_index].info.width as usize;
    let h = frames[frame_index].info.height as usize;

    for row in 0..h {
        for col in 0..w {
            let displaced_frame_index = (frame_index + (row + col) / DISPLACEMENT_STEP) % frames.len();
            let dst_pixel_index = output_frame.pixel_index(row, col);
            let src_pixel_index = frames[displaced_frame_index].pixel_index(row, col);
            output_frame.pixels[dst_pixel_index + 0] =
                frames[displaced_frame_index].pixels[src_pixel_index + 0];
            output_frame.pixels[dst_pixel_index + 1] =
                frames[displaced_frame_index].pixels[src_pixel_index + 1];
            output_frame.pixels[dst_pixel_index + 2] =
                frames[displaced_frame_index].pixels[src_pixel_index + 2];
        }
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let input_folder = "./input";
    let output_folder = "./output";
    let frame_count = 300;
    let mut frames = Vec::<Frame>::new();

    println!("Loading frames...");
    for i in 1..=frame_count {
        let input_path = format!("{}/{:04}.png", input_folder, i);
        frames.push(Frame::load(Path::new(&input_path))?);
    }

    assert!(frame_count > 0);
    let mut output_frame = Frame::new(&frames[0].info);

    std::fs::create_dir_all(output_folder)?;
    for i in 0..frame_count {
        let output_path = format!("{}/{:04}.png", output_folder, i + 1);
        print!("\rDisplacing frame {} out of {}", i + 1, frame_count);
        displace_frame_by_row(&frames, i, &mut output_frame);
        output_frame.save(Path::new(&output_path))?;
    }
    print!("\n");

    Ok(())
}
