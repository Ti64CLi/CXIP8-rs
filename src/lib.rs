use rand::random;

pub const A: usize = 0xA;
pub const B: usize = 0xB;
pub const C: usize = 0xC;
pub const D: usize = 0xD;
pub const E: usize = 0xE;
pub const F: usize = 0xF;

pub const SCREEN_WIDTH_C8: usize = 64;
pub const SCREEN_HEIGHT_C8: usize = 32;

const RAM_SIZE: usize = 4096;
const NUM_REGS: usize = 16;
const NUM_KEYS: usize = 16;
const START_ADDRESS: u16 = 0x200;

const FONTSET_SIZE: usize = 80;
const FONTSET: [u8; FONTSET_SIZE] = [
    0xF0, 0x90, 0x90, 0x90, 0xF0, // 0
    0x20, 0x60, 0x20, 0x20, 0x70, // 1
    0xF0, 0x10, 0xF0, 0x80, 0xF0, // 2
    0xF0, 0x10, 0xF0, 0x10, 0xF0, // 3
    0x90, 0x90, 0xF0, 0x10, 0x10, // 4
    0xF0, 0x80, 0xF0, 0x10, 0xF0, // 5
    0xF0, 0x80, 0xF0, 0x90, 0xF0, // 6
    0xF0, 0x10, 0x20, 0x40, 0x40, // 7
    0xF0, 0x90, 0xF0, 0x90, 0xF0, // 8
    0xF0, 0x90, 0xF0, 0x10, 0xF0, // 9
    0xF0, 0x90, 0xF0, 0x90, 0x90, // A
    0xE0, 0x90, 0xE0, 0x90, 0xE0, // B
    0xF0, 0x80, 0x80, 0x80, 0xF0, // C
    0xE0, 0x90, 0x90, 0x90, 0xE0, // D
    0xF0, 0x80, 0xF0, 0x80, 0xF0, // E
    0xF0, 0x80, 0xF0, 0x80, 0x80, // F
];

pub enum C8Type {
    CHIP8,
    SUPER,
    CHIP8C,
} // for future improvments

struct Screen {
    buffer: [bool; SCREEN_WIDTH_C8 * SCREEN_HEIGHT_C8],
}

struct Keyboard {
    keys: [bool; NUM_KEYS],
}

pub enum Endianness {
    LITTLE,
    BIG,
}

struct Memory {
    memory: [u8; RAM_SIZE],
    endianness: Endianness,
}

pub struct Chip8 {
    screen: Screen,
    v: [u8; NUM_REGS],
    pc: u16,
    i: u16,
    stack: Vec<u16>,
    memory: Memory,
    keyboard: Keyboard,
    delay_timer: u8,
    sound_timer: u8,
}

impl Screen {
    pub fn new() -> Self {
        Screen {
            buffer: [false; SCREEN_WIDTH_C8 * SCREEN_HEIGHT_C8],
        }
    }
    // Clears the screen
    pub fn clear(&mut self) {
        self.buffer = [false; SCREEN_WIDTH_C8 * SCREEN_HEIGHT_C8];
    }

    // Exposes screen
    pub fn get_screen(&self) -> &[bool] {
        &self.buffer
    }

    pub fn set_pixel(&mut self, x: usize, y: usize, flag: bool) {
        self.buffer[x + y * SCREEN_WIDTH_C8] = flag;
    }

    pub fn get_pixel(&self, x: usize, y: usize) -> bool {
        self.buffer[x + y * SCREEN_WIDTH_C8]
    }
}

impl Keyboard {
    pub fn new() -> Self {
        Keyboard {
            keys: [false; NUM_KEYS],
        }
    }

    pub fn reset(&mut self) {
        self.keys = [false; NUM_KEYS];
    }

    pub fn is_pressed(&self, key: u8) -> bool {
        self.keys[key as usize]
    }

    pub fn keypress(&mut self, key: usize, pressed: bool) {
        self.keys[key] = pressed;
    }
}

impl Memory {
    pub fn new() -> Self {
        Memory {
            memory: [0; RAM_SIZE],
            endianness: Endianness::BIG,
        }
    }

    pub fn reset(&mut self) {
        self.memory = [0; RAM_SIZE];
        self.endianness = Endianness::BIG;
    }

    pub fn set_endianness(&mut self, endianness: Endianness) {
        self.endianness = endianness;
    }

    pub fn write_u8(&mut self, address: u16, value: u8) {
        self.memory[address as usize] = value;
    }

