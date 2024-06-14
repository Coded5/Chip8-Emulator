use std::{
    fs::{metadata, File},
    io::{BufReader, Read}
};

pub struct Chip8 {
    pub registers: [u8; 16],
    pub memory: [u8; 4096],
    pub index_register: u16,
    pub program_counter: u16,
    pub stack: [u16; 16],
    pub stack_pointer: u16,
    pub delay_timer: u8,
    pub sound_timer: u8,
    pub keypad: [bool; 16],
    pub video: [u8; 64*32],
    pub opcode: u16
}

const START_ADDRESS: u16 = 0x200;
const FONTSET_START_ADDRESS: u16 = 0x50;

const FONTSET_SIZE: u16 = 80;

const FONT_DATA: [u8; FONTSET_SIZE as usize] = [
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
	  0xF0, 0x80, 0xF0, 0x80, 0x80  // F
];

#[allow(dead_code)]
impl Chip8 {

    pub fn create() -> Chip8 {
        let mut chip = Chip8 {
            registers: [0; 16],
            memory: [0; 4096],
            index_register: 0,
            program_counter: START_ADDRESS,
            stack: [0; 16],
            stack_pointer: 0,
            delay_timer: 0, 
            sound_timer: 0, 
            keypad: [false; 16],
            video: [0; 64*32],
            opcode: 0
        };

        for i in 0..FONTSET_SIZE {
            chip.memory[( FONTSET_START_ADDRESS + i ) as usize] = FONT_DATA[i as usize];
        }

        chip
    }

    //Clear Display
    fn op_00e0(&mut self) {
        self.video = [0; 64*32];
    }

    //RET: return from a subroutine
    fn op_00ee(&mut self) {
        self.stack_pointer -= 1;
        self.program_counter = self.stack[self.stack_pointer as usize];
    }

    //JP addr
    fn op_1nnn(&mut self) {
        let address: u16 = self.opcode & 0x0FFF;
        self.program_counter = address;
    }

    //CALL addr
    //Will return eventually
    fn op_2nnn(&mut self) {
        let address: u16 = self.opcode & 0x0FFF;

        self.stack[self.stack_pointer as usize] = self.program_counter;
        self.stack_pointer += 1;

        self.program_counter = address;
    }

    //SE Vx, byte
    //Skip next instruction if Vx == kk
    fn op_3xnn(&mut self) {
        let register_index: usize = ((self.opcode & 0x0F00) >> 8_u16) as usize;
        let compare_to    : u8 = (self.opcode & 0x00FF) as u8;

        if self.registers[register_index] == compare_to {
            self.program_counter += 2;
        }
    }

    //SNE Vx, byte
    //Skip next instruction if Vx != kk
    fn op_4xnn(&mut self) {
        let register_index: usize = ((self.opcode & 0x0F00) >> 8_u16) as usize;
        let compare_to    : u8    = (self.opcode & 0x00FF) as u8;

        if self.registers[register_index] != compare_to {
            self.program_counter += 2;
        }
    }

    //SE Vx, Vy
    //Skip if equal
    fn op_5xy0(&mut self) {
        let register_0: usize = ((self.opcode & 0x0F00) >> 8_u16) as usize;
        let register_1: usize = ((self.opcode & 0x00F0) >> 4_u16) as usize;

        if self.registers[register_0] == self.registers[register_1] {
            self.program_counter += 2;
        }
    }

    //SNE Vx, Vy
    //Skip not equal
    fn op_6xnn(&mut self) {
        let register_index: usize = ((self.opcode & 0x0F00) >> 8_u16) as usize;
        let set_to        : u8   = (self.opcode & 0x00FF) as u8;

        self.registers[register_index] = set_to;
    }

    fn op_7xnn(&mut self) {
        let register_index: usize = ((self.opcode & 0x0F00) >> 8_u16) as usize;
        let add           : u16   = self.opcode & 0x00FF;

        let sum: u16 = self.registers[register_index] as u16 + add;

        self.registers[register_index] = (sum & 0xFF) as u8;
    }

