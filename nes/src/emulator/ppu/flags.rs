// PPUCTRL
// 7  bit  0
// ---- ----
// VPHB SINN
// |||| ||||
// |||| ||++- Base nametable address
// |||| ||    (0 = $2000; 1 = $2400; 2 = $2800; 3 = $2C00)
// |||| |+--- VRAM address increment per CPU read/write of PPUDATA
// |||| |     (0: add 1, going across; 1: add 32, going down)
// |||| +---- Sprite pattern table address for 8x8 sprites
// ||||       (0: $0000; 1: $1000; ignored in 8x16 mode)
// |||+------ Background pattern table address (0: $0000; 1: $1000)
// ||+------- Sprite size (0: 8x8 pixels; 1: 8x16 pixels)
// |+-------- PPU master/slave select
// |          (0: read backdrop from EXT pins; 1: output color on EXT pins)
// +--------- Generate an NMI at the start of the
//            vertical blanking interval (0: off; 1: on)
pub enum PPUCTRL {
    V = 1 << 7,
    P = 1 << 6,
    H = 1 << 5,
    B = 1 << 4,
    S = 1 << 3,
    I = 1 << 2,
}

impl Into<u8> for PPUCTRL {
    fn into(self) -> u8 {
        self as u8
    }
}

// PPUMASK
// 7  bit  0
// ---- ----
// BGRs bMmG
// |||| ||||
// |||| |||+- Greyscale (0: normal color, 1: produce a greyscale display) (GR)
// |||| ||+-- 1: Show background in leftmost 8 pixels of screen, 0: Hide (BGL)
// |||| |+--- 1: Show sprites in leftmost 8 pixels of screen, 0: Hide (SL)
// |||| +---- 1: Show background (BG)
// |||+------ 1: Show sprites (S)
// ||+------- Emphasize red*
// |+-------- Emphasize green*
// +--------- Emphasize blue
pub enum PPUMASK {
    B = 1 << 7,
    G = 1 << 6,
    R = 1 << 5,
    S = 1 << 4,
    BG = 1 << 3,
    SL = 1 << 2,
    BGL = 1 << 1,
    GR = 1,
}

impl Into<u8> for PPUMASK {
    fn into(self) -> u8 {
        self as u8
    }
}

// PPUSTATUS
// 7  bit  0
// ---- ----
// VSO. ....
// |||| ||||
// |||+-++++- Least significant bits previously written into a PPU register
// |||        (due to register not being updated for this address)
// ||+------- Sprite overflow. The intent was for this flag to be set
// ||         whenever more than eight sprites appear on a scanline, but a
// ||         hardware bug causes the actual behavior to be more complicated
// ||         and generate false positives as well as false negatives; see
// ||         PPU sprite evaluation. This flag is set during sprite
// ||         evaluation and cleared at dot 1 (the second dot) of the
// ||         pre-render line.
// |+-------- Sprite 0 Hit.  Set when a nonzero pixel of sprite 0 overlaps
// |          a nonzero background pixel; cleared at dot 1 of the pre-render
// |          line.  Used for raster timing.
// +--------- Vertical blank has started (0: not in vblank; 1: in vblank).
//            Set at dot 1 of line 241 (the line *after* the post-render
//            line); cleared after reading $2002 and at dot 1 of the
//            pre-render line.
pub enum PPUSTATUS {
    V = 1 << 7,
    S = 1 << 6,
    O = 1 << 5,
}

impl Into<u8> for PPUSTATUS {
    fn into(self) -> u8 {
        self as u8
    }
}