    pub fn _write_u16(&mut self, address: u16, value: u16) {
        let v_lower = (value & 0xFF) as u8;
        let v_upper = ((value & 0xFF00) >> 8) as u8;
        let addr = address as usize;
        match self.endianness {
            Endianness::LITTLE => {
                self.memory[addr] = v_lower;
                self.memory[addr + 1] = v_upper;
            }
            Endianness::BIG => {
                self.memory[addr] = v_upper;
                self.memory[addr + 1] = v_lower;
            }
        }
    }

    pub fn read_u8(&self, address: u16) -> u8 {
        self.memory[address as usize]
    }

    pub fn read_u16(&self, address: u16) -> u16 {
        let v_addr = self.memory[address as usize] as u16;
        let v_addr1 = self.memory[(address + 1) as usize] as u16;

        match self.endianness {
            Endianness::LITTLE => (v_addr1 << 8) + v_addr,
            Endianness::BIG => (v_addr << 8) + v_addr1,
        }
    }

    pub fn write_data(&mut self, data: &[u8], address: u16) {
        let addr = address as usize;
        self.memory[addr..(addr + data.len())].copy_from_slice(data);
    }

    pub fn _read_data(&self, buffer: &mut [u8], address: u16, size: usize) {
        let total_size = if size > buffer.len() {
            buffer.len()
        } else {
            size
        };
        let addr = address as usize;

        buffer[..total_size].copy_from_slice(&self.memory[addr..(addr + total_size)]);
    }
}

impl Chip8 {
    pub fn new() -> Self {
        let mut chip8 = Chip8 {
            screen: Screen::new(),
            v: [0; NUM_REGS],
            pc: START_ADDRESS,
            i: 0,
            stack: vec![],
            memory: Memory::new(),
            keyboard: Keyboard::new(),
            delay_timer: 0,
            sound_timer: 0,
        };

        chip8.memory.write_data(&FONTSET, 0);

        chip8
    }

    pub fn reset(&mut self) {
        self.screen.clear();
        self.v = [0; NUM_REGS];
        self.pc = START_ADDRESS;
        self.i = 0;
        self.stack = vec![];
        self.memory.reset();
        self.keyboard.reset();
        self.delay_timer = 0;
        self.sound_timer = 0;

        self.memory.write_data(&FONTSET, 0);
    }

    pub fn set_endianness(&mut self, endianness: Endianness) {
        self.memory.set_endianness(endianness);
    }

    fn fetch_opcode(&mut self) -> u16 {
        self.pc += 2;
        self.memory.read_u16(self.pc - 2)
    }

