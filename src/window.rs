extern crate minifb;

pub struct Window {
    pub handle: minifb::Window,
    pub width: usize,
    pub height: usize,
    pub buffer: Vec<u32>, // encoding: 0RGB
}
impl Window {
    pub fn new(scale: usize, width: usize, height: usize) -> Self {
        let mut win_options = minifb::WindowOptions::default();
        win_options.scale = match scale {
            1 => minifb::Scale::X1,
            2 => minifb::Scale::X2,
            4 => minifb::Scale::X4,
            8 => minifb::Scale::X8,
            16 => minifb::Scale::X16,
            32 => minifb::Scale::X32,
            _ => minifb::Scale::FitScreen,
        };

        Self{
            handle: minifb::Window::new(
                "Rotating Cube - Press ESC to exit",
                width,
                height,
                win_options,
            ).unwrap_or_else(|e| {
                panic!("{}", e);
            }),
            width,
            height,
            buffer: vec![0; width * height],
        }
    }


}