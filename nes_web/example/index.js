const screen = document.getElementById("screen");

const init = () => {
    const AudioContext = window["AudioContext"] || window["webkitAudioContext"];
    const nes_audio = new AudioContext();
    const worker = new Worker("worker.js");

    const BUF_LENGTH = 0.1;
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

    const render = frame => {
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

    const workerPromise = new Promise((resolve, reject) => {
        worker.onmessage = () => {
            worker.onmessage = ({ data }) => {
                queue_audio(new Float32Array(data.audio));
                render(new Uint8Array(data.video));
            };
            resolve(worker);
        };
    });

    return workerPromise;
}

var workerPromise = null;

const selectRom = files => {
    if (files.length !== 1) {
        throw new Error("Must select exactly 1 file");
    }

    if (workerPromise === null) {
        workerPromise = init();
    }

    const file = files[0];
    const reader = new FileReader();

    const romPromise = new Promise((resolve, reject) => {
        reader.onloadend = () => {
            const array = reader.result;

            resolve(array);
        }
    });

    Promise.all([workerPromise, romPromise])
        .then(([worker,  rom]) => {
            worker.postMessage({ kind: "rom", data: rom }, [rom]);

            screen.onkeydown = event => {
                event.preventDefault();
                worker.postMessage({ kind: "keydown", key: event.key });
            };

            screen.onkeyup = event => {
                event.preventDefault();
                worker.postMessage({ kind: "keyup", key: event.key });
            };

            screen.focus();
        });


    reader.readAsArrayBuffer(file);
};


window.selectRom = selectRom;
