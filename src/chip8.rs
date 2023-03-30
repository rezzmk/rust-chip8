use rand::Rng;
use std::path::Path;
use std::fs::File;
use std::io::Read;

// http://devernay.free.fr/hacks/chip8/C8TECH10.HTM#font
const FONTSET: [u8; 80] = [
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

pub struct State {
    memory: [u8; 4096],         // 4KB of memory
    v: [u8; 16],                // 16 8-bit data registers (V0-VF)
    i: u16,                     // 16-bit index register (I)
    pc: u16,                    // 16-bit program counter (PC)
    stack: [u16; 16],           // 16-level stack to store return addresses
    sp: u8,                     // 8-bit stack pointer (SP)
    display: [bool; 64 * 32],   // 64x32 pixel monochrome display
    delay_timer: u8,            // 8-bit delay timer
    sound_timer: u8,            // 8-bit sound timer
    keypad: [bool; 16],         // 16-key hexadecimal keypad (0-9, A-F)
}

impl State {
    pub fn new() -> Self {
        let mut state = State {
            memory: [0u8; 4096],
            v: [0; 16],
            i: 0,
            pc: 0x200,
            stack: [0; 16],
            sp: 0,
            display: [false; 64 * 32],
            delay_timer: 0,
            sound_timer: 0,
            keypad: [false; 16],
        };

        state.load_font_set();
        return state;
    }

    pub fn get_display(&self) -> &[bool; 64 * 32] {
        return &self.display;
    }

    fn load_font_set(&mut self) {
        self.memory[0..FONTSET.len()].copy_from_slice(&FONTSET);
    }

    pub fn load_rom<P: AsRef<Path>>(&mut self, path: P) -> std::io::Result<()> {
        let mut file = File::open(path)?;

        let program_start = &mut self.memory[0x200..];
        let bytes_read = file.read(program_start)?;

        if bytes_read == 0 {
            return Err(std::io::Error::new(std::io::ErrorKind::InvalidData, "The ROM is fked"));
        }

        self.pc = 0x200 as u16;

        Ok(())
    }

    pub fn emulate_cycle(&mut self) {
        self.execute_opcode();
        self.update_timers();
    }

    fn fetch_opcode(&self) -> u16 {
        return (self.memory[self.pc as usize] as u16) << 8 | (self.memory[self.pc as usize + 1] as u16)
    }

    fn execute_opcode(&mut self) {
        let opcode = self.fetch_opcode();
        println!("opcode: {:#X}", opcode);

        let nibbles = (
            (opcode & 0xF000) >> 12 as u8,
            (opcode & 0x0F00) >> 8 as u8,
            (opcode & 0x00F0) >> 4 as u8,
            (opcode & 0x000F) as u8,
        );

        let pc_change = match nibbles { 
            (0x00, 0x00, 0x0e, 0x00) => self.op_00e0(),
            (0x00, 0x00, 0x0e, 0x0e) => self.op_00ee(),
            (0x01, _, _, _) => self.op_1nnn(opcode),
            (0x02, _, _, _) => self.op_2nnn(opcode),
            (0x03, _, _, _) => self.op_3xkk(opcode),
            (0x04, _, _, _) => self.op_4xkk(opcode),
            (0x05, _, _, 0x00) => self.op_5xy0(opcode),
            (0x06, _, _, _) => self.op_6xkk(opcode),
            (0x07, _, _, _) => self.op_7xkk(opcode),
            (0x08, _, _, 0x00) => self.op_8xy0(opcode),
            (0x08, _, _, 0x01) => self.op_8xy1(opcode),
            (0x08, _, _, 0x02) => self.op_8xy2(opcode),
            (0x08, _, _, 0x03) => self.op_8xy3(opcode),
            (0x08, _, _, 0x04) => self.op_8xy4(opcode),
            (0x08, _, _, 0x05) => self.op_8xy5(opcode),
            (0x08, _, _, 0x06) => self.op_8xy6(opcode),
            (0x08, _, _, 0x07) => self.op_8xy7(opcode),
            (0x08, _, _, 0x0e) => self.op_8xye(opcode),
            (0x09, _, _, 0x00) => self.op_9xy0(opcode),
            (0x0a, _, _, _) => self.op_annn(opcode),
            (0x0b, _, _, _) => self.op_bnnn(opcode),
            (0x0c, _, _, _) => self.op_cxkk(opcode),
            (0x0d, _, _, _) => self.op_dxyn(opcode),
            (0x0e, _, 0x09, 0x0e) => self.op_ex9e(opcode),
            (0x0e, _, 0x0a, 0x01) => self.op_exa1(opcode),
            (0x0f, _, 0x00, 0x07) => self.op_fx07(opcode),
            (0x0f, _, 0x00, 0x0a) => self.op_fx0a(opcode),
            (0x0f, _, 0x01, 0x05) => self.op_fx15(opcode),
            (0x0f, _, 0x01, 0x08) => self.op_fx18(opcode),
            (0x0f, _, 0x01, 0x0e) => self.op_fx1e(opcode),
            (0x0f, _, 0x02, 0x09) => self.op_fx29(opcode),
            (0x0f, _, 0x03, 0x03) => self.op_fx33(opcode),
            (0x0f, _, 0x05, 0x05) => self.op_fx55(opcode),
            (0x0f, _, 0x06, 0x05) => self.op_fx65(opcode),
            _ => self.pc += 2
        };
    }

    fn op_invalid(&self, opcode: u16) {
        //println!("Unknown opcode: {:04X}", opcode);
    }

    // Clear the display
    fn op_00e0(&mut self) {
        self.display.fill(false);
        self.pc += 2;
    }

    // Return from a subroutine
    fn op_00ee(&mut self) {
        self.sp -= 1;
        self.pc = self.stack[self.sp as usize] + 2;
    }

    // Jump to address NNN
    fn op_1nnn(&mut self, opcode: u16) {
        println!("Jumping to {:#X}", opcode);
        self.pc = opcode & 0x0FFF;
    }

    // Call subroutine at NNN
    fn op_2nnn(&mut self, opcode: u16) {
        self.stack[self.sp as usize] = self.pc;
        self.sp += 1;
        self.pc = opcode & 0x0FFF;
    }

    // Skip next instruction if Vx == KK
    fn op_3xkk(&mut self, opcode: u16) {
        let x = ((opcode & 0x0F00) >> 8) as usize;
        let kk = (opcode & 0x00FF) as u8;
        if self.v[x] == kk {
            self.pc += 4;
        } else {
            self.pc += 2;
        }
    }

    // Skip next instruction if Vx != KK
    fn op_4xkk(&mut self, opcode: u16) {
        let x = ((opcode & 0x0F00) >> 8) as usize;
        let kk = (opcode & 0x00FF) as u8;
        if self.v[x] != kk {
            self.pc += 4;
        } else {
            self.pc += 2;
        }
    }

    // Skip next instruction if Vx == Vy
    fn op_5xy0(&mut self, opcode: u16) {
        let x = ((opcode & 0x0F00) >> 8) as usize;
        let y = ((opcode & 0x00F0) >> 4) as usize;
        if self.v[x] == self.v[y] {
            self.pc += 4;
        } else {
            self.pc += 2;
        }
    }

    // Set Vx = kk
    fn op_6xkk(&mut self, opcode: u16) {
        let x = ((opcode & 0x0F00) >> 8) as usize;
        let kk = (opcode & 0x00FF) as u8;
        self.v[x] = kk;
        self.pc += 2;
    }

    // Set Vx = Vx + kk
    fn op_7xkk(&mut self, opcode: u16) {
        let x = ((opcode & 0x0F00) >> 8) as usize;
        let kk = (opcode & 0x00FF) as u8;
        self.v[x] = self.v[x].wrapping_add(kk);
        self.pc += 2;
    }

    // Set Vx = Vy
    fn op_8xy0(&mut self, opcode: u16) {
        let x = ((opcode & 0x0F00) >> 8) as usize;
        let y = ((opcode & 0x00F0) >> 4) as usize;
        self.v[x] = self.v[y];
        self.pc += 2;
    }

    // Set Vx = Vx OR Vy.
    fn op_8xy1(&mut self, opcode: u16) {
        let x = ((opcode & 0x0F00) >> 8) as usize;
        let y = ((opcode & 0x00F0) >> 4) as usize;
        self.v[x] |= self.v[y];
        self.pc += 2;
    }

    // Set Vx = Vx AND Vy.
    fn op_8xy2(&mut self, opcode: u16) {
        let x = ((opcode & 0x0F00) >> 8) as usize;
        let y = ((opcode & 0x00F0) >> 4) as usize;
        self.v[x] &= self.v[y];
        self.pc += 2;
    }

    // Set Vx = Vx XOR Vy.
    fn op_8xy3(&mut self, opcode: u16) {
        let x = ((opcode & 0x0F00) >> 8) as usize;
        let y = ((opcode & 0x00F0) >> 4) as usize;
        self.v[x] ^= self.v[y];
        self.pc += 2;
    }

    // Set Vx = Vx + Vy, set VF = carry.
    fn op_8xy4(&mut self, opcode: u16) {
        let x = ((opcode & 0x0F00) >> 8) as usize;
        let y = ((opcode & 0x00F0) >> 4) as usize;
        let (sum, carry) = self.v[x].overflowing_add(self.v[y]);
        self.v[0xF] = if carry { 1 } else { 0 };
        self.v[x] = sum;
        self.pc += 2;
    }

    fn op_8xy5(&mut self, opcode: u16) {
        let x = ((opcode & 0x0F00) >> 8) as usize;
        let y = ((opcode & 0x00F0) >> 4) as usize;
        let (result, borrow) = self.v[x].overflowing_sub(self.v[y]);
        self.v[0xF] = if borrow { 0 } else { 1 };
        self.v[x] = result;
        self.pc += 2;
    }

    // Set Vx = Vx SHR 1.
    fn op_8xy6(&mut self, opcode: u16) {
        let x = ((opcode & 0x0F00) >> 8) as usize;
        self.v[0xF] = self.v[x] & 0x01;
        self.v[x] >>= 1;
        self.pc += 2;
    }

    fn op_8xy7(&mut self, opcode: u16) {
        let x = ((opcode & 0x0F00) >> 8) as usize;
        let y = ((opcode & 0x00F0) >> 4) as usize;
        let (result, borrow) = self.v[y].overflowing_sub(self.v[x]);
        self.v[0xF] = if borrow { 0 } else { 1 };
        self.v[x] = result;
        self.pc += 2;
    }

    fn op_8xye(&mut self, opcode: u16) {
        let x = ((opcode & 0x0F00) >> 8) as usize;
        self.v[0xF] = (self.v[x] & 0x80) >> 7;
        self.v[x] <<= 1;
        self.pc += 2;
    }

    fn op_9xy0(&mut self, opcode: u16) {
        let x = ((opcode & 0x0F00) >> 8) as usize;
        let y = ((opcode & 0x00F0) >> 4) as usize;

        if self.v[x] != self.v[y] {
            self.pc += 4;
        }
        else {
            self.pc += 2;
        }
    }

    fn op_annn(&mut self, opcode: u16) {
        self.i = opcode & 0x0FFF;
        self.pc += 2;
    }

    fn op_bnnn(&mut self, opcode: u16) {
        let nnn = opcode & 0x0FFF;
        self.pc = nnn + self.v[0] as u16;
    }

    fn op_cxkk(&mut self, opcode: u16) {
        let x = ((opcode & 0x0F00) >> 8) as usize;
        let kk = (opcode & 0x00FF) as u8;

        let random: u8 = rand::thread_rng().gen();
        self.v[x] = random & kk;
        self.pc += 2;
    }

    fn op_dxyn(&mut self, opcode: u16) {
        let vx = self.v[(opcode & 0x0F00 >> 8) as usize] as u16;
        let vy = self.v[(opcode & 0x00F0 >> 4) as usize] as u16;
        let height = opcode & 0x000F;

        self.v[0xF] = 0;
        for y in 0..height {
            let pixel = self.memory[(self.i + y) as usize];
            for x in 0..8 {
                if pixel & (0x80 >> x) != 0 {
                    let index = ((vx + x + ((vy + y) * 64)) % (64 * 32)) as usize;
                    if self.display[index] {
                        self.v[0xF] = 1;
                    }
                    self.display[index] ^= true;
                }
            }
        }

        self.pc += 2;
    }

    fn op_ex9e(&mut self, opcode: u16) {
        let x = ((opcode & 0x0F00) >> 8) as usize;
        if self.keypad[self.v[x] as usize] {
            self.pc += 4;
        }
        else {
            self.pc += 2;
        }
    }

    fn op_exa1(&mut self, opcode: u16) {
        let x = ((opcode & 0x0F00) >> 8) as usize;
        if !self.keypad[self.v[x] as usize] {
            self.pc += 4;
        }
        else {
            self.pc += 2;
        }
    }

    fn op_fx07(&mut self, opcode: u16) {
        let x = ((opcode & 0x0F00) >> 8) as usize;
        self.v[x] = self.delay_timer;
        self.pc += 2;
    }

    fn op_fx0a(&mut self, opcode: u16) {
        let x = ((opcode & 0x0F00) >> 8) as usize;

        let mut pressed = false;
        for i in 0..self.keypad.len() {
            if self.keypad[i] {
                self.v[x] = i as u8;
                pressed = true;
            }
        }

        if !pressed {
            println!("Waiting for keypress");
            return;
        }

        self.pc += 2;
    }
 
    fn op_fx15(&mut self, opcode: u16) {
        let x = ((opcode & 0x0F00) >> 8) as usize;
        self.delay_timer = self.v[x];
        self.pc += 2;
    }

    fn op_fx18(&mut self, opcode: u16) {
        let x = ((opcode & 0x0F00) >> 8) as usize;
        self.sound_timer = self.v[x];
        self.pc += 2;
    }

    fn op_fx1e(&mut self, opcode: u16) {
        let x = ((opcode & 0x0F00) >> 8) as usize;
        self.i = self.i.wrapping_add(self.v[x] as u16);
        self.pc += 2;
    }

    fn op_fx29(&mut self, opcode: u16) {
        let x = ((opcode & 0x0F00) >> 8) as usize;
        self.i = self.v[x] as u16 * 5; // each sprite has 5 bytes of data
        self.pc += 2;
    }

    fn op_fx33(&mut self, opcode: u16) {
        let x = ((opcode & 0x0F00) >> 8) as usize;
        let vx = self.v[x];

        self.memory[self.i as usize] = vx / 100;
        self.memory[(self.i + 1) as usize] = (vx % 100) / 10;
        self.memory[(self.i + 2) as usize] = vx % 10;
        self.pc += 2;
    }

    fn op_fx55(&mut self, opcode: u16) {
        let x = (opcode & 0x0F00) >> 8;

        for index in 0..=x as usize {
            self.memory[self.i as usize + index] = self.v[index];
        }

        self.pc += 2;
    }

    fn op_fx65(&mut  self, opcode: u16) {
        let x = (opcode & 0x0F00) >> 8;

        for i in 0..=x as usize {
            self.v[i] = self.memory[(self.i as usize) + i];
        }

        self.pc += 2;
    }

    pub fn update_timers(&mut self) {
        if self.delay_timer > 0 {
            self.delay_timer -= 1;
        }

        if self.sound_timer > 0 {
            self.sound_timer -= 1;
        }
    }

    pub fn key_down(&mut self, key: u8) {
        if key < 16 {
            self.keypad[key as usize] = true;
        }
    }

    pub fn key_up(&mut self, key: u8) {
        if key < 16 {
            self.keypad[key as usize] = false;
        }
    }
}
