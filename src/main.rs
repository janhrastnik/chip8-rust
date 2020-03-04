use minifb::{Key, KeyRepeat, Window, WindowOptions};
use rand::Rng;
use std::time::{Duration, Instant};

fn main() {
    let fontset = vec![
        0xF0, 0x90, 0x90, 0x90, 0xF0, //0
        0x20, 0x60, 0x20, 0x20, 0x70, //1
        0xF0, 0x10, 0xF0, 0x80, 0xF0, //2
        0xF0, 0x10, 0xF0, 0x10, 0xF0, //3
        0x90, 0x90, 0xF0, 0x10, 0x10, //4
        0xF0, 0x80, 0xF0, 0x10, 0xF0, //5
        0xF0, 0x80, 0xF0, 0x90, 0xF0, //6
        0xF0, 0x10, 0x20, 0x40, 0x40, //7
        0xF0, 0x90, 0xF0, 0x90, 0xF0, //8
        0xF0, 0x90, 0xF0, 0x10, 0xF0, //9
        0xF0, 0x90, 0xF0, 0x90, 0x90, //A
        0xE0, 0x90, 0xE0, 0x90, 0xE0, //B
        0xF0, 0x80, 0x80, 0x80, 0xF0, //C
        0xE0, 0x90, 0x90, 0x90, 0xE0, //D
        0xF0, 0x80, 0xF0, 0x80, 0xF0, //E
        0xF0, 0x80, 0xF0, 0x80, 0x80, //F
    ];
    let mut chip8 = Chip8::new();
    chip8.load_rom("roms/INVADERS");
    chip8.load_fonts(fontset);

    let mut window = Window::new("Chip8 Emulator", 640, 320, WindowOptions::default())
        .unwrap_or_else(|e| {
            panic!("{}", e);
        });

    window.limit_update_rate(Some(std::time::Duration::from_micros(14000)));
    let mut time = Instant::now();

    while window.is_open() && !window.is_key_down(Key::Escape) {
        chip8.run();
        chip8.redraw_flag = true;
        window.get_keys_pressed(KeyRepeat::Yes).map(|keys| {
            let mut key = None;
            if !keys.is_empty() {
                key = match keys[0] {
                    Key::Key1 => Some(0x1),
                    Key::Key2 => Some(0x2),
                    Key::Key3 => Some(0x3),
                    Key::Key4 => Some(0xC),
                    Key::Q => Some(0x4),
                    Key::W => Some(0x5),
                    Key::E => Some(0x6),
                    Key::R => Some(0xD),
                    Key::A => Some(0x7),
                    Key::S => Some(0x8),
                    Key::D => Some(0x9),
                    Key::F => Some(0xE),
                    Key::Y => Some(0xA),
                    Key::X => Some(0x0),
                    Key::C => Some(0xB),
                    Key::V => Some(0xF),
                    _ => None,
                };
            }
            if key.is_some() || Instant::now() - time >= Duration::from_millis(200) {
                chip8.pressed_key = key;
                time = Instant::now();
            }
        });
        let mut buffer = chip8.display;
        for i in 0..buffer.len() {
            if buffer[i] == 1 {
                buffer[i] = 0xffffff;
            }
        }
        if chip8.redraw_flag {
            window.update_with_buffer(buffer.as_ref(), 64, 32).unwrap();
            chip8.redraw_flag = false;
        }
    }
}

#[derive(Debug)]
pub struct Opcode {
    leading: u8,
    x: u8,
    y: u8,
    n: u8,
    nnn: u16,
    kk: u8,
}

pub struct Chip8 {
    counter: u16,
    stack_pointer: u16,
    stack: [u16; 16],
    address_register: u16,
    memory: [u8; 4096],
    data_registers: [u8; 16],
    delay_timer: u8,
    sound_timer: u8,
    redraw_flag: bool,
    display: [u32; 64 * 32],
    pressed_key: Option<u8>,
}

impl Chip8 {
    fn new() -> Self {
        Chip8 {
            counter: 512,
            stack_pointer: 0,
            stack: [0; 16],
            address_register: 0,
            memory: [0; 4096],
            data_registers: [0; 16],
            delay_timer: 0,
            sound_timer: 0,
            redraw_flag: false,
            display: [0; 64 * 32],
            pressed_key: None,
        }
    }

    fn load_rom(&mut self, filepath: &str) {
        let content = std::fs::read(filepath).expect("unable to read");

        for (i, u) in content.iter().enumerate() {
            self.memory[i + 512] = *u;
        }
    }

    fn load_fonts(&mut self, fonts: Vec<u8>) {
        for (i, font) in fonts.iter().enumerate() {
            self.memory[i] = *font;
        }
    }

