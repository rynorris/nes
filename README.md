# mos-6500
NES emulator written in Rust

[![Build Status](https://travis-ci.org/DiscoViking/mos-6500.svg?branch=master)](https://travis-ci.org/DiscoViking/mos-6500) [![codecov](https://codecov.io/gh/DiscoViking/mos-6500/branch/master/graph/badge.svg)](https://codecov.io/gh/DiscoViking/mos-6500)

## Emulator Status

**CPU**
  - [x] Official Opcodes
  - [ ] Unofficial Opcodes

**PPU**
  - [x] Tiles
  - [x] Palettes
  - [X] Sprites
  
 **IO**
  - [x] Graphics output
  - [ ] Properly emulate NTSC video signal
  - [X] Controller input
  
 **Other**
  - [x] Basic iNES file loading
  - [ ] Support common mappers (~NROM~, ~MMC1~, MMC3)
  - [x] Clock to drive all components at the correct speed
  
  ## Examples
  
  ![Donkey Kong](https://user-images.githubusercontent.com/3620166/47778096-6b6d4e00-dd39-11e8-9dc1-2bd7946627ca.gif)
