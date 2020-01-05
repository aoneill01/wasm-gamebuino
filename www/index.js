import { Gamebuino } from "wasm-gamebuino";

const gamebuino = Gamebuino.new();
gamebuino.dummy();
for (let i = 0; i < 1000; i++) gamebuino.debug_instruction(0b0001110001000000); // add 1 to r0
gamebuino.debug_instruction(0b0010000100000001); // move 1 to r1
gamebuino.debug_instruction(0b0100001001001001); // negate r1
gamebuino.dummy();


const start = window.performance.now();
gamebuino.run(75000000);
// gamebuino.run(48000000);
const end = window.performance.now();
console.log(end - start);
gamebuino.dummy();