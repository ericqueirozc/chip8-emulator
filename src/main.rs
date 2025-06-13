use minifb::{Key, Window, WindowOptions};
use std::fs::File;
use std::io::Read;
use std::path;
use std::thread::sleep;
use std::time::Duration;

use std::collections::HashMap;

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
fn build_keymap() -> HashMap<Key, u8> {
    use Key::*;
    [
        (Key1, 0x1),
        (Key2, 0x2),
        (Key3, 0x3),
        (Key4, 0xC),
        (Q, 0x4),
        (W, 0x5),
        (E, 0x6),
        (R, 0xD),
        (A, 0x7),
        (S, 0x8),
        (D, 0x9),
        (F, 0xE),
        (Z, 0xA),
        (X, 0x0),
        (C, 0xB),
        (V, 0xF),
    ]
    .into_iter()
    .collect()
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

    fn load_test_instructions(&mut self) {
        let program: [u8; 12 + 5] = [
            0x00, 0xE0, // CLS
            0xF0, 0x0A, // LD V0, K
            0x60, 0x00, // LD V0, 0
            0x61, 0x00, // LD V1, 0
            0xA3, 0x00, // LD I, 0x200
            0xD0, 0x15, // DRW V0, V1, 5
            // Dados do sprite (a partir de 0x200)
            0xF0, 0x90, 0x90, 0x90, 0xF0,
        ];

        for (i, &byte) in program[..10].iter().enumerate() {
            self.memory[0x200 + i] = byte;
        }
        for (i, &byte) in program[12..].iter().enumerate() {
            self.memory[0x300 + i] = byte;
        }
    }

    //Segundo a especificação os timers diminuiem uma unidade a cada 60Hz e isso é usado para coisas como animção e música
    pub fn tick_timers(&mut self) {
        if self.delay_timer > 0 {
            self.delay_timer -= 1;
        }

        if self.sound_timer > 0 {
            self.sound_timer -= 1;
            //Trigger beep
            println!("BEEEEEP!")
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

            0xE000 => match opcode & 0x00FF {
                0x9E => {
                    //Pula a próxima instrução caso o botão com o valor de Vx estiver pressionado
                    let x: usize = ((opcode & 0x0F00) >> 8) as usize;
                    let key = self.v[x] as usize;
                    if self.keypad[key] {
                        self.pc += 2;
                    }
                }

                0xA1 => {
                    // Pula a próxima instrução caso o botão com o valor de Vx NÃO estiver pressionado
                    let x = ((opcode & 0x0F00) >> 8) as usize;
                    let key = self.v[x] as usize;
                    if !self.keypad[key] {
                        self.pc += 2;
                    }
                }

                _ => println!("Unknown 0xE op: {:04X}", opcode),
            },

            //Set the I register to address NNN
            0xA000..=0xAFFF => {
                let addr = (opcode & 0x0FFF) as u16;
                self.i = addr;
                println!("Executed LD I, {:#05X}", addr);
            }

            //Return from subroutine
            0x00EE => {
                if self.sp == 0 {
                    println!("Stack underflow!");
                    return;
                }
                //Decrementa o ponteiro da pilha para pegar o endereço do topo
                //Isso é necessário pois o ponteiro da pilha aponta para o próximo endereço livre

                self.sp -= 1;

                //Recupera o endereço do topo da pilha que representa o endereço de retorno
                let return_addr = self.stack[self.sp as usize];

                //Seta o pc para o endereço de retorno
                self.pc = return_addr;
                println!(
                    "Executed RET (Return from subroutine) to {:#05X}",
                    return_addr
                );
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

            // 2NNN: CALL NNN
            //Call subroutine at NNN (push current PC to stack).
            //Chama a subrotina no endereço NNN. Colocar no
            0x2000..0x2FFF => {
                let addr = opcode & 0x0FFF;
                //Coloca o endereço atual do pc no topo da pilha que é o endereço para qual retornará ao final da subrotina
                //A pilha guarda os endereços das subrotinas que estão sendo executadas
                self.stack[self.sp as usize] = self.pc;
                //Incrementa o ponteiro da pilha para caso uma nova subrotina seja chamada ela seja colocada no topo
                self.sp += 1;
                //Seta o pc para o endereço da subrotina
                self.pc = addr;
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

            0x8000..=0x8FFF => {
                let x = ((opcode & 0x0F00) >> 8) as usize;
                let y = ((opcode & 0x00F0) >> 4) as usize;

                match opcode & 0x000F {
                    0x0 => {
                        self.v[x] = self.v[y];
                        println!("Executed LD V{:X}, V{:X}", x, y);
                    }
                    0x1 => {
                        self.v[x] |= self.v[y];
                        println!("Executed OR V{:X}, V{:X}", x, y);
                    }
                    0x2 => {
                        self.v[x] &= self.v[y];
                        println!("Executed AND V{:X}, V{:X}", x, y);
                    }
                    0x3 => {
                        self.v[x] ^= self.v[y];
                        println!("Executed XOR V{:X}, V{:X}", x, y);
                    }
                    0x4 => {
                        let (result, carry) = self.v[x].overflowing_add(self.v[y]);
                        self.v[x] = result;
                        self.v[0xF] = if carry { 1 } else { 0 };
                        println!("Executed ADD V{:X}, V{:X} (with carry)", x, y);
                    }
                    0x5 => {
                        let (result, borrow) = self.v[x].overflowing_sub(self.v[y]);
                        self.v[x] = result;
                        self.v[0xF] = if borrow { 0 } else { 1 };
                        println!("Executed SUB V{:X}, V{:X}", x, y);
                    }
                    0x6 => {
                        //Salva o bit menos significativo em VF
                        self.v[0xF] = self.v[x] & 0x01;
                        //Move o valor de VX 1 bit para direita
                        self.v[x] >>= 1;
                        println!("Executed SHR V{:X}", x);
                    }
                    0x7 => {
                        let (result, borrow) = self.v[y].overflowing_sub(self.v[x]);
                        self.v[x] = result;
                        self.v[0xF] = if borrow { 0 } else { 1 };
                        println!("Executed SUB V{:X}, V{:X}", x, y);
                    }
                    0xE => {
                        //Salva o bit mais significativo em VF
                        self.v[0xF] = (self.v[x] & 0x08) >> 7;
                        //Move o valor de VX 1 bit para esquerda
                        self.v[x] <<= 1;
                        println!("Executed SHR V{:X}", x);
                    }
                    _ => println!("Unknown 0x8 instruction: {:04X}", opcode),
                }
            }

            //Draw Sprites
            //0xDXYN
            0xD000..=0xDFFF => {
                //recuperando o valor de x e movendo 8 bits para direita para ter o valor "puro"
                let x = self.v[((opcode & 0x0F00) >> 8) as usize] as u16;
                //recuperando o valor de x e movendo 8 bits para direita para ter o valor "puro"
                let y = self.v[((opcode & 0x00F0) >> 4) as usize] as u16;
                //recuperando a altura do sprite. A altura do sprite também representa seu tamanho em bytes
                //pois para cada unidade de altura tem um byte (8 bits - 10101010) que será desenhado horizontalmente
                //pois o sprite tem apenas 1 byte de largura
                let height = (opcode & 0x000F) as u16;

                self.v[0xF] = 0; // Reset VF

                for byte in 0..height {
                    //o modulo é usado para que caso a coordenada passe do limite da tela [VIDEO_HEIGHT] o pixel comece novamente em baixo ao invés de apenas n aparecer
                    let y_coord = (y + byte) % VIDEO_HEIGHT as u16;
                    //os bytes sprite que será desenhado está no endereço de memoria I e vai até I+N (ou I + height)
                    let sprite = self.memory[(self.i + byte) as usize];

                    //loop para desenhar a linha
                    for bit in 0..8 {
                        let x_coord = (x + bit) % VIDEO_WIDTH as u16;
                        //index para acessar o pixel no video buffer. Como estamos trabalhando com um array de uma dimenção
                        //para acessar o pixel (x,y) precisamos acessar o index [width*y+x]
                        let index = (y_coord * VIDEO_WIDTH as u16 + x_coord) as usize;

                        //para recuperar o bit atual que será desenhado
                        let sprite_pixel = (sprite >> (7 - bit)) & 1;

                        //pega o estado atual daquela posição no buffer de video
                        let screen_pixel = self.video[index];

                        //Ele checa se o sprite_pixel é 1, Se for 1 ele faz um toggle(se tiver ligado desliga, se tiver desligado liga)
                        //do pixel no video buffer. Se for 0 não faz nada.
                        // XOR the sprite pixel onto the screen
                        self.video[index] ^= sprite_pixel != 0;

                        //Se o pixel for desligado precisamos ligar a flag de colizão do register VF
                        // Set VF if a pixel was unset (collision)
                        if screen_pixel && !self.video[index] {
                            self.v[0xF] = 1;
                        }
                    }
                }
                println!("Coloriu");
            }

            0xF000..=0xFFFF => match opcode & 0x00FF {
                //Timers -------------------------------------------

                // Vai salvar o valor do delay_timer em VX
                0x07 => {
                    let x = ((opcode & 0x0F00) >> 8) as usize;
                    self.v[x] = self.delay_timer;
                    println!("Executed LD V{:X}, DT", x);
                }

                //Define o delay_timer com valor de VX
                0x15 => {
                    let x = ((opcode & 0x0F00) >> 8) as usize;
                    self.delay_timer = self.v[x];
                    println!("Executed LD DT, V{:X}", x);
                }

                //Define o sound_timer com o valor de VX
                0x18 => {
                    let x = ((opcode & 0x0F00) >> 8) as usize;
                    self.sound_timer = self.v[x];
                    println!("Executed LD ST, V{:X}", x);
                }

                //IO---------------

                //Soma o valor de VX ao de I
                0x1E => {
                    let x = ((opcode & 0x0F00) >> 8) as usize;
                    self.i = self.i.wrapping_add(self.v[x] as u16);
                }

                0x0A => {
                    let x = ((opcode & 0x0F00) >> 8) as usize;
                    if let Some(pressed_key) = self.keypad.iter().position(|&k| k) {
                        self.v[x] = pressed_key as u8;
                        println!("Executed LD V{:X}, K", x);
                    } else {
                        self.pc -= 2;
                    }
                }

                // Seta I com o endereço de um character armazenado em Vx
                // Fontes normalmente ocupam 5 bytes e são armazenadas a partir do endereço 0x000
                0x29 => {
                    let x = ((opcode & 0x0F00) >> 8) as usize;
                    let digit = self.v[x] as u16;
                    self.i = digit * 5;
                    println!("Executed LD F, V{:X} (char sprite addr)", x);
                }

                // Armazena o valor de Vx em formato decimal nos endereços I, I+1 e I+2
                0x33 => {
                    let x = ((opcode & 0x0F00) >> 8) as usize;
                    let vx = self.v[x];
                    self.memory[self.i as usize] = vx / 100;
                    self.memory[(self.i + 1) as usize] = (vx % 100) / 10;
                    self.memory[(self.i + 2) as usize] = vx % 10;
                    println!("Executed LD B, V{:X}", x);
                }

                //Armazena os valores de V0 até Vx na memoria a partir do endereço I
                0x55 => {
                    let x = ((opcode & 0x0F00) >> 8) as usize;
                    for i in 0..=x {
                        self.memory[(self.i + i as u16) as usize] = self.v[i];
                    }
                    println!("Executed LD [I], V0..V{:X}", x);
                }

                //Armazena os valores a partir de I até x em V0 até Vx
                0x65 => {
                    let x = ((opcode & 0x0F00) >> 8) as usize;
                    for i in 0..=x {
                        self.v[i] = self.memory[(self.i + i as u16) as usize]
                    }
                    println!("Executed LD V0..V{:X}, [I]", x);
                }

                _ => println!("Unknown 0xF instruction: {:04X}", opcode),
            },

            // Pula a próxima instrução caso Vx seja igual a kk
            0x3000..=0x3FFF => {
                let x = ((opcode & 0x0F00) >> 8) as usize;
                let kk = (opcode & 0x00FF) as u8;
                if self.v[x] == kk {
                    self.pc += 2;
                }
                println!("Executed SE V{:X}, {:#X}", x, kk);
            }

            // Pula a próxima instrução caso Vx seja diferente a kk
            0x4000..=0x4FFF => {
                let x = ((opcode & 0x0F00) >> 8) as usize;
                let kk = (opcode & 0x00FF) as u8;
                if self.v[x] != kk {
                    self.pc += 2;
                }
                println!("Executed SNE V{:X}, {:#X}", x, kk);
            }

            // Pula a próxima instrução caso Vx seja igual a Vy
            0x5000..=0x5FFF => {
                if (opcode & 0x000F) == 0x0 {
                    let x = ((opcode & 0x0F00) >> 8) as usize;
                    let y = ((opcode & 0x00F0) >> 4) as usize;
                    if self.v[x] == self.v[y] {
                        self.pc += 2;
                    }
                    println!("Executed SE V{:X}, V{:#X}", x, y);
                } else {
                    println!("Unknown 0x5 instruction: {:04X}", opcode);
                }
            }

            // Pula a próxima instrução caso Vx seja diferente a Vy
            0x9000..=0x9FFF => {
                if (opcode & 0x000F) == 0x0 {
                    let x = ((opcode & 0x0F00) >> 8) as usize;
                    let y = ((opcode & 0x00F0) >> 4) as usize;
                    if self.v[x] != self.v[y] {
                        self.pc += 2;
                    }
                    println!("Executed SE V{:X}, V{:#X}", x, y);
                } else {
                    println!("Unknown 0x5 instruction: {:04X}", opcode);
                }
            }

            //Salva em Vx um (número aleatório de 0 a 255 AND kk)
            0xC000..=0xCFFF => {
                let x = ((opcode & 0x0F00) >> 8) as usize;
                let kk = (opcode & 0x00FF) as u8;

                let rnd: u8 = rand::random();
                self.v[x] = rnd & kk;

                println!("Executed RND V{:X}, {:#X} → random {:#X}", x, kk, rnd);
            }

            _ => print!("Unknown opcode! {:#06X}", opcode),
        }

        //Como dois bytes são lidos de uma vez o Program Counter tem que pular dois endereços de memoria de uma vez
        self.pc += 2;
    }
}
fn main() {
    let mut chip8 = Chip8::new();

    chip8.load_rom("roms/random_number_test.ch8");
    // chip8.load_test_instructions();

    let width = VIDEO_WIDTH;
    let height = VIDEO_HEIGHT;
    let keymap = build_keymap();

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
        chip8.tick_timers();
        chip8.keypad = [false; 16]; // limpa o estado das teclas

        for key in window.get_keys_pressed(minifb::KeyRepeat::Yes) {
            if let Some(&chip8_index) = keymap.get(&key) {
                chip8.keypad[chip8_index as usize] = true;
            }
        }
        sleep(Duration::from_millis(16));
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