    fn decode_opcode(&mut self, opcode: u16) {
        let digit1 = ((opcode & 0xF000) >> 12) as usize;
        let x = ((opcode & 0xF00) >> 8) as usize;
        let y = ((opcode & 0xF0) >> 4) as usize;
        let n = (opcode & 0xF) as usize;

        let nnn = opcode & 0xFFF;
        let nn = (opcode & 0xFF) as u8;

        match (digit1, x, y, n) {
            (0, 0, 0, 0) => return, // NOP
            (0, 0, E, 0) => self.screen.clear(),
			(0, 0, E, E) => {
				match self.stack.pop() {
					Some(addr) => self.pc = addr,
					None => panic!("[Error] Unexpected opcode '0x00EE' : Stack underflow (Unexpected return from subroutine)"),
				}
			},
			(1, _, _, _) => self.pc = nnn,
			(2, _, _, _) => {
				self.stack.push(self.pc);
				self.pc = nnn;
			},
			(3, _, _, _) => {
				if self.v[x] == nn {
					self.pc += 2;
				}
			},
			(4, _, _, _) => {
				if self.v[x] != nn {
					self.pc += 2;
				}
			},
			(5, _, _, 0) => {
				if self.v[x] == self.v[y] {
					self.pc += 2;
				}
			},
			(6, _, _, _) => self.v[x] = nn,
			(7, _, _, _) => self.v[x] = self.v[x].wrapping_add(nn),
			(8, _, _, 0) => self.v[x] = self.v[y],
			(8, _, _, 1) => self.v[x] |= self.v[y],
			(8, _, _, 2) => self.v[x] &= self.v[y],
			(8, _, _, 3) => self.v[x] ^= self.v[y],
			(8, _, _, 4) => {
				let (vx, carry) = self.v[x].overflowing_add(self.v[y]);
				self.v[x] = vx;
				self.v[F] = if carry {1} else {0};
			},
			(8, _, _, 5) => {
				let (vx, borrow) = self.v[x].overflowing_sub(self.v[y]);
				self.v[x] = vx;
				self.v[F] = if borrow {0} else {1};
			},
			(8, _, _, 6) => {
				self.v[F] = self.v[x] & 1;
				self.v[x] >>= 1;
			},
			(8, _, _, 7) => {
				let (vx, borrow) = self.v[y].overflowing_sub(self.v[x]);
				self.v[x] = vx;
				self.v[F] = if borrow {0} else {1};
			},
			(8, _, _, E) => {
				self.v[F] = (self.v[x] >> 7) & 1;
				self.v[x] <<= 1;
			},
			(9, _, _, 0) => {
				if self.v[x] != self.v[y] {
					self.pc += 2;
				}
			},
			(A, _, _, _) => self.i = nnn,
			(B, _, _, _) => self.pc = (self.v[0] as u16) + nnn,
			(C, _, _, _) => self.v[x] = random::<u8>() & nn,
			(D, _, _, _) => {
				let x_coord = self.v[x];
				let y_coord = self.v[y];

				let mut flipped = false;

				for row in 0..n {
					let byte = self.memory.read_u8(self.i + row as u16);

					for column in 0..8 {
						if (byte & (0b1000_0000 >> column)) != 0 {
							let x_pixel = (x_coord as usize + column) % SCREEN_WIDTH_C8;
							let y_pixel = (y_coord as usize + row) % SCREEN_HEIGHT_C8;

							let pixel_on = self.screen.get_pixel(x_pixel, y_pixel);
							flipped |= pixel_on;
							self.screen.set_pixel(x_pixel, y_pixel, !pixel_on);
						}
					}
				}

				self.v[F] = if flipped {1} else {0};
			},
			(E, _, 9, E) => {
				if self.keyboard.is_pressed(self.v[x]) {
					self.pc += 2;
				}
			},
			(E, _, A, 1) => {
				if !self.keyboard.is_pressed(self.v[x]) {
					self.pc += 2;
				}
			},
			(F, _, 0, 7) => self.v[x] = self.delay_timer,
			(F, _, 0, A) => {
				let mut pressed = false;

				for i in 0..16 {
					if self.keyboard.is_pressed(i) {
						self.v[x] = i;
						pressed = true;
						break;
					}
				}

				if !pressed {
					self.pc -= 2;
				}
			},
			(F, _, 1, 5) => self.delay_timer = self.v[x],
			(F, _, 1, 8) => self.sound_timer = self.v[x],
			(F, _, 1, E) => self.i += self.v[x] as u16,
			(F, _, 2, 9) => {
				self.i = (self.v[x] * 5) as u16;
			},
			(F, _, 3, 3) => {
				let value = self.v[x] as f32;

				self.memory.write_u8(self.i, (value / 100.0).floor() as u8);
				self.memory.write_u8(self.i + 1, (value % 100.0 / 10.0).floor() as u8);
				self.memory.write_u8(self.i + 2, (value % 10.0) as u8);
			},
			(F, _, 5, 5) => {
				for i in 0..x+1 {
					self.memory.write_u8(self.i + i as u16, self.v[i]);
				}
			},
			(F, _, 6, 5) => {
				for i in 0..x+1 {
					self.v[i] = self.memory.read_u8(self.i + i as u16);
				}
			},
            (_, _, _, _) => unimplemented!("Unimplemented opcode : {:#06X}", opcode),
        }
    }

    pub fn timers_tick(&mut self) {
        if self.delay_timer > 0 {
            self.delay_timer -= 1;
        }

        if self.sound_timer > 0 {
            self.sound_timer -= 1;

            if self.sound_timer == 0 {
                // BEEEP
            }
        }
    }

    pub fn keypress(&mut self, key: usize, pressed: bool) {
        self.keyboard.keypress(key, pressed);
    }

    pub fn cpu_tick(&mut self) {
		// print!("pc = {:#06X}", self.pc);
        // fetch
        let opcode = self.fetch_opcode();
		// println!(" (opcode = {:#06X})", opcode);

        // decode and execute
        self.decode_opcode(opcode);
    }

    pub fn get_screen(&self) -> &[bool] {
        self.screen.get_screen()
    }

    pub fn load_rom(&mut self, data: &[u8]) {
        self.memory.write_data(data, START_ADDRESS);
    }
}
