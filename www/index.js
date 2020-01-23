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

const dpadX = 119;
const dpadY = 216;
const dpadDist = 87;

const aX = 649;
const aY = 233;

const bX = 726;
const bY = 199;

const menuX = 297;
const menuY = 382;

const homeX = 505;
const homeY = 382;

const btnDist = 40;

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

controls.addEventListener("touchstart", handleTouches);
controls.addEventListener("touchmove", handleTouches);
controls.addEventListener("touchend", handleTouches);

controls.addEventListener("mousedown", handleMouseDown);
controls.addEventListener("mousemove", handleMouseMove);
controls.addEventListener("mouseup", handleMouseUp);

function squareDist(touch, x, y) {
    return (
        (touch.pageX - x) * (touch.pageX - x) +
        (touch.pageY - y) * (touch.pageY - y)
    );
}

var mousePressed = false;
function handleMouseDown(event) {
    mousePressed = true;
    handleMouseMove(event);
}

function handleMouseMove(event) {
    if (mousePressed) buttonData = handleTouch(event);
}

function handleMouseUp() {
    mousePressed = false;
    buttonData = 0b11111111;
}

function handleTouches(event) {
    event.preventDefault();

    buttonData = 0b11111111;

    for (var touch of event.touches) {
        buttonData &= handleTouch(touch);
    }
}

function handleTouch(touch) {
    if (squareDist(touch, dpadX, dpadY) < dpadDist * dpadDist) {
        var angle = Math.atan2(dpadY - touch.pageY, touch.pageX - dpadX);

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
    } else if (squareDist(touch, aX, aY) < btnDist * btnDist) {
        return 0b11101111;
    } else if (squareDist(touch, bX, bY) < btnDist * btnDist) {
        return 0b11011111;
    } else if (squareDist(touch, menuX, menuY) < btnDist * btnDist) {
        return 0b10111111;
    } else if (squareDist(touch, homeX, homeY) < btnDist * btnDist) {
        return 0b01111111;
    }

    return 0b11111111;
}

function start(program) {
    gamebuino = Gamebuino.new();

    if (!program) {
        fetch(
            "https://raw.githubusercontent.com/Rodot/Games-META/master/binaries/METAtris/METAtris.bin"
        )
            .then(response => response.arrayBuffer())
            .then(buffer => {
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
