import { Gamebuino } from "wasm-gamebuino";
import { memory } from "wasm-gamebuino/wasm_gamebuino_bg";

const gamebuino = Gamebuino.new();

let ctx = document.getElementById("gbscreen").getContext("2d");

let imageData = ctx.getImageData(0, 0, 160, 128);

let buttonData = 0b11111111;

const keymap = [
  [83, 40], // down
  [65, 81, 37], // left
  [68, 39], // right
  [87, 90, 38], // up
  [74], // A
  [75], // B
  [85], // MENU
  [73]  // HOME
];

document.addEventListener("keydown", event => {
  for (var i = 0; i < keymap.length; i++) {
    for (var code of keymap[i]) {
      if (code == event.keyCode) {
        event.preventDefault();
        buttonData &= (~(1 << i));
        return;
      }
    }
  }
});

document.addEventListener('keyup', event => {
  for (var i = 0; i < keymap.length; i++) {
    for (var code of keymap[i]) {
      if (code == event.keyCode) {
        buttonData |= (1 << i);
        return;
      }
    }
  }
});

fetch("https://raw.githubusercontent.com/Rodot/Games-META/master/binaries/METAtris/METAtris.bin")
// fetch("https://raw.githubusercontent.com/Rodot/Games-META/master/binaries/CatsAndCoinsDemo/CatsAndCoinsDemo.bin")
// fetch("https://raw.githubusercontent.com/Rodot/Games-META/master/binaries/Solitaire/Solitaire.bin")
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
  gamebuino.run(20000000/60, buttonData);

  let buf8 = new Uint8Array(memory.buffer, gamebuino.screen_data(), 160 * 128 * 4);
  imageData.data.set(buf8);
  ctx.putImageData(imageData, 0, 0);

  requestAnimationFrame(step);
}