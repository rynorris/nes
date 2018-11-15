// In iNES mapper number order.

// #0 NROM
mod nrom;
pub use self::nrom::NROM;

// #1 MMC1
mod mmc1;
pub use self::mmc1::MMC1;

// #2 UxROM
mod uxrom;
pub use self::uxrom::UXROM;

// #3 CNROM
mod cnrom;
pub use self::cnrom::CNROM;

// #4 MMC3
mod mmc3;
pub use self::mmc3::MMC3;

// #7 AxROM
mod axrom;
pub use self::axrom::AXROM;
