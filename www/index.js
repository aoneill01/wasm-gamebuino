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
        reader.onload = function(e) {
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

const pointerPresses = {};

document.addEventListener("keydown", event => {
    for (var i = 0; i < keymap.length; i++) {
        for (var code of keymap[i]) {
            if (code == event.keyCode) {
                event.preventDefault();
                buttonData &= ~(1 << i);
                return;
            }
        }
    }
});

document.addEventListener("keyup", event => {
    for (var i = 0; i < keymap.length; i++) {
        for (var code of keymap[i]) {
            if (code == event.keyCode) {
                buttonData |= 1 << i;
                return;
            }
        }
    }
});

const controls = document.getElementById("console");

controls.addEventListener("pointerdown", handlePointerDown);
controls.addEventListener("pointermove", handlePointerMove);
document.addEventListener("pointerup", handlePointerUp);
document.addEventListener("pointercancel", handlePointerUp);

function squareDist(touch, x, y) {
    return (
        (touch.offsetX - x) * (touch.offsetX - x) +
        (touch.offsetY - y) * (touch.offsetY - y)
    );
}

function handlePointerDown(event) {
    event.preventDefault();
    pointerPresses[event.pointerId] = 0b11111111;
    handlePointerMove(event);
    updateButtonData();
}

function handlePointerMove(event) {
    if (pointerPresses.hasOwnProperty(event.pointerId)) {
        pointerPresses[event.pointerId] = handlePointer(event);
    }
    updateButtonData();
}

function handlePointerUp(event) {
    delete pointerPresses[event.pointerId];
    updateButtonData();
}

function updateButtonData() {
    buttonData = 0b11111111;
    for (let prop in pointerPresses) {
        buttonData = buttonData & pointerPresses[prop];
    }
}

function handlePointer(event) {
    if (squareDist(event, dpadX, dpadY) < dpadDist * dpadDist) {
        var angle = Math.atan2(dpadY - event.offsetY, event.offsetX - dpadX);

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
    } else if (squareDist(event, aX, aY) < btnDist * btnDist) {
        return 0b11101111;
    } else if (squareDist(event, bX, bY) < btnDist * btnDist) {
        return 0b11011111;
    } else if (squareDist(event, menuX, menuY) < btnDist * btnDist) {
        return 0b10111111;
    } else if (squareDist(event, homeX, homeY) < btnDist * btnDist) {
        return 0b01111111;
    }

    return 0b11111111;
}

function start(program) {
    gamebuino = Gamebuino.new();

    let arrayBufferPromise;

    if (program) {
        arrayBufferPromise = Promise.resolve(program);
    } else {
        arrayBufferPromise = fetch(
            "https://raw.githubusercontent.com/Rodot/Games-META/master/binaries/METAtris/METAtris.bin"
        ).then(response => response.arrayBuffer());
    }

    arrayBufferPromise.then(buffer => {
        gamebuino.load_program(new Uint8Array(buffer), 0x4000);
        step();
    });
}

function step(timestamp) {
    const goalTicksPerSecond = 20000000;
    const maxIterations = goalTicksPerSecond / 30;
    let delta = timestamp - lastTimestamp;
    lastTimestamp = timestamp;
    let iterations = (delta * goalTicksPerSecond) / 1000;
    if (iterations > maxIterations) iterations = maxIterations;

    gamebuino.run(iterations, buttonData);

    let buf8 = new Uint8Array(
        memory.buffer,
        gamebuino.screen_data(),
        160 * 128 * 4
    );
    imageData.data.set(buf8);
    ctx.putImageData(imageData, 0, 0);
    ctx.drawImage(canvas, 0, 0);

    requestId = requestAnimationFrame(step);
}
