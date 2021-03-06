# nes
NES emulator written in Rust

[![CircleCI](https://circleci.com/gh/rynorris/nes.svg?style=svg)](https://circleci.com/gh/rynorris/nes) [![codecov](https://codecov.io/gh/rynorris/nes/branch/master/graph/badge.svg)](https://codecov.io/gh/rynorris/nes)

## Emulator Status

**CPU**
  - [x] Official Opcodes
  - [ ] Unofficial Opcodes

**PPU**
  - [x] Tiles
  - [x] Palettes
  - [X] Sprites
  
**APU**
  - [x] Synthesizer
  - [x] High quality downsampling
  
**IO**
  - [x] Graphics output
  - [ ] Properly emulate NTSC video signal
  - [X] Controller input
  
**Debug Tools**
  - [x] CPU instruction tracing
  - [x] Granular speed controls.
  - [x] PPU debug window
  - [x] APU debug window
  - [ ] Proper debugger capabilities (step/trap/breakpoints)
  
**Other**
  - [x] Basic iNES file loading
  - [x] Support common mappers (~NROM~, ~MMC1~, ~MMC3~)
  - [x] Clock to drive all components at the correct speed
  
  ## Examples
  
  ![Megaman 2](https://user-images.githubusercontent.com/3620166/48202700-f806b480-e3a8-11e8-84a5-42c877cc6767.gif)