    fn run(&mut self) {
        let op = ((self.memory[self.counter as usize] as u16) << 8)
            | (self.memory[(self.counter + 1) as usize] as u16);

        let opcode = Opcode {
            leading: ((op & 0xF000) >> 12) as u8,
            x: ((op & 0x0F00) >> 8) as u8,
            y: ((op & 0x00F0) >> 4) as u8,
            n: (op & 0x000F) as u8,
            nnn: (op & 0x0FFF) as u16,
            kk: (op & 0x000FF) as u8,
        };

        match opcode.leading {
            0x0 => match opcode.nnn {
                0x00e0 => {
                    // clear the display
                    self.display = [0; 64 * 32];
                    self.redraw_flag = true;
                    self.counter += 2;
                }
                0x00ee => {
                    // return from a subroutine
                    self.stack_pointer -= 1;
                    self.counter = self.stack[self.stack_pointer as usize];
                    self.counter += 2;
                }
                _ => {
                    // jump to addr, not needed in modern interpreters
                }
            },
            0x1 => {
                // jump to location nnn
                self.counter = opcode.nnn;
            }
            0x2 => {
                // call subroutine at nnn
                self.stack[self.stack_pointer as usize] = self.counter;
                self.stack_pointer += 1;
                self.counter = opcode.nnn;
            }
            0x3 => {
                //  Skip next instruction if Vx = kk.
                if self.data_registers[opcode.x as usize] == opcode.kk {
                    self.counter += 4;
                } else {
                    self.counter += 2;
                }
            }
            0x4 => {
                //  Skip next instruction if Vx != kk.
                if self.data_registers[opcode.x as usize] != opcode.kk {
                    self.counter += 4;
                } else {
                    self.counter += 2;
                }
            }
            0x5 => {
                //  Skip next instruction if Vx = Vy.
                if self.data_registers[opcode.y as usize] == self.data_registers[opcode.x as usize]
                {
                    self.counter += 4;
                } else {
                    self.counter += 2;
                }
            }
            0x6 => {
                //  Set Vx = kk.
                self.data_registers[opcode.x as usize] = opcode.kk;
                self.counter += 2;
            }
            0x7 => {
                //  Set Vx = Vx + kk.
                let sum = self.data_registers[opcode.x as usize].wrapping_add(opcode.kk);
                self.data_registers[opcode.x as usize] = sum;
                self.counter += 2;
            }
            0x8 => match opcode.n {
                0x0 => {
                    //  Set Vx = Vy.
                    self.data_registers[opcode.x as usize] = self.data_registers[opcode.y as usize];
                    self.counter += 2;
                }
                0x1 => {
                    //  Set Vx = Vx OR Vy.
                    self.data_registers[opcode.x as usize] |=
                        self.data_registers[opcode.y as usize];
                    self.counter += 2;
                }
                0x2 => {
                    //  Set Vx = Vx AND Vy.
                    self.data_registers[opcode.x as usize] &=
                        self.data_registers[opcode.y as usize];
                    self.counter += 2;
                }
                0x3 => {
                    //  Set Vx = Vx XOR Vy.
                    self.data_registers[opcode.x as usize] ^=
                        self.data_registers[opcode.y as usize];
                    self.counter += 2;
                }
                0x4 => {
                    // Set Vx = Vx + Vy, set VF = carry.
                    let value: u16 = (self.data_registers[opcode.x as usize] as u16)
                        + (self.data_registers[opcode.y as usize] as u16);
                    self.data_registers[opcode.x as usize] = value as u8;
                    if value > 255 {
                        self.data_registers[15] = 1;
                    } else {
                        self.data_registers[15] = 0;
                    }
                    self.counter += 2;
                }
                0x5 => {
                    //  Set Vx = Vx - Vy, set VF = NOT borrow.
                    let diff: i8 = self.data_registers[opcode.x as usize] as i8
                        - self.data_registers[opcode.y as usize] as i8;
                    self.data_registers[opcode.x as usize] = diff as u8;
                    if diff < 0 {
                        self.data_registers[15] = 1;
                    } else {
                        self.data_registers[15] = 0;
                    }
                    self.counter += 2;
                }
                0x6 => {
                    //  Set Vx = Vx SHR 1.
                    self.data_registers[15] = self.data_registers[opcode.x as usize] & 1;
                    self.data_registers[opcode.x as usize] >>= 1;
                    self.counter += 2;
                }
                0x7 => {
                    //  Set Vx = Vy - Vx, set VF = NOT borrow.
                    let diff: i8 = self.data_registers[opcode.y as usize] as i8
                        - self.data_registers[opcode.x as usize] as i8;
                    self.data_registers[opcode.x as usize] = diff as u8;
                    if diff < 0 {
                        self.data_registers[15] = 1;
                    } else {
                        self.data_registers[15] = 0;
                    }
                    self.counter += 2;
                }
                0xe => {
                    //  Set Vx = Vx SHL 1.
                    self.data_registers[15] = self.data_registers[opcode.x as usize] >> 7;
                    self.data_registers[opcode.x as usize] <<= 1;
                    self.counter += 2;
                }
                _ => panic!("unexpected opcode"),
            },
            0x9 => {
                //  Skip next instruction if Vx != Vy.
                if self.data_registers[opcode.x as usize] != self.data_registers[opcode.y as usize]
                {
                    self.counter += 4;
                } else {
                    self.counter += 2;
                }
            }
            0xa => {
                //  Set I = nnn.
                self.address_register = opcode.nnn;
                self.counter += 2;
            }
            0xb => {
                //  Jump to location nnn + V0.
                self.counter = opcode.nnn + self.data_registers[0] as u16;
            }
            0xc => {
                //  Set Vx = random byte AND kk.
                let mut rng = rand::thread_rng();
                self.data_registers[opcode.x as usize] = rng.gen::<u8>() & opcode.kk;
                self.counter += 2;
            }
            0xd => {
                //  Display n-byte sprite starting at memory location I at (Vx, Vy), set VF = collision.
                self.data_registers[15] = 0;
                for byte in 0..opcode.n {
                    let y = (self.data_registers[opcode.y as usize] + byte) % 32;
                    for bit in 0..8 {
                        let x = (self.data_registers[opcode.x as usize] + bit) % 64;
                        let color = (self.memory[(self.address_register + byte as u16) as usize]
                            >> (7 - bit))
                            & 1;
                        self.data_registers[15] |=
                            color & self.display[y as usize * 64 + x as usize] as u8;

                        self.display[y as usize * 64 + x as usize] ^= color as u32;
                    }
                }
                self.redraw_flag = true;
                self.counter += 2;
            }
            0xe => match opcode.kk {
                0x9e => {
                    //  Skip next instruction if key with the value of Vx is pressed.
                    let register_key = self.data_registers[opcode.x as usize];
                    if self.pressed_key.is_some() && register_key == self.pressed_key.unwrap() {
                        self.counter += 4;
                    } else {
                        self.counter += 2;
                    }
                }
                0xa1 => {
                    //  Skip next instruction if key with the value of Vx is not pressed.
                    let register_key = self.data_registers[opcode.x as usize];
                    if self.pressed_key.is_some() && register_key != self.pressed_key.unwrap() {
                        self.counter += 4;
                    } else {
                        self.counter += 2;
                    }
                }
                _ => panic!("unexpected opcode"),
            },
            0xf => match opcode.kk {
                0x07 => {
                    //  Set Vx = delay timer value.
                    self.data_registers[opcode.x as usize] = self.delay_timer;
                    self.counter += 2;
                }
                0x0a => {
                    //  Wait for a key press, store the value of the key in Vx.
                    if self.pressed_key.is_some() {
                        self.data_registers[opcode.x as usize] = self.pressed_key.unwrap();
                        self.counter += 2;
                    }
                    self.redraw_flag = true;
                }
                0x15 => {
                    //  Set delay timer = Vx.
                    self.delay_timer = self.data_registers[opcode.x as usize];
                    self.counter += 2;
                }
                0x18 => {
                    //  Set sound timer = Vx.
                    self.sound_timer = self.data_registers[opcode.x as usize];
                    self.counter += 2;
                }
                0x1e => {
                    //  Set I = I + Vx. In case of overflow set VF to 1.
                    self.address_register += self.data_registers[opcode.x as usize] as u16;
                    self.data_registers[15] = if self.address_register > 0x0F00 { 1 } else { 0 };
                    self.counter += 2;
                }
                0x29 => {
                    //  Set I = location of sprite for digit Vx.
                    self.address_register = (self.data_registers[opcode.x as usize] * 5) as u16; // font is 4x5
                    self.counter += 2;
                }
                0x33 => {
                    //  Store BCD representation of Vx in memory locations I, I+1, and I+2.
                    self.memory[self.address_register as usize] =
                        self.data_registers[opcode.x as usize] / 100;
                    self.memory[self.address_register as usize + 1] =
                        (self.data_registers[opcode.x as usize] % 100) / 10;
                    self.memory[self.address_register as usize + 2] =
                        self.data_registers[opcode.x as usize] % 10;
                    self.counter += 2;
                }
                0x55 => {
                    //  Store registers V0 through Vx in memory starting at location I.
                    for i in 0..opcode.x + 1 {
                        self.memory[(self.address_register + i as u16) as usize] =
                            self.data_registers[opcode.x as usize];
                    }
                    self.counter += 2;
                }
                0x65 => {
                    //  Read registers V0 through Vx from memory starting at location I.
                    for i in 0..opcode.x + 1 {
                        self.data_registers[opcode.x as usize] =
                            self.memory[(self.address_register + i as u16) as usize];
                    }
                    self.counter += 2;
                }
                _ => panic!("unexpected opcode"),
            },
            _ => panic!("unexpected leading number"),
        };
        if self.delay_timer > 0 {
            self.delay_timer -= 1;
        }
        if self.sound_timer > 0 {
            self.sound_timer -= 1;
        }
    }
}
