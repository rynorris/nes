const toArrayBuffer = array => array.buffer.slice(array.byteOffset, array.byteLength + array.byteOffset)

const CYCLES_PER_SECOND = 21477272;
const FRAMES_PER_SECOND = (1000 / 16);
const CYCLES_PER_FRAME = 1.5 * CYCLES_PER_SECOND / FRAMES_PER_SECOND;

import("nes_web")
    .then(({ Emulator, Event, Key }) => {
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

        var stopRunning = () => {};

        var nes = null;
        const loadRom = buf => {
            stopRunning();

            var running = true;

            nes = Emulator.new(new Uint8Array(buf));

            function step() {
                var cycles = BigInt(0);
                while (cycles <= CYCLES_PER_FRAME) {
                    cycles += nes.run(100);
                }

                const audio = nes.get_audio(cycles, BigInt(Math.round(48000 / (1000 / 16))));
                const video = nes.get_frame();

                const audio_arr = toArrayBuffer(audio);
                const video_arr = toArrayBuffer(video);

                postMessage({ audio: audio_arr, video: video_arr }, [audio_arr, video_arr]);
            }

            const interval = setInterval(step, 16);

            stopRunning = () => clearInterval(interval);
        };

        onmessage = ({ data }) => {
            var key;
            switch (data.kind) {
                case "rom":
                    loadRom(data.data);
                    break;
                case "keydown":
                    key = convertJsKeyCode(data.key);
                    if (key !== null) {
                        nes.broadcast(Event.key_down(key));
                    }
                    break;
                case "keyup":
                    key = convertJsKeyCode(data.key);
                    if (key !== null) {
                        nes.broadcast(Event.key_up(key));
                    }
                    break;
            }
        };
    });

