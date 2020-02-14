import { Gamebuino } from "wasm-gamebuino";
import { memory } from "wasm-gamebuino/wasm_gamebuino_bg";

const keymap = [
    [83, 40], // down
    [65, 81, 37], // left
    [68, 39], // right
    [87, 90, 38], // up
    [74], // A
    [75], // B
    [85], // MENU
    [73] // HOME
];

const dpadX = 109;
const dpadY = 206;
const dpadDist = 87;

const aX = 639;
const aY = 223;

const bX = 716;
const bY = 189;

const menuX = 287;
const menuY = 372;

const homeX = 495;
const homeY = 372;

const btnDist = 40;

class GamebuinoEmulator extends HTMLElement {
    constructor() {
        super();

        this.root = this.attachShadow({ mode: "open" });

        this.root.innerHTML = `
        <style>
            :host {
                display: inline-block;
                width: 788px;
                height: 428px;
            }

            #console {
                width: 788px;
                height: 428px;
                background-image: url(console.png);
                position: relative;
            }

            #gbscreen {
                position: absolute;
                top: 80px;
                left: 232px;
            }
        </style>
        <div id="console">
            <canvas id="gbscreen" width="320" height="256"></canvas>
        </div>
        `;

        this.canvas = this.root.getElementById("gbscreen");
        this.ctx = this.canvas.getContext("2d");
        this.ctx.scale(2, 2);
        this.ctx.imageSmoothingEnabled = false;

        this.imageData = this.ctx.getImageData(0, 0, 160, 128);

        this.buttonData = 0b11111111;
        this.lastTimestamp = 0;
        this.requestId;

        this.pointerPresses = {};

        document.addEventListener("keydown", event => {
            for (var i = 0; i < keymap.length; i++) {
                for (var code of keymap[i]) {
                    if (code == event.keyCode) {
                        event.preventDefault();
                        this.buttonData &= ~(1 << i);
                        return;
                    }
                }
            }
        });

        document.addEventListener("keyup", event => {
            for (var i = 0; i < keymap.length; i++) {
                for (var code of keymap[i]) {
                    if (code == event.keyCode) {
                        this.buttonData |= 1 << i;
                        return;
                    }
                }
            }
        });

        const controls = this.root.getElementById("console");

        controls.addEventListener("pointerdown", event =>
            this.handlePointerDown(event)
        );
        controls.addEventListener("pointermove", event =>
            this.handlePointerMove(event)
        );
        document.addEventListener("pointerup", event =>
            this.handlePointerUp(event)
        );
        document.addEventListener("pointercancel", event =>
            this.handlePointerUp(event)
        );

        this.start();
    }

    get src() {
        return this.getAttribute("src");
    }

    set src(value) {
        return this.setAttribute("src", value);
    }

    static get observedAttributes() {
        return ["src"];
    }

    attributeChangedCallback(name, oldValue, newValue) {
        this.load();
    }

    load(program) {
        this.start(program);
    }

    start(program) {
        let arrayBufferPromise;

        if (program) {
            arrayBufferPromise = Promise.resolve(program);
        } else {
            arrayBufferPromise = fetch(this.src)
                .then(response => response.arrayBuffer());
        }

        arrayBufferPromise.then(buffer => {
            if (this.requestId) cancelAnimationFrame(this.requestId);
            this.gamebuino = Gamebuino.new();
            this.gamebuino.load_program(new Uint8Array(buffer), 0x4000);
            this.step();
        });
    }

    step(timestamp) {
        const goalTicksPerSecond = 20000000;
        const maxIterations = goalTicksPerSecond / 30;
        const delta = timestamp - this.lastTimestamp;
        this.lastTimestamp = timestamp;
        let iterations = (delta * goalTicksPerSecond) / 1000;
        if (iterations > maxIterations) iterations = maxIterations;

        this.gamebuino.run(iterations, this.buttonData);

        let buf8 = new Uint8Array(
            memory.buffer,
            this.gamebuino.screen_data(),
            160 * 128 * 4
        );
        this.imageData.data.set(buf8);
        this.ctx.putImageData(this.imageData, 0, 0);
        this.ctx.drawImage(this.canvas, 0, 0);

        this.requestId = requestAnimationFrame(t => this.step(t));
    }

    squareDist(touch, x, y) {
        return (
            (touch.offsetX - x) * (touch.offsetX - x) +
            (touch.offsetY - y) * (touch.offsetY - y)
        );
    }

    handlePointerDown(event) {
        event.preventDefault();
        this.pointerPresses[event.pointerId] = 0b11111111;
        this.handlePointerMove(event);
        this.updateButtonData();
    }

    handlePointerMove(event) {
        if (this.pointerPresses.hasOwnProperty(event.pointerId)) {
            this.pointerPresses[event.pointerId] = this.handlePointer(event);
        }
        this.updateButtonData();
    }

    handlePointerUp(event) {
        delete this.pointerPresses[event.pointerId];
        this.updateButtonData();
    }

    updateButtonData() {
        this.buttonData = 0b11111111;
        for (let prop in this.pointerPresses) {
            this.buttonData &= this.pointerPresses[prop];
        }
    }

    handlePointer(event) {
        if (this.squareDist(event, dpadX, dpadY) < dpadDist * dpadDist) {
            var angle = Math.atan2(
                dpadY - event.offsetY,
                event.offsetX - dpadX
            );

            if (angle < (-7 * Math.PI) / 8) {
                return 0b11111101;
            } else if (angle < (-5 * Math.PI) / 8) {
                return 0b11111100;
            } else if (angle < (-3 * Math.PI) / 8) {
                return 0b11111110;
            } else if (angle < -Math.PI / 8) {
                return 0b11111010;
            } else if (angle < Math.PI / 8) {
                return 0b11111011;
            } else if (angle < (3 * Math.PI) / 8) {
                return 0b11110011;
            } else if (angle < (5 * Math.PI) / 8) {
                return 0b11110111;
            } else if (angle < (7 * Math.PI) / 8) {
                return 0b11110101;
            } else {
                return 0b11111101;
            }
        } else if (this.squareDist(event, aX, aY) < btnDist * btnDist) {
            return 0b11101111;
        } else if (this.squareDist(event, bX, bY) < btnDist * btnDist) {
            return 0b11011111;
        } else if (this.squareDist(event, menuX, menuY) < btnDist * btnDist) {
            return 0b10111111;
        } else if (this.squareDist(event, homeX, homeY) < btnDist * btnDist) {
            return 0b01111111;
        }

        return 0b11111111;
    }
}

customElements.define("gamebuino-emulator", GamebuinoEmulator);
