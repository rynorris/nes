import { Emulator, Event, Key } from "nes_web";

window.nes_audio = new AudioContext();
var compressor = window.nes_audio.createDynamicsCompressor();
compressor.connect(window.nes_audio.destination);

const render = nes => {
    const screen = document.getElementById("screen");
    const tgt = document.createElement("canvas");
    tgt.width = 256;
    tgt.height = 240;

    const ctx = tgt.getContext("2d");
    const img = ctx.createImageData(256, 240);
    const frame = nes.get_frame();
    for (let i = 0; i < img.data.length / 4; i++) {
        // RGBA
        img.data[i * 4] = frame[i * 3];
        img.data[i * 4 + 1] = frame[i * 3 + 1];
        img.data[i * 4 + 2] = frame[i * 3 + 2];
        img.data[i * 4 + 3] = 255;
    }
    ctx.putImageData(img, 0, 0);

    screen.getContext("2d").drawImage(tgt, 0, 0, 512, 480);
};

var BUF_LENGTH = 0.1;
var nextStartTime = 0;

const queue_audio = (nes, cycles, samples) => {
    const buf = nes.get_audio(BigInt(cycles), BigInt(samples))
    if (buf.length === 0) {
        return;
    }

    const ctx = window.nes_audio;
    const audioBuffer = ctx.createBuffer(1, buf.length, 48000);
    audioBuffer.getChannelData(0).set(buf);
    const source = ctx.createBufferSource();
    source.buffer = audioBuffer;
    source.connect(ctx.destination);

    const bufLength = audioBuffer.length / audioBuffer.sampleRate;

    if (nextStartTime === 0) {
        // Initial buffer time.
        nextStartTime = ctx.currentTime + BUF_LENGTH;
    }

    source.start(nextStartTime);

    nextStartTime += bufLength;

    // Clamp audio latency.
    if (nextStartTime < ctx.currentTime || nextStartTime > ctx.currentTime + BUF_LENGTH) {
        // Reset buffer.
        nextStartTime = ctx.currentTime + BUF_LENGTH;
        console.log("Reset audio latency");
    }
}

var stopRunning = () => {};

const selectRom = files => {
    if (files.length !== 1) {
        throw new Error("Must select exactly 1 file");
    }

    stopRunning();

    const file = files[0];
    const reader = new FileReader();
    var nes = null;
    var running = true;

    reader.onloadend = () => {
        const array = reader.result;
        nes = Emulator.new(new Uint8Array(array));

        var ts = null;

        function render_cb(timestamp) {
            var cycles = nes.run(100000);
            render(nes);

            if (ts !== null) {
                const dt = timestamp - ts;
                queue_audio(nes, cycles, Math.round(48000 / 60));
            }

            if (running) {
                window.requestAnimationFrame(render_cb);
            }

            ts = timestamp;
            cycles = BigInt(0);
        }

        stopRunning = () => running = false;

        nes_audio.resume();
        window.requestAnimationFrame(render_cb);
    }

    reader.readAsArrayBuffer(file);

    document.onkeydown = event => {
        var key = convertJsKeyCode(event.key);
        if (key !== null) {
            nes.broadcast(Event.key_down(key));
        }
    };

    document.onkeyup = event => {
        var key = convertJsKeyCode(event.key);
        if (key !== null) {
            nes.broadcast(Event.key_up(key));
        }
    };
};

const convertJsKeyCode = key => {
    switch (key) {
        case "a":
            return Key.A;
        case "s":
            return Key.S;
        case "x":
            return Key.X;
        case "z":
            return Key.Z;
        case "ArrowLeft":
            return Key.Left;
        case "ArrowRight":
            return Key.Right;
        case "ArrowUp":
            return Key.Up;
        case "ArrowDown":
            return Key.Down;
        default:
            return null;
    }
};

window.selectRom = selectRom;
