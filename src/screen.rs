use std::cmp;

const SCREEN_WIDTH: usize = 64;
const SCREEN_HEIGHT: usize = 32;
const SCREEN_PIXELS: usize = SCREEN_WIDTH * SCREEN_HEIGHT;

pub struct Screen {
    // TODO is there a better way of storing instead of a bool array
    pixels: [bool; SCREEN_PIXELS],
}

impl Screen {
    pub fn new() -> Self {
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
            let state = (line >> (7 - i)) & 1 == 1;
            any_changes |= self.set_pixel(i + x_usize, y_usize, state);
        }

        any_changes
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn clear() {
        let mut screen = Screen::new();
        screen.pixels[0] = true;
        screen.pixels[100] = true;
        screen.pixels[123] = true;
        screen.pixels[screen.pixels.len() - 1] = true;

        screen.clear();
        for i in 0..screen.pixels.len() {
            assert_eq!(false, screen.pixels[i]);
        }
    }

    #[test]
    fn set_pixel() {
        let mut screen = Screen::new();
        assert_eq!(false, screen.pixels[10]);

        let mut changed = screen.set_pixel(10, 0, false);
        assert_eq!(false, screen.pixels[10]);
        assert_eq!(false, changed);

        changed = screen.set_pixel(10, 0, true);
        assert_eq!(true, screen.pixels[10]);
        assert_eq!(true, changed);

        changed = screen.set_pixel(10, 0, true);
        assert_eq!(true, screen.pixels[10]);
        assert_eq!(false, changed);
    }

    #[test]
    fn draw_sprite_line() {
        let line = 0b00111100;
        let mut screen = Screen::new();
        let mut changed = screen.draw_sprite_line(10, 0, line);

        assert_eq!(false, screen.pixels[10]);
        assert_eq!(false, screen.pixels[11]);
        assert_eq!(true, screen.pixels[12]);
        assert_eq!(true, screen.pixels[13]);
        assert_eq!(true, screen.pixels[14]);
        assert_eq!(true, screen.pixels[15]);
        assert_eq!(false, screen.pixels[16]);
        assert_eq!(false, screen.pixels[17]);
        assert_eq!(true, changed);

        changed = screen.draw_sprite_line(10, 0, line);
        assert_eq!(false, changed);
    }
}
