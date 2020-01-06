import { Gamebuino } from "wasm-gamebuino";

const gamebuino = Gamebuino.new();

fetch("https://raw.githubusercontent.com/Rodot/Games-META/master/binaries/Solitaire/Solitaire.bin")
  .then((response) => response.arrayBuffer())
  .then((buffer) => {
    gamebuino.load_program(new Uint8Array(buffer), 0x4000);
    for (let i = 0; i < 20; i++) gamebuino.step();
    
    document.getElementById("next").addEventListener("click", ev => {
        ev.preventDefault();
        gamebuino.step();
    });

    const start = window.performance.now();
    gamebuino.run(48000000);
    const end = window.performance.now();
    console.log(end - start);
  });