# WebAssembly Gamebuino Emulator

This is an emulator for the [Gamebuino Meta](https://gamebuino.com/). It is a port of [gamebuino-meta](https://github.com/aoneill01/gamebuino-emulator) to Rust + WebAssembly.

## Example

[http://games.aoneill.com/wasm-proto/](http://games.aoneill.com/wasm-proto/)

## Usage

```html
<gamebuino-emulator src="https://raw.githubusercontent.com/aoneill01/meta-solitaire/master/binaries/Solitaire/SOLITAIRE.BIN"></gamebuino-emulator>

<script src="https://unpkg.com/@aoneill01/wasm-gamebuino" type="module"></script>
```

## Building

### Pre-requisisites

* [Rust](https://www.rust-lang.org/tools/install)
* [wasm-pack](https://rustwasm.github.io/wasm-pack/installer/)
* [npm](https://www.npmjs.com/get-npm)

```
npm install
npm run build
```
