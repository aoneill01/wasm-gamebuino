import { Gamebuino } from "wasm-gamebuino";
import { memory } from "wasm-gamebuino/wasm_gamebuino_bg";

const canvas = document.getElementById("gbscreen");
let ctx = canvas.getContext("2d");
ctx.scale(2, 2);
ctx.imageSmoothingEnabled = false;

let imageData = ctx.getImageData(0, 0, 160, 128);

let gamebuino;
let buttonData = 0b11111111;
let lastTimestamp = 0;
let requestId;

start();

document.getElementById("file-upload").onchange = function() {
  if (this.files.length == 1) {
      var f = this.files[0];
      var reader = new FileReader();
      reader.onload = function (e) {
          if (requestId) cancelAnimationFrame(requestId);
          start(e.target.result);
      };
      reader.readAsArrayBuffer(f);
      this.value = "";
  }
};

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

function start(program) {
  gamebuino = Gamebuino.new();

  if (!program) {
  fetch("https://raw.githubusercontent.com/Rodot/Games-META/master/binaries/METAtris/METAtris.bin")
    .then((response) => response.arrayBuffer())
    .then((buffer) => {
      gamebuino.load_program(new Uint8Array(buffer), 0x4000);
      step();
    });
  } else {
    gamebuino.load_program(new Uint8Array(program), 0x4000);
    step();
  }
}

function step(timestamp) {
  const goalTicksPerSecond = 20000000;
  const maxIterations = goalTicksPerSecond / 30;
  let delta = timestamp - lastTimestamp;
  lastTimestamp = timestamp;
  let iterations = delta * goalTicksPerSecond / 1000;
  if (iterations > maxIterations) iterations = maxIterations;

  gamebuino.run(iterations, buttonData);

  let buf8 = new Uint8Array(memory.buffer, gamebuino.screen_data(), 160 * 128 * 4);
  imageData.data.set(buf8);
  ctx.putImageData(imageData, 0, 0);
  ctx.drawImage(canvas, 0, 0);

  requestId = requestAnimationFrame(step);
}