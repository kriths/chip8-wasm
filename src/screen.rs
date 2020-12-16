use std::cmp;

const SCREEN_WIDTH: usize = 64;
const SCREEN_HEIGHT: usize = 32;
const SCREEN_PIXELS: usize = SCREEN_WIDTH * SCREEN_HEIGHT;

pub struct Screen {
    // TODO is there a better way of storing instead of a bool array
    pixels: [bool; SCREEN_PIXELS],
}

impl Screen {
    pub fn init() -> Self {
        Screen {
            pixels: [false; SCREEN_PIXELS],
        }
    }

    pub fn clear(&mut self) {
        for i in 0..self.pixels.len() {
            self.pixels[i] = false;
        }
    }

    fn set_pixel(&mut self, x: usize, y: usize, state: bool) -> bool {
        let index = y * SCREEN_WIDTH + x;
        let prev = self.pixels[index];
        self.pixels[index] = state;
        state != prev
    }

    pub fn draw_sprite_line(&mut self, x: u8, y: u8, line: u8) -> bool {
        let x_usize = x as usize;
        let y_usize = y as usize;

        let mut any_changes = false;
        let x_max = cmp::min(x_usize + 8, SCREEN_WIDTH);
        for i in 0..x_max-x_usize {
            let state = (line >> (7 - x)) & 1 == 1;
            any_changes |= self.set_pixel(i, y_usize, state);
        }

        any_changes
    }
}
