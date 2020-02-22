#![allow(dead_code)]
pub mod apu;
pub mod clock;
pub mod components;
pub mod controller;
pub mod cpu;
pub mod ines;
pub mod io;
pub mod mappers;
pub mod memory;
pub mod ppu;
pub mod state;
pub mod util;

#[cfg(test)]
mod test;

use std::cell::RefCell;
use std::rc::Rc;

use crate::emulator::apu::AudioOut;
use crate::emulator::controller::Button;
use crate::emulator::io::event::{EventBus, Key};
use crate::emulator::io::Screen;
use crate::emulator::memory::{IORegisters, Writer};
use crate::emulator::state::{NESState, SaveState};

// Timings (NTSC).
// Master clock = 21.477272 MHz ~= 46.5ns per clock.
// CPU clock = 12 master clocks.
// PPU clock = 4 master clocks.
pub const NES_MASTER_CLOCK_HZ: u64 = 21_477_272;
pub const NES_CPU_CLOCK_FACTOR: u32 = 12;
pub const NES_APU_CLOCK_FACTOR: u32 = 24;
pub const NES_PPU_CLOCK_FACTOR: u32 = 4;

pub struct NES {
    clock: clock::Clock,
    pub cpu: Rc<RefCell<cpu::CPU>>,
    pub ppu: Rc<RefCell<ppu::PPU>>,
    pub apu: Rc<RefCell<apu::APU>>,
    pub mapper: Rc<RefCell<dyn memory::Mapper>>,
    pub ram: Rc<RefCell<memory::Memory>>,
    pub sram: Rc<RefCell<memory::Memory>>,
    pub vram: Rc<RefCell<memory::Memory>>,
    pub screen: Rc<RefCell<Screen>>,
    pub joy1: Rc<RefCell<controller::Controller>>,
    pub joy2: Rc<RefCell<controller::Controller>>,
    nmi_pin: bool,
}

impl NES {
    pub fn new<A>(
        event_bus: Rc<RefCell<EventBus>>,
        screen: Rc<RefCell<Screen>>,
        audio: A,
        rom: ines::ROM,
    ) -> NES
    where
        A: AudioOut + 'static,
    {
        // Create master clock.
        let mut clock = clock::Clock::new();

        // Load ROM into memory.
        let mapper = rom.get_mapper();

        // Create RAM modules.
        let ram = Rc::new(RefCell::new(memory::Memory::new_ram(0x800)));
        let sram = Rc::new(RefCell::new(memory::Memory::new_ram(0x2000)));
        let vram = Rc::new(RefCell::new(memory::Memory::new_ram(0x2000)));

        // Create graphics output module and PPU.
        let ppu_memory = memory::PPUMemory::new(
            Box::new(memory::ChrMapper::new(mapper.clone())),
            Box::new(mapper.clone()),
            Box::new(vram.clone()),
        );

        let ppu = Rc::new(RefCell::new(ppu::PPU::new(
            ppu_memory,
            Box::new(screen.clone()),
        )));

        // Create APU.
        let apu = Rc::new(RefCell::new(apu::APU::new(
            Box::new(audio),
            Box::new(memory::PrgMapper::new(mapper.clone())),
        )));

        // Create controllers.
        let joy1 = Rc::new(RefCell::new(controller::Controller::new(
            [
                (Key::Z, Button::A),
                (Key::X, Button::B),
                (Key::A, Button::Start),
                (Key::S, Button::Select),
                (Key::Up, Button::Up),
                (Key::Down, Button::Down),
                (Key::Left, Button::Left),
                (Key::Right, Button::Right),
            ]
            .iter()
            .cloned()
            .collect(),
        )));

        let joy2 = Rc::new(RefCell::new(controller::Controller::new(
            [].iter().cloned().collect(),
        )));

        event_bus.borrow_mut().register(Box::new(joy1.clone()));
        event_bus.borrow_mut().register(Box::new(joy2.clone()));

        // Create CPU.
        let io_registers = Rc::new(RefCell::new(memory::IORegisters::new(
            Box::new(apu.clone()),
            Box::new(joy1.clone()),
            Box::new(joy2.clone()),
        )));

        let cpu_memory = memory::CPUMemory::new(
            Box::new(ram.clone()),
            Box::new(ppu.clone()),
            Box::new(io_registers.clone()),
            Box::new(sram.clone()),
            Box::new(memory::PrgMapper::new(mapper.clone())),
        );

        let cpu = Rc::new(RefCell::new(cpu::new(Box::new(cpu_memory))));
        cpu.borrow_mut().disable_bcd();
        cpu.borrow_mut().startup_sequence();

        let dma_controller = DMAController::new(io_registers.clone(), cpu.clone());

        // Wire up the clock timings.
        let cpu_ticker = clock::ScaledTicker::new(Box::new(dma_controller), NES_CPU_CLOCK_FACTOR);
        let ppu_ticker = clock::ScaledTicker::new(Box::new(ppu.clone()), NES_PPU_CLOCK_FACTOR);
        let apu_ticker = clock::ScaledTicker::new(Box::new(apu.clone()), NES_APU_CLOCK_FACTOR);
        clock.manage(cpu_ticker);
        clock.manage(apu_ticker);
        clock.manage(ppu_ticker);

        NES {
            clock,
            cpu,
            ppu,
            apu,
            mapper,
            ram,
            sram,
            vram,
            screen,
            joy1,
            joy2,
            nmi_pin: false,
        }
    }

