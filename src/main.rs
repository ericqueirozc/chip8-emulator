use minifb::{Key, Window, WindowOptions};
use std::fs::File;
use std::io::Read;
use std::path;

const MEMORY_SIZE: usize = 4092;
const REGISTER_COUNT: usize = 16;
const STACK_SIZE: usize = 16;
const KEYPAD_SIZE: usize = 16;
pub const VIDEO_WIDTH: usize = 64;
pub const VIDEO_HEIGHT: usize = 32;
pub const DISPLAY_SCALE: usize = 10;
const START_ADDRESS: usize = 0x200;

struct Chip8 {
    // 4K memory
    memory: [u8; MEMORY_SIZE],

    // 16 general purpose 8-bit registers: V0 to VF
    v: [u8; REGISTER_COUNT],

    // Index register (16-bit)
    i: u16,

    // Program Counter starts at 0x200
    pc: u16,

    // Stack for subrotines calls
    stack: [u16; STACK_SIZE],
    sp: u8,

    // Timers (decrement at 60hz)
    delay_timer: u8,
    sound_timer: u8,

    // Input keypad (16 keys)
    keypad: [bool; KEYPAD_SIZE],

    // Video Buffer
    video: [bool; VIDEO_WIDTH * VIDEO_HEIGHT],
}

impl Chip8 {
    fn new() -> Self {
        Chip8 {
            memory: [0; MEMORY_SIZE],
            v: [0; REGISTER_COUNT],
            i: 0,
            pc: 0x200, // CHIP-8 programs start at 0x200
            stack: [0; STACK_SIZE],
            sp: 0,
            delay_timer: 0,
            sound_timer: 0,
            keypad: [false; KEYPAD_SIZE],
            video: [false; VIDEO_WIDTH * VIDEO_HEIGHT],
        }
    }

    fn cycle(&mut self) {
        //FETCH

        let high_byte: u16 = self.memory[self.pc as usize] as u16;
        let low_byte: u16 = self.memory[(self.pc + 1) as usize] as u16;
        let opcode: u16 = (high_byte << 8) | low_byte;

        match opcode {
            //Limpa a tela de toda informação
            //CLS - Clear Screen
            0x00E0 => {
                self.video = [false; VIDEO_WIDTH * VIDEO_HEIGHT];
                println!("Executed CLS (Clear Screen)");
            }

            //Set the I register to address NNN
            0xA000..=0xAFFF => {
                let addr = (opcode & 0x0FFF) as u16;
                self.i = addr;
                println!("Executed LD I, {:#05X}", addr);
            }

            //1nnn - Jump to address nnn
            //Instrução de setar um valor para o pc
            0x1000..=0x1FFF => {
                let addr = opcode & 0x0FFF;
                self.pc = addr;
                println!("Executed JP {:03X}", addr);
                //Encerra o fluxo aqui pois se ele passar ele vai incrementar o pc no final do match
                //o que sairia do endereço que acabou de ser gerado
                return;
            }

            //6xkk - Set Vx = kk
            //Passa um determinado valor para um register
            0x6000..=0x6FFF => {
                let x: usize = ((opcode & 0x0F00) >> 8) as usize;
                let kk: u8 = (opcode & 0x00FF) as u8;
                self.v[x] = kk;
                println!("Executed LD V{:X}, {:#X}", x, kk);
            }

            //7xkk - Set Vx = Vx + kk
            //Instrução que faz o somatório do valor atual do register com o valor em kk
            0x7000..=0x7FFF => {
                let x = ((opcode & 0x0F00) >> 8) as usize;
                let kk = (opcode & 0x00FF) as u8;
                self.v[x] = self.v[x].wrapping_add(kk);
                println!("Executed ADD V{:X}, {:#X}", x, kk);
            }

            //Draw Sprites
            0xD000..=0xDFFF => {
                let x = self.v[((opcode & 0x0F00) >> 8) as usize] as u16;
                let y = self.v[((opcode & 0x00F0) >> 8) as usize] as u16;
                let height = (opcode & 0x000F) as u16;

                self.v[0xF] = 0; // Reset VF (que é utilizado para collizão(ainda não sei como))

                for byte in 0..height {
                    let y_coord = (y + byte) % VIDEO_HEIGHT as u16;
                    let sprite = self.memory[(self.i + byte) as usize];

                    for bit in 0..8 {
                        let x_coord = (x + bit) % VIDEO_WIDTH as u16;
                        let index = (y_coord * VIDEO_WIDTH as u16 + x_coord) as usize;

                        let sprite_pixel = (sprite >> (7 - bit)) & 1;
                        let screen_pixel = self.video[index];

                        // XOR the sprite pixel onto the screen
                        self.video[index] ^= sprite_pixel != 0;

                        // Set VF if a pixel was unset (collision)
                        if screen_pixel && !self.video[index] {
                            self.v[0xF] = 1;
                        }
                    }
                }
                println!("Executed DRW Vx, Vy, N ({}, {}, {})", x, y, height);
            }

            _ => print!("Unknown opcode! {:#06X}", opcode),
        }

        //Como dois bytes são lidos de uma vez o Program Counter tem que pular dois endereços de memoria de uma vez
        self.pc += 2;
    }

