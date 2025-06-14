# CHIP-8 Emulator in Rust

Um emulador simples do sistema virtual **CHIP-8**, desenvolvido em Rust como projeto educacional para explorar conceitos de programação de baixo nível, manipulação de memória, ciclos de CPU e gráficos baseados em buffer.

---

## Sobre o Projeto

Este projeto é um emulador funcional da arquitetura CHIP-8, um interpretador usado em sistemas embarcados da década de 70 para rodar jogos simples. Ele tem como objetivo principal ser **didático**, **legível** e servir como introdução à emulação e à linguagem Rust.


## Funcionalidades Implementadas

✅ Ciclo de CPU  
✅ Execução de instruções do conjunto CHIP-8  
✅ Renderização gráfica (em `minifb`)  
✅ Temporizadores (delay e sound)  
✅ Leitura do teclado físico (mapeamento para 0x0 - 0xF)  
✅ Carregamento de ROMs `.ch8`


## 🧠 Como Funciona

- A CPU busca instruções de 2 bytes da memória.
- O registrador `pc` é incrementado após cada instrução.
- A memória, os registradores e o framebuffer simulam o comportamento do CHIP-8 real.
- Um buffer de vídeo de 64x32 pixels é usado para desenhar na tela.
- O emulador interpreta os **opcodes** e os executa de acordo com sua semântica.
