use crate::error::*;
use crate::ins::Ins;
use rand::{rngs::ThreadRng, thread_rng, Rng};
use sdl2::{
    keyboard::Scancode,
    pixels::Color,
    rect::Rect,
    render::{Canvas, RenderTarget},
    EventPump,
};
use std::collections::HashSet;
use std::time::Instant;

const FONT: [u8; 80] = [
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

const FONT_SPRITE_SIZE: usize = 5;

const PROGRAM_START: usize = 0x200;

pub const DISPLAY_WIDTH: usize = 64;
pub const DISPLAY_HEIGHT: usize = 32;
const DISPLAY_SIZE: usize = DISPLAY_WIDTH * DISPLAY_HEIGHT;

pub const PIXEL_SIZE: usize = 15;

pub struct Machine {
    pc: u16,
    i: u16,
    v: [u8; 16],
    memory: [u8; 4096],
    stack: Vec<u16>,
    display: [bool; DISPLAY_SIZE],
    dt: u8,
    st: u8,
    timer: Option<Instant>,
    rng: ThreadRng,
    keyboard_state: HashSet<Scancode>,
    old_keyboard_state: HashSet<Scancode>,
}

impl Machine {
    pub fn new() -> Self {
        let mut memory = [0; 4096];
        memory[0..80].copy_from_slice(&FONT);
        Self {
            pc: PROGRAM_START as u16,
            i: 0,
            v: [0; 16],
            memory,
            stack: vec![],
            display: [false; DISPLAY_SIZE],
            dt: 0,
            st: 0,
            timer: None,
            rng: thread_rng(),
            keyboard_state: HashSet::new(),
            old_keyboard_state: HashSet::new(),
        }
    }
    pub fn load(&mut self, program: &[u8]) {
        self.memory[PROGRAM_START..PROGRAM_START + program.len()].copy_from_slice(program);
    }

    fn draw(&self, canvas: &mut Canvas<impl RenderTarget>) -> Result<()> {
        canvas.clear();

        for (y, line) in self.display.chunks_exact(DISPLAY_WIDTH).enumerate() {
            for (x, pixel) in line.iter().enumerate() {
                if *pixel {
                    canvas.set_draw_color(Color::WHITE);
                } else {
                    canvas.set_draw_color(Color::BLACK);
                }
                canvas
                    .fill_rect(Rect::new(
                        x as i32 * PIXEL_SIZE as i32,
                        y as i32 * PIXEL_SIZE as i32,
                        PIXEL_SIZE as u32,
                        PIXEL_SIZE as u32,
                    ))
                    .map_err(Error::FillRect)?;
            }
        }

        canvas.present();
        Ok(())
    }

    pub fn step(
        &mut self,
        event_pump: &mut EventPump,
        canvas: &mut Canvas<impl RenderTarget>,
    ) -> Result<()> {
        // update timer
        if let Some(timer) = self.timer {
            let elapsed = timer.elapsed().as_micros() / 16667;
            if elapsed > 0 {
                self.dt = self.dt.saturating_sub(elapsed as u8);
                self.st = self.st.saturating_sub(elapsed as u8);
                self.timer = Some(Instant::now());
            }
        } else {
            self.timer = Some(Instant::now());
        }

        if self.st != 0 {
            print!("\x07"); // ring bell
        }

        // update keyboard state
        event_pump.pump_events();
        let mut state = event_pump.keyboard_state().pressed_scancodes().collect();
        std::mem::swap(&mut state, &mut self.keyboard_state);
        std::mem::swap(&mut state, &mut self.old_keyboard_state);

        // decode instruction
        let ins_bytes: [u8; 2] = self.memory[self.pc as usize..self.pc as usize + 2]
            .try_into()
            .unwrap();
        let ins = Ins::decode(u16::from_be_bytes(ins_bytes))?;
        self.pc += 2;

        // run instruction
        match ins {
            Ins::Sys(_) => {} // no special machine code support
            Ins::Cls => {
                self.display = [false; DISPLAY_SIZE];
                canvas.clear();
                canvas.present();
            }
            Ins::Ret => self.pc = self.stack.pop().expect("Stack underflow"),
            Ins::Jp(addr) => {
                self.pc = addr;
            }
            Ins::Call(addr) => {
                self.stack.push(self.pc);
                self.pc = addr;
            }
            Ins::SeVB(reg, byte) => {
                if self.v[reg] == byte {
                    self.pc += 2;
                }
            }
            Ins::SneVB(reg, byte) => {
                if self.v[reg] != byte {
                    self.pc += 2;
                }
            }
            Ins::SeVV(x, y) => {
                if self.v[x] == self.v[y] {
                    self.pc += 2;
                }
            }
            Ins::LdVB(reg, byte) => {
                self.v[reg] = byte;
            }
            Ins::AddVB(reg, byte) => {
                self.v[reg] = self.v[reg].wrapping_add(byte);
            }
            Ins::LdVV(x, y) => {
                self.v[x] = self.v[y];
            }
            Ins::Or(x, y) => {
                self.v[x] |= self.v[y];
            }
            Ins::And(x, y) => {
                self.v[x] &= self.v[y];
            }
            Ins::Xor(x, y) => {
                self.v[x] ^= self.v[y];
            }
            Ins::AddVV(x, y) => {
                let (res, is_overflow) = self.v[x].overflowing_add(self.v[y]);
                self.v[0xF] = if is_overflow { 1 } else { 0 };
                self.v[x] = res;
            }
            Ins::Sub(x, y) => {
                let res = self.v[x].wrapping_sub(self.v[y]);
                self.v[0xF] = if self.v[x] > self.v[y] { 1 } else { 0 };
                self.v[x] = res;
            }
            Ins::Shr(x) => {
                self.v[0xF] = self.v[x] & 1;
                self.v[x] >>= 1;
            }
            Ins::Subn(x, y) => {
                let res = self.v[y].wrapping_sub(self.v[x]);
                self.v[0xF] = if self.v[y] > self.v[x] { 1 } else { 0 };
                self.v[x] = res;
            }
            Ins::Shl(x) => {
                self.v[0xF] = self.v[x] >> 7;
                self.v[x] <<= 1;
            }
            Ins::SneVV(x, y) => {
                if self.v[x] != self.v[y] {
                    self.pc += 2;
                }
            }
            Ins::LdI(addr) => {
                self.i = addr;
            }
            Ins::JpV0(addr) => {
                self.pc = self.v[0] as u16 + addr;
            }
            Ins::Rnd(x, mask) => {
                self.v[x] = self.rng.gen::<u8>() & mask;
            }
            Ins::Drw(x, y, n) => {
                let sprite = &self.memory[self.i as usize..self.i as usize + n as usize];
                self.v[0xF] = 0;
                for (i, byte) in sprite.iter().enumerate() {
                    for j in 0..8 {
                        let bit = (byte >> (7 - j) & 1) == 1;
                        let xj = (self.v[x] as usize + j) % DISPLAY_WIDTH;
                        let yi = (self.v[y] as usize + i) % DISPLAY_HEIGHT;
                        let tgt = xj + yi * DISPLAY_WIDTH;

                        if self.display[tgt] && bit {
                            self.v[0xF] = 1;
                        }
                        self.display[tgt] ^= bit;
                    }
                }
                self.draw(canvas)?;
            }
            Ins::Skp(x) => {
                if let Some(key) = keypad_to_scancode(self.v[x]) {
                    if self.keyboard_state.contains(&key) {
                        self.pc += 2;
                    }
                }
            }
            Ins::Sknp(x) => {
                if let Some(key) = keypad_to_scancode(self.v[x]) {
                    if !self.keyboard_state.contains(&key) {
                        self.pc += 2;
                    }
                } else {
                    self.pc += 2;
                }
            }
            Ins::LdVDt(x) => {
                self.v[x] = self.dt;
            }
            Ins::LdVK(x) => {
                if let Some(scancode) = self
                    .keyboard_state
                    .difference(&self.old_keyboard_state)
                    .next()
                {
                    if let Some(val) = scancode_to_keypad(scancode) {
                        self.v[x] = val;
                    } else {
                        self.pc -= 2;
                    }
                } else {
                    self.pc -= 2;
                }
            }
            Ins::LdDtV(x) => {
                self.dt = self.v[x];
            }
            Ins::LdStV(x) => {
                self.st = self.v[x];
            }
            Ins::AddI(x) => {
                self.i += self.v[x] as u16;
            }
            Ins::LdFV(x) => {
                let digit = self.v[x] % 10;
                self.i = (digit as usize * FONT_SPRITE_SIZE) as u16;
            }
            Ins::LdBV(x) => {
                self.memory[self.i as usize] = self.v[x] / 100;
                self.memory[self.i as usize + 1] = self.v[x] % 100 / 10;
                self.memory[self.i as usize + 2] = self.v[x] % 10;
            }
            Ins::LdIlocV(x) => {
                self.memory[self.i as usize..=((self.i + x as u16) as usize)]
                    .copy_from_slice(&self.v[0..=(x as usize)]);
            }
            Ins::LdVIloc(x) => {
                self.v[0..=(x as usize)].copy_from_slice(
                    &self.memory[self.i as usize..=((self.i + x as u16) as usize)],
                );
            }
        }
        Ok(())
    }
}

fn keypad_to_scancode(val: u8) -> Option<Scancode> {
    match val {
        0x1 => Some(Scancode::Num1),
        0x2 => Some(Scancode::Num2),
        0x3 => Some(Scancode::Num3),
        0xC => Some(Scancode::Num4),
        0x4 => Some(Scancode::Q),
        0x5 => Some(Scancode::W),
        0x6 => Some(Scancode::E),
        0xD => Some(Scancode::R),
        0x7 => Some(Scancode::A),
        0x8 => Some(Scancode::S),
        0x9 => Some(Scancode::D),
        0xE => Some(Scancode::F),
        0xA => Some(Scancode::Z),
        0x0 => Some(Scancode::X),
        0xB => Some(Scancode::C),
        0xF => Some(Scancode::V),
        _ => None,
    }
}

fn scancode_to_keypad(scancode: &Scancode) -> Option<u8> {
    match scancode {
        Scancode::Num1 => Some(0x1),
        Scancode::Num2 => Some(0x2),
        Scancode::Num3 => Some(0x3),
        Scancode::Num4 => Some(0xC),
        Scancode::Q => Some(0x4),
        Scancode::W => Some(0x5),
        Scancode::E => Some(0x6),
        Scancode::R => Some(0xD),
        Scancode::A => Some(0x7),
        Scancode::S => Some(0x8),
        Scancode::D => Some(0x9),
        Scancode::F => Some(0xE),
        Scancode::Z => Some(0xA),
        Scancode::X => Some(0x0),
        Scancode::C => Some(0xB),
        Scancode::V => Some(0xF),
        _ => None,
    }
}
