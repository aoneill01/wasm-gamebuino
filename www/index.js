import { Gamebuino } from "wasm-gamebuino";
import { memory } from "wasm-gamebuino/wasm_gamebuino_bg";

const gamebuino = Gamebuino.new();

const canvas = document.getElementById("gbscreen");
let ctx = canvas.getContext("2d");
ctx.scale(2, 2);
ctx.imageSmoothingEnabled = false;

let imageData = ctx.getImageData(0, 0, 160, 128);

let buttonData = 0b11111111;

start();
// diff();
// runTo(8594726);

function step() {
  gamebuino.run(20000000/60, buttonData);

  let buf8 = new Uint8Array(memory.buffer, gamebuino.screen_data(), 160 * 128 * 4);
  imageData.data.set(buf8);
  ctx.putImageData(imageData, 0, 0);
  ctx.drawImage(canvas, 0, 0);

  requestAnimationFrame(step);
}

function start() {
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

  // fetch("https://raw.githubusercontent.com/Rodot/Games-META/master/binaries/PongMETA/PongMETA.bin")
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
}

// function diff() {
//   const gameUrl = "https://raw.githubusercontent.com/Rodot/Games-META/master/binaries/PongMETA/PongMETA.bin";
  
//   fetch(gameUrl)
//     .then((response) => response.arrayBuffer())
//     .then((buffer) => {
//       gamebuino.load_program(new Uint8Array(buffer), 0x4000);
//     });
  
//   const original = new metaEmulator.Emulator("original");
//   original.autoStart = false;
//   original.loadFromUrl(gameUrl);

//   setTimeout(() => {
//     let i = 0;
//     work();
//     function work() {
//       console.log("work");
//       let buf8 = new Uint8Array(memory.buffer, gamebuino.screen_data(), 160 * 128 * 4);
//       imageData.data.set(buf8);
//       ctx.putImageData(imageData, 0, 0);
//       ctx.drawImage(canvas, 0, 0);

//       original._screen.updateCanvas();

//       while (doesStepMatch()) {
//         i++;
//         if (i === 150000000) {
//           break;
//         }
//         if (i % 333333 === 0) {
//           setTimeout(work, 1);
//           return;
//         }
//       }
//       console.log(i);
//     }

//     function doesStepMatch() {
//       gamebuino.step();
//       original._atsamd21.step();
    
//       if (i >= 7137867 && i < 8000000) return true;
  
//       for (let i = 0; i < 16; i++) {
//         let a = gamebuino.get_register(i);
//         let b = original._atsamd21.registers[i];
//         if ((a >> 0) !== (b >> 0)) {
//           console.log(a, b);
//           debugger;
//           return false;
//         }
//       }
    
//       return true;
//     }
//   }, 1000);
// }

// function runTo(count) {
//   const gameUrl = "https://raw.githubusercontent.com/Rodot/Games-META/master/binaries/PongMETA/PongMETA.bin";
  
//   fetch(gameUrl)
//     .then((response) => response.arrayBuffer())
//     .then((buffer) => {
//       gamebuino.load_program(new Uint8Array(buffer), 0x4000);

//       for (let i = 0; i < count; i++) gamebuino.step();
//       gamebuino.enable_logging();
//       gamebuino.step();
//       gamebuino.step();

//       let buf8 = new Uint8Array(memory.buffer, gamebuino.screen_data(), 160 * 128 * 4);
//       imageData.data.set(buf8);
//       ctx.putImageData(imageData, 0, 0);
//       ctx.drawImage(canvas, 0, 0);
//     });
// }