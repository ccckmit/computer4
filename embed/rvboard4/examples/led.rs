//! LED Blinking Example for RVBoard4
//!
//! This example demonstrates how to blink an LED on the RVBoard4 development board.
//!
//! ## Architecture
//! - `LedHal` trait: Hardware Abstraction Layer for LED operations
//! - `led_blink()` function: Generic blinking logic using any HAL implementation
//!
//! ## For Embedded (no_std)
//! Implement `LedHal` for your GPIO hardware to control the LED.
//!
//! ## For Simulation
//! See `simulator/` directory - implements `LedHal` with SDL2 graphics.

pub trait LedHal {
    fn led_on(&mut self);
    fn led_off(&mut self);
    fn delay_ms(&mut self, ms: u32);
}

pub fn led_blink<H: LedHal>(hal: &mut H, times: u32) {
    for _ in 0..times {
        hal.led_on();
        hal.delay_ms(500);
        hal.led_off();
        hal.delay_ms(500);
    }
}

#[cfg(target_os = "none")]
mod embedded {
    use super::*;

    pub struct FakeGpio;

    impl FakeGpio {
        pub fn new() -> Self {
            FakeGpio
        }
    }

    impl LedHal for FakeGpio {
        fn led_on(&mut self) {}
        fn led_off(&mut self) {}
        fn delay_ms(&mut self, _ms: u32) {}
    }
}