    fn op_8xyk(&mut self, k: u32) {
        let a: usize = ((self.opcode & 0x0F00) >> 8u16) as usize;
        let b: usize = ((self.opcode & 0x00F0) >> 4u16) as usize;

        match k {
            0 => self.registers[a]  = self.registers[b],
            1 => self.registers[a] |= self.registers[b],
            2 => self.registers[a] &= self.registers[b],
            3 => self.registers[a] ^= self.registers[b],
            4 => {
                let sum: u16 = self.registers[a] as u16 + self.registers[b] as u16;
                self.registers[0xF] = if sum > 0xFF { 1 } else { 0 };

                self.registers[a] = (sum & 0xFF) as u8;
            },
            5 => {
                self.registers[0xF] = if self.registers[a] >= self.registers[b] { 1 } else { 0 };
                self.registers[a] = self.registers[a].overflowing_sub(self.registers[b]).0;
            },
            6 => {
                self.registers[0xF] = self.registers[a] & 0x1;
                self.registers[a] >>= 1;
            },
            7 => {
                self.registers[0xF] = if self.registers[b] > self.registers[a] { 1 } else { 0 };
                self.registers[a] = self.registers[b] - self.registers[a];
            },
            0xE => {
                self.registers[0xF] = (self.registers[a] & 0x80) >> 7;
                self.registers[a] <<= 1;
            },
            _ => {
                panic!("Invalid opcode: {:#04x}", self.opcode);
            }
        }
    }

    //SNE Vx, Vy
    fn op_9xy0(&mut self) {
        let a: usize = ((self.opcode & 0x0F00) >> 8u16) as usize;
        let b: usize = ((self.opcode & 0x00F0) >> 4u16) as usize;

        if self.registers[a] != self.registers[b] {
            self.program_counter += 2;
        }
    }

    //LD I, addr
    fn op_annn(&mut self) {
        let address: u16 = self.opcode & 0x0FFF;
        self.index_register = address;
    }

    //JP V0, addr
    fn op_bnnn(&mut self) {
        let address: u16 = self.opcode & 0x0FFF;
        self.program_counter = self.registers[0] as u16 + address;
    }

    //RND Vx, byte
    fn op_cxkk(&mut self) {
        let register_index = ((self.opcode & 0x0F00) >> 8) as usize;
        let byte: u8 = (self.opcode & 0x00FF) as u8;

        self.registers[register_index] = rand::random::<u8>() & byte;
    }

    fn op_dxyn(&mut self) {
        let a: usize = ((self.opcode & 0x0F00) >> 8) as usize;
        let b: usize = ((self.opcode & 0x00F0) >> 4) as usize;
        let height: u8 = (self.opcode & 0x000F) as u8;

        const VIDEO_WIDTH: u16 = 64;
        const VIDEO_HEIGHT: u16 = 32;

        let x: u8 = self.registers[a] % 64;
        let y: u8 = self.registers[b] % 32; 

        self.registers[0xF] = 0;

        for row in 0..height {
            let i: usize = ( self.index_register + (row as u16) ) as usize;
            let sprite: u8 = self.memory[i];

            for col in 0..8_u8 {
                let pixel: u8 = sprite & (0x80 >> col);
                let ypos: u16 = (y as u16) + (row as u16);
                let xpos: u16 = (x as u16) + (col as u16);

                let screen_pixel: &mut u8 = &mut self.video[( xpos + ypos * 64 ) as usize];

                println!("sprite_byte: {:#04x}, pixel: {}", sprite, pixel);
                if pixel != 0x0 {
                    if *screen_pixel == 0xFF {
                        self.registers[0xF] = 1;
                    }

                    *screen_pixel ^= 0xFF;

                }
            }
        }

        self.registers[0xF] = 0;
        
    }

    fn op_ex9e(&mut self) {
        let register_index: usize = ((self.opcode & 0x0F00) >> 8) as usize;
        let key: usize = self.registers[register_index] as usize;

        if self.keypad[key] {
            self.program_counter += 2;
        }
    }

    fn op_exa1(&mut self) {
        let register_index: usize = ((self.opcode & 0x0F00) >> 8) as usize;
        let key: usize = self.registers[register_index] as usize;

        if !self.keypad[key] {
            self.program_counter += 2;
        }
    }

    fn op_fx07(&mut self) {
        let register_index: usize = ((self.opcode & 0x0F00) >> 8) as usize;
        self.registers[register_index] = self.delay_timer;
    }

    fn op_fx0a(&mut self) {
        let register_index: usize = ((self.opcode & 0x0F00) >> 8) as usize;
        let mut flag: bool = false;

        for (i, key) in self.keypad.iter().enumerate() {
            if *key {
                self.registers[register_index] = i as u8;
                flag = true;
            }
        }

        if !flag {
            self.program_counter -= 2;
        }
    }

    fn op_fx15(&mut self) {
        let register_index: usize = ((self.opcode & 0x0F00) >> 8) as usize;
        self.delay_timer = self.registers[register_index];
    }

    fn op_fx18(&mut self) {
        let register_index: usize = ((self.opcode & 0x0F00) >> 8) as usize;
        self.sound_timer = self.registers[register_index];
    }

    fn op_fx1e(&mut self) {
        let register_index: usize = ((self.opcode & 0x0F00) >> 8) as usize;
        self.index_register += self.registers[register_index] as u16;
    }

