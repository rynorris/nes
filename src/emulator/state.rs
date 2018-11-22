// This file contains the save states API.
// Changes could break old save states.

extern crate serde;

use self::serde::{Serialize, Deserialize};

pub trait SaveState<'de, T: Serialize + Deserialize<'de>> {
    fn freeze(&mut self) -> T;
    fn hydrate(&mut self, t: T);
}

#[derive(Serialize, Deserialize)]
pub struct CPUState {
    pub a: u8,
    pub x: u8,
    pub y: u8,
    pub sp: u8,
    pub pc: u16,
    pub p: u8,
    pub dec_arith_on: bool,
    pub irq_flip_flop: bool,
    pub nmi_flip_flop: bool,
}
