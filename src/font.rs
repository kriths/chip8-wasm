const MEMORY_OFFSET: usize = 0;
const LINES_PER_SPRITE: usize = 5;

const FONT_MAP: [u8; LINES_PER_SPRITE * 0x10] = [
    0b11110000, // 0
    0b10010000,
    0b10010000,
    0b10010000,
    0b11110000,

    0b00100000,  // 1
    0b01100000,
    0b00100000,
    0b00100000,
    0b01110000,

    0b11110000, // 2
    0b00010000,
    0b11110000,
    0b10000000,
    0b11110000,

    0b11110000, // 3
    0b00010000,
    0b11110000,
    0b00010000,
    0b11110000,

    0b10010000, // 4
    0b10010000,
    0b11110000,
    0b00010000,
    0b00010000,

    0b11110000, // 5
    0b10000000,
    0b11110000,
    0b00010000,
    0b11110000,

    0b11110000, // 6
    0b10000000,
    0b11110000,
    0b10010000,
    0b11110000,

    0b11110000, // 7
    0b00010000,
    0b00100000,
    0b01000000,
    0b01000000,

    0b11110000, // 8
    0b10010000,
    0b11110000,
    0b10010000,
    0b11110000,

    0b11110000, // 9
    0b10010000,
    0b11110000,
    0b00010000,
    0b11110000,

    0b11110000, // A
    0b10010000,
    0b11110000,
    0b10010000,
    0b10010000,

    0b11100000, // B
    0b10010000,
    0b11100000,
    0b10010000,
    0b11100000,

    0b11110000, // C
    0b10000000,
    0b10000000,
    0b10000000,
    0b11110000,

    0b11100000, // D
    0b10010000,
    0b10010000,
    0b10010000,
    0b11100000,

    0b11110000, // E
    0b10000000,
    0b11110000,
    0b10000000,
    0b11110000,

    0b11110000, // F
    0b10000000,
    0b11110000,
    0b10000000,
    0b10000000,
];

pub fn load_fonts(memory: &mut [u8; 4096]) {
    if MEMORY_OFFSET + FONT_MAP.len() > memory.len() {
        panic!("Not enough memory to load fonts")
    }

    for i in 0..FONT_MAP.len() {
        memory[MEMORY_OFFSET + i] = FONT_MAP[i];
    }
}

pub fn find_font_sprite(letter: u8) -> usize {
    if letter > 0xF {
        eprintln!("Cannot find sprite for letter");
        return MEMORY_OFFSET;
    }

    MEMORY_OFFSET + (letter as usize * LINES_PER_SPRITE)
}
