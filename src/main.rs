use std::error::Error;
use std::fs::File;
use std::io::BufWriter;
use png::OutputInfo;
use std::path::Path;

struct Frame {
    pixels: Vec<u8>,
    info: OutputInfo,
}

impl Frame {
    fn load(path: &Path) -> Result<Self, Box<dyn Error>> {
        let decoder = png::Decoder::new(File::open(path)?);
        let (info, mut reader) = decoder.read_info()?;
        let mut frame = Frame::new(info);
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

    fn new(info: OutputInfo) -> Self {
        Self {
            pixels: vec![0; info.buffer_size()],
            info: info,
        }
    }

    fn pixel_index(&self, row: usize, col: usize) -> usize {
        row * self.info.line_size + col * self.info.color_type.samples()
    }

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
}

fn main() -> Result<(), Box<dyn Error>> {
    // format!("{:04}", 42);
    let input_folder = "./input";
    let output_folder = "./output";
    let frame_count = 300;
    let mut frames = Vec::<Frame>::new();

    for i in 1..=frame_count {
        let input_path = format!("{}/{:04}.png", input_folder, i);
        frames.push(Frame::load(Path::new(&input_path))?);
        println!("Loading {}", input_path);
    }

    for i in 0..frame_count {
        frames[i].rotate();
        println!("Rotating frame {}", i);
    }

    std::fs::create_dir_all(output_folder)?;
    for i in 0..frame_count {
        let output_path = format!("{}/{:04}.png", output_folder, i + 1);
        frames[i].save(Path::new(&output_path))?;
        println!("Saving {}", output_path);
    }

    Ok(())
}
