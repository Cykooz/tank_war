use noise::{self, Fbm, MultiFractal, NoiseFn, Seedable};

pub struct Landscape {
    width: u32,
    height: u32,
    buffer: Vec<u8>,
    noise: Option<Fbm>,
    amplitude: f64,
    pub dx: i32,
    pub changed: bool,
}

impl Landscape {
    pub fn new(width: u32, height: u32) -> Result<Landscape, String> {
        if width.min(height) == 0 || width.max(height) > ::std::i32::MAX as u32 {
            return Err(format!(
                "'width' and 'height' must be greater than 0 and less or equal than {}",
                std::i32::MAX
            ));
        }

        let stride = width as usize;
        let res_size = stride * height as usize;
        let noise = Fbm::new()
            .set_seed(0)
            .set_octaves(4)
            .set_frequency(2. / f64::from(width));
        Ok(Landscape {
            width,
            height,
            buffer: vec![0; res_size],
            amplitude: f64::from(height) / 2.,
            dx: 0,
            noise: Some(noise),
            changed: true,
        })
    }

    pub fn set_seed(&mut self, seed: u32) {
        let noise = std::mem::replace(&mut self.noise, None);
        self.noise = Some(noise.unwrap().set_seed(seed));
    }

    pub fn seed(&self) -> u32 {
        self.noise.as_ref().unwrap().seed()
    }

    pub fn set_octaves(&mut self, octaves: usize) {
        let noise = std::mem::replace(&mut self.noise, None);
        self.noise = Some(noise.unwrap().set_octaves(octaves));
    }

    pub fn set_frequency(mut self, frequency: f64) {
        let noise = std::mem::replace(&mut self.noise, None);
        self.noise = Some(noise.unwrap().set_frequency(frequency));
    }

    #[inline]
    pub fn size(&self) -> (u32, u32) {
        (self.width, self.height)
    }

    pub fn generate(&mut self) {
        let stride = self.width as usize;
        let y_center: f64 = f64::from(self.height) / 2.;
        let noise = self.noise.as_mut().unwrap();

        for x in 0..self.width as i32 {
            let sx = f64::from(x + self.dx);
            let value = noise.get([sx, 0.]) * self.amplitude;
            let y = (y_center + value).round().max(0.) as usize;
            let y = y.min(self.height as usize);
            let index = y * stride + (x as usize);

            if y > 0 {
                self.buffer
                    .iter_mut()
                    .skip(x as usize)
                    .step_by(stride)
                    .take(y)
                    .for_each(|v| *v = 0);
            }

            self.buffer
                .iter_mut()
                .skip(index)
                .step_by(stride)
                .for_each(|v| *v = 1);
        }
    }

    /// Gets iterator through coordinates of not empty points of landscape.
    #[inline]
    pub fn iter_filled_points(&self) -> impl Iterator<Item = (i32, i32)> + '_ {
        let width = self.width;
        self.buffer.iter().enumerate().filter_map(move |(i, v)| {
            if *v > 0 {
                let x = i as u32 % width;
                let y = i as u32 / width;
                Some((x as _, y as _))
            } else {
                None
            }
        })
    }

    /// Get mutable slice with row of pixels given length
    pub fn get_pixels_line_mut(&mut self, point: (i32, i32), length: u32) -> Option<&mut [u8]> {
        let (x, y) = point;
        if x < 0 || y < 0 || x >= self.width as _ || y >= self.height as _ || length == 0 {
            return None;
        }
        let index = (y * self.width as i32 + x) as usize;
        let length = length.min(self.width - x as u32) as usize;
        Some(&mut self.buffer[index..index + length])
    }

    pub fn is_not_empty(&self, x: i32, y: i32) -> bool {
        if x < 0 || y < 0 || x >= self.width as i32 || y >= self.height as i32 {
            return false;
        }
        let index = (y as u32 * self.width + x as u32) as usize;
        self.buffer[index] > 0
    }

    pub fn to_rgba(&self) -> Vec<u8> {
        let image_size = self.buffer.len() * 4;
        let mut rgba: Vec<u8> = Vec::with_capacity(image_size);
        let buf = unsafe {
            rgba.set_len(image_size);
            rgba.align_to_mut::<u32>().1
        };
        for (&v, d) in self.buffer.iter().zip(buf) {
            *d = if v == 0 { 0 } else { 0xff_cf_bd_00 }
        }

        rgba
    }
}

//impl Draw for Landscape {
//    fn draw(&self, canvas: &mut Canvas<Window>) -> Result<(), String> {
//        canvas.set_draw_color(Color::RGB(0, 189, 207));
//        for point in self.iter_filled_points() {
//            canvas.draw_point(point)?;
//        }
//        Ok(())
//    }
//}
