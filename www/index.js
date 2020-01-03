import { Gamebuino } from "wasm-gamebuino";

const gamebuino = Gamebuino.new();
gamebuino.dummy();

const start = window.performance.now();
gamebuino.run(48000000);
const end = window.performance.now();
console.log(end - start);