    #[inline]
    pub fn tick(&mut self) -> u64 {
        let cycles = self.clock.tick();
        if self.ppu.borrow().nmi_triggered() {
            if self.nmi_pin == false {
                self.cpu.borrow_mut().trigger_nmi();
                self.nmi_pin = true;
            }
        } else {
            self.nmi_pin = false;
        }

        if self.apu.borrow().irq_triggered() {
            self.cpu.borrow_mut().trigger_irq();
        }

        if self.mapper.borrow().irq_triggered() {
            self.cpu.borrow_mut().trigger_irq();
        }

        cycles
    }

    pub fn tick_multi(&mut self, ticks: u32) -> u64 {
        let mut cycles = 0u64;
        for _ in 0..ticks {
            cycles += self.tick()
        }
        cycles
    }

    pub fn reset(&mut self) {
        // Silence APU.
        self.apu.borrow_mut().write(0x4015, 0x00);

        // Restart CPU.
        self.cpu.borrow_mut().startup_sequence();
    }
}

pub struct DMAController {
    copies_remaining: u16,
    base_address: u16,
    io_registers: Rc<RefCell<IORegisters>>,
    cpu: Rc<RefCell<cpu::CPU>>,
}

impl DMAController {
    pub fn new(
        io_registers: Rc<RefCell<IORegisters>>,
        cpu: Rc<RefCell<cpu::CPU>>,
    ) -> DMAController {
        DMAController {
            copies_remaining: 0,
            base_address: 0,
            io_registers,
            cpu,
        }
    }
}

impl clock::Ticker for DMAController {
    fn tick(&mut self) -> u32 {
        match self.io_registers.borrow_mut().get_oamdma() {
            None => (),
            Some(byte) => {
                // DMA triggered.
                self.base_address = (byte as u16) << 8;
                self.copies_remaining = 256;
            }
        }

        if self.copies_remaining > 0 {
            // CPU is suspended during copy.
            let byte = self
                .cpu
                .borrow_mut()
                .load_memory(self.base_address.wrapping_add(256 - self.copies_remaining));
            self.cpu.borrow_mut().store_memory(0x2004, byte);
            self.copies_remaining -= 1;
            2
        } else {
            self.cpu.borrow_mut().tick()
        }
    }
}

impl<'de> SaveState<'de, NESState> for NES {
    fn freeze(&mut self) -> NESState {
        NESState {
            cpu: self.cpu.borrow_mut().freeze(),
            ppu: self.ppu.borrow_mut().freeze(),
            mapper: self.mapper.borrow_mut().freeze(),
            ram: self.ram.borrow_mut().freeze(),
            sram: self.sram.borrow_mut().freeze(),
            vram: self.vram.borrow_mut().freeze(),
            screen: self.screen.borrow_mut().freeze(),
            joy1: self.joy1.borrow_mut().freeze(),
            joy2: self.joy2.borrow_mut().freeze(),
        }
    }

    fn hydrate(&mut self, state: NESState) {
        self.cpu.borrow_mut().hydrate(state.cpu);
        self.ppu.borrow_mut().hydrate(state.ppu);
        self.mapper.borrow_mut().hydrate(state.mapper);
        self.ram.borrow_mut().hydrate(state.ram);
        self.sram.borrow_mut().hydrate(state.sram);
        self.vram.borrow_mut().hydrate(state.vram);
        self.screen.borrow_mut().hydrate(state.screen);
        self.joy1.borrow_mut().hydrate(state.joy1);
        self.joy2.borrow_mut().hydrate(state.joy2);
    }
}