    fn op_fx29(&mut self) {
        let register_index: usize = ((self.opcode & 0x0F00) >> 8) as usize;
        let digit = self.registers[register_index];

        self.index_register = FONTSET_START_ADDRESS + (5 * digit) as u16;
    }

    fn op_fx33(&mut self) {
        let register_index: usize = ((self.opcode & 0x0F00) >> 8) as usize;
        let value: f64 = self.registers[register_index] as f64;

        let hundreds: u8 = (value / 100.0).floor() as u8;
        let tens    : u8 = ((value / 10.0) % 10.0).floor() as u8;
        let ones    : u8 = (value % 10.0) as u8;

        self.memory[self.index_register as usize] = hundreds;
        self.memory[( self.index_register+1 ) as usize] = tens;
        self.memory[( self.index_register+2 ) as usize] = ones;
    }

    fn op_fx55(&mut self) {
        let register_index: usize = ((self.opcode & 0x0F00) >> 8) as usize;
        
        for i in 0..=register_index {
            self.memory[( self.index_register + i as u16) as usize] = self.registers[i];
        }
    }

    fn op_fx65(&mut self) {
        let register_index: usize = ((self.opcode & 0x0F00) >> 8) as usize;
        
        for i in 0..=self.registers[register_index] {
            self.registers[i as usize] = self.memory[(self.index_register + i as u16) as usize];
        }
    }
    
    pub fn run(&mut self) {
        let program_counter = self.program_counter as usize;
        let first_part : u16 = ( self.memory[program_counter] as u16 ) << 8_u16;
        let second_part: u16 = ( self.memory[program_counter+1] ) as u16;
        self.opcode = first_part | second_part;

        self.program_counter += 2;
        //println!("Decoding opcode: {:#04x} at {:#04x}", self.opcode, self.program_counter);

        //Decode
        let cmd: u8 = ((self.opcode & 0xF000) >> 12) as u8;
        //println!("CMD: {:#02x}", cmd);

        //Execute
        match cmd {
            0x0 => {
                let operand: u16 = self.opcode & 0x0FFF;
                match operand {
                    0x0E0 => self.op_00e0(),
                    0x0EE => self.op_00ee(),
                    _ => panic!("Invalid opcode: {:#04x}", self.opcode)
                }
            },
            0x1 => self.op_1nnn(),
            0x2 => self.op_2nnn(),
            0x3 => self.op_3xnn(),
            0x4 => self.op_4xnn(),
            0x5 => self.op_5xy0(),
            0x6 => self.op_6xnn(),
            0x7 => self.op_7xnn(),
            0x8 => {
                let k = self.opcode & 0x000F;
                self.op_8xyk(k.into());
            },
            0x9 => self.op_9xy0(),
            0xA => self.op_annn(),
            0xB => self.op_bnnn(),
            0xC => self.op_cxkk(),
            0xD => self.op_dxyn(),
            0xE => {
                let identity: u16 = self.opcode & 0x00FF;
                match identity {
                    0x9E => self.op_ex9e(),
                    0xA1 => self.op_exa1(),
                    _ => panic!("Invalid opcode: {:#04x}", self.opcode)
                }
            },
            0xF => {
                let identity: u16 = self.opcode & 0x00FF;
                match identity {
                    0x07 => self.op_fx07(),
                    0x0A => self.op_fx0a(),
                    0x15 => self.op_fx15(),
                    0x18 => self.op_fx18(),
                    0x1E => self.op_fx1e(),
                    0x29 => self.op_fx29(),
                    0x33 => self.op_fx33(),
                    0x55 => self.op_fx55(),
                    0x65 => self.op_fx65(),
                    _ => panic!("Invalid opcode {:#04x}", self.opcode)
                }
            },
            _ => panic!("Invalid opcode: {:#04x}", self.opcode)
        }

        if self.delay_timer > 0 { self.delay_timer -= 1; }
        if self.sound_timer > 0 { self.sound_timer -= 1; }
    }

    pub fn temp(&mut self) {
        for _ in 0..246 {
            let program_counter = self.program_counter as usize;
            let first_part : u16 = ( self.memory[program_counter] as u16 ) << 8_u16;
            let second_part: u16 = ( self.memory[program_counter+1] ) as u16;
            println!("{:#04x}", first_part | second_part);

            self.program_counter += 2;
        }
    }

    pub fn load_rom(&mut self, path: &str) {
        let file = File::open(path).unwrap();
        let mut buffer_reader = BufReader::new(file);

        let size = metadata(path).unwrap().len() as usize;
        let mut buffer = vec![0u8; size];
        buffer_reader.read_exact(&mut buffer).unwrap();

        println!("size: {}", size);
        for (i, buf) in buffer.iter().enumerate() {
            self.memory[(START_ADDRESS as usize) + i] = *buf;
        }

    }

}
