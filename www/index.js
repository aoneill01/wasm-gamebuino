import { Gamebuino } from "wasm-gamebuino";
import { memory } from "wasm-gamebuino/wasm_gamebuino_bg";

const gamebuino = Gamebuino.new();

let ctx = document.getElementById("gbscreen").getContext("2d");

let imageData = ctx.getImageData(0, 0, 160, 128);

fetch("https://raw.githubusercontent.com/Rodot/Games-META/master/binaries/Solitaire/Solitaire.bin")
  .then((response) => response.arrayBuffer())
  .then((buffer) => {
    gamebuino.load_program(new Uint8Array(buffer), 0x4000);

    step();

    // const start = window.performance.now();
    // gamebuino.run(48000000);
    // const end = window.performance.now();
    // console.log(end - start);
  });

function step() {
  gamebuino.run(20000000/60);

  let buf8 = new Uint8Array(memory.buffer, gamebuino.screen_data(), 160 * 128 * 4);
  imageData.data.set(buf8);
  ctx.putImageData(imageData, 0, 0);

  requestAnimationFrame(step);
}