    //O Result serve para indicar que a função pode falhar e retornar um valor de sucesso ou um erro

    //&mut self significa que o estado da objeto atual será mudado

    //usasse &str na tipagem do parâmetro ou invés de apenas str para referenciar uma string de tamanho definido
    //onde str não pode ser usado diretamente pois é um tipo de tamanho dinâmico (DST) e representa uma sequência
    //de texto imutavél sem tamanho definido
    fn load_rom(&mut self, filename: &str) -> std::io::Result<()> {
        //como o processo de abrir o file pode falhar nos colocamos o ? no final da linha
        //é um shortcut para
        // let mut file =  match File::open(filename){
        //     Ok(f) => f,
        //     Err(e) => return Err(e),
        // }
        //Onde se der sucesso ele retorna o resultado esperado, senão, o erro
        let mut file = File::open(filename)?;
        //Cria um vetor de tamanho dinâmico pois não sabemos o tamanho na ROM
        let mut buffer = Vec::new();
        //é necessário passar &mut pois a função mudadará o estado do buffer
        file.read_to_end(&mut buffer)?;
        //um for in onde a gente tem o elemento e o index ao mesmo tempo!
        //o .iter() fazer que iteramos por todos elementos de buffer
        //o .enumerate fazer com que retorne tanto o valor quanto o index equivalente
        //o & em &byte é usado para DESREFERENCIAR o valor do byte que vem como &u8. ao usar &byte o &u8 já retorna como u8
        //meio confuso a principio para eu que nunca programei low level.
        for (i, &byte) in buffer.iter().enumerate() {
            self.memory[START_ADDRESS + i] = byte;
        }

        Ok(())
    }

    fn read_byte(&self, addr: usize) -> u8 {
        self.memory[addr]
    }

    fn write_byte(&mut self, addr: usize, value: u8) {
        self.memory[addr] = value;
    }
}

fn main() {
    let mut chip8 = Chip8::new();

    chip8.load_rom("roms/IBM Logo.ch8");

    let width = VIDEO_WIDTH;
    let height = VIDEO_HEIGHT;

    let mut window = Window::new(
        "CHIP-8 Emulator",
        width * DISPLAY_SCALE,
        height * DISPLAY_SCALE,
        WindowOptions {
            scale: minifb::Scale::X1,
            ..WindowOptions::default()
        },
    )
    .unwrap_or_else(|e| panic!("{}", e));

    // Frame buffer for minifb (32-bit color)
    let mut buffer: Vec<u32> = vec![0; width * height];

    while window.is_open() && !window.is_key_down(Key::Escape) {
        chip8.cycle();

        // Update buffer: map chip8.video (bool) to white or black pixels
        for (i, &pixel_on) in chip8.video.iter().enumerate() {
            buffer[i] = if pixel_on { 0xFFFFFF } else { 0x000000 };
        }

        // Expand to scale
        let scaled_buffer = scale_buffer(&buffer, width, height, DISPLAY_SCALE);

        window
            .update_with_buffer(
                &scaled_buffer,
                width * DISPLAY_SCALE,
                height * DISPLAY_SCALE,
            )
            .unwrap();
    }
}

fn scale_buffer(buffer: &[u32], width: usize, height: usize, scale: usize) -> Vec<u32> {
    let scaled_width = width * scale;
    let scaled_height = height * scale;

    let mut scaled = vec![0u32; scaled_width * scaled_height];

    for y in 0..height {
        for x in 0..width {
            let color = buffer[y * width + x];
            for dy in 0..scale {
                for dx in 0..scale {
                    let sx = x * scale + dx;
                    let sy = y * scale + dy;
                    let scaled_index = sy * scaled_width + sx;
                    scaled[scaled_index] = color;
                }
            }
        }
    }

    scaled
}
