use std::fs::File;
use std::io::Read;
use std::path;

const MEMORY_SIZE: usize = 4092;
const REGISTER_COUNT: usize = 16;
const STACK_SIZE: usize = 16;
const KEYPAD_SIZE: usize = 16;
const VIDEO_WIDTH: usize = 64;
const VIDEO_HEIGHT: usize = 32;
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

    match chip8.load_rom("roms/IBM Logo.ch8") {
        Ok(()) => println!("ROM loaded succefully!"),
        Err(e) => eprintln!("Failed to load rom {}", e),
    }
}
