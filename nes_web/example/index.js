const nes_audio = new AudioContext();
const worker = new Worker("worker.js");

const render = frame => {
    const screen = document.getElementById("screen");
    const tgt = document.createElement("canvas");
    tgt.width = 256;
    tgt.height = 240;

    const ctx = tgt.getContext("2d");
    const img = ctx.createImageData(256, 240);
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

const queue_audio = buf => {
    if (buf.length === 0) {
        return;
    }

    const ctx = nes_audio;
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
        
        worker.postMessage({ kind: "rom", data: array }, [array]);
    }

    reader.readAsArrayBuffer(file);
};

document.onkeydown = event => {
    worker.postMessage({ kind: "keydown", key: event.key });
};

document.onkeyup = event => {
    worker.postMessage({ kind: "keyup", key: event.key });
};

worker.onmessage = ({ data }) => {
    queue_audio(new Float32Array(data.audio));
    render(new Uint8Array(data.video));
};

window.selectRom = selectRom;
