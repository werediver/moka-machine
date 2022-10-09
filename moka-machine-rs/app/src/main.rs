#![no_std]
#![no_main]
#![feature(alloc_error_handler)]
#![feature(panic_info_message)]

mod uptime;
mod uptime_delay;

use core::{alloc::Layout, panic::PanicInfo};

use alloc_cortex_m::CortexMHeap;
use app_core::controller::{Action, Controller};
use bsp::hal::{clocks::init_clocks_and_plls, entry, pac, Clock, Sio, Watchdog, I2C};
use embedded_hal::{adc::OneShot, blocking::delay::DelayMs, digital::v2::OutputPin};
use rp_pico as bsp;
use rp_pico::hal;

use fugit::RateExtU32;
use libm::fabsf;
use rtt_target::{rprintln, rtt_init_print};

use mlx9061x::{Mlx9061x, SlaveAddr};

use crate::uptime::Uptime;

#[entry]
fn main() -> ! {
    rtt_init_print!();
    init_heap();

    let mut pac = pac::Peripherals::take().unwrap();
    let sio = Sio::new(pac.SIO);

    let core = pac::CorePeripherals::take().unwrap();
    let mut uptime = Uptime::new(core.SYST);

    let pins = bsp::Pins::new(
        pac.IO_BANK0,
        pac.PADS_BANK0,
        sio.gpio_bank0,
        &mut pac.RESETS,
    );

    let mut watchdog = Watchdog::new(pac.WATCHDOG);

    // External crystal on the Pico board is 12 Mhz
    let external_xtal_freq_hz = 12_000_000u32;
    let clocks = init_clocks_and_plls(
        external_xtal_freq_hz,
        pac.XOSC,
        pac.CLOCKS,
        pac.PLL_SYS,
        pac.PLL_USB,
        &mut pac.RESETS,
        &mut watchdog,
    )
    .ok()
    .unwrap();

    let mut led_pin = pins.led.into_push_pull_output();
    let mut heater_a = pins.gpio14.into_push_pull_output();
    let mut heater_b = pins.gpio15.into_push_pull_output();

    let mut enable_heater = move |enable: bool| {
        if enable {
            led_pin.set_high().unwrap();

            heater_a.set_low().unwrap();
            Uptime::delay_us(100);
            heater_b.set_low().unwrap();
        } else {
            led_pin.set_low().unwrap();

            heater_a.set_high().unwrap();
            heater_b.set_high().unwrap();
        }
    };

    let i2c = I2C::i2c0(
        pac.I2C0,
        pins.gpio12.into_mode(),
        pins.gpio13.into_mode(),
        100.kHz(),
        &mut pac.RESETS,
        clocks.system_clock.freq(),
    );

    let mut adc = hal::Adc::new(pac.ADC, &mut pac.RESETS);
    let mut mcu_temp_sensor = adc.enable_temp_sensor();

    let mut ncir = Mlx9061x::new_mlx90614(i2c, SlaveAddr::default(), 100).unwrap();

    let mut controller = Controller::new(0.2);
    controller.set_target_temp(Some(45.0));

    rprintln!("Ready");
    enable_heater(false);
    if fabsf(ncir.emissivity().unwrap() - 0.8) > 0.001 {
        const EMISSIVITY: f32 = 0.8;
        rprintln!("Setting NCIR emissivity to {}", EMISSIVITY);
        ncir.set_emissivity(EMISSIVITY, &mut uptime);
    }
    Uptime::delay_ms(2000);

    loop {
        let raw = adc.read(&mut mcu_temp_sensor).unwrap();
        let mcu_temp = pico_temp(raw);
        let amb_temp = ncir.ambient_temperature().unwrap_or(f32::NAN);
        let obj_temp = ncir.object1_temperature().unwrap_or(f32::NAN);
        rprintln!("mcu: {:.2}", mcu_temp);
        rprintln!("amb: {:.2}, obj: {:.2}", amb_temp, obj_temp);

        if let Some(action) = controller.update(obj_temp) {
            match action {
                Action::EnableHeater => enable_heater(true),
                Action::DisableHeater => enable_heater(false),
            }
        }

        Uptime::delay_ms(200);
    }
}

/// Convert an ADC temperature sensor readout to degrees Celsius.
///
/// Tuned for the Pico board. May not be valid on a different board.
fn pico_temp(raw: u16) -> f32 {
    /// R7 on the Pico board.
    const R: f32 = 200.0;
    /// Approximate ADC current draw.
    const I_ADC: f32 = 150e-6;
    /// Approximate temperature sensor BJT/diode bias current.
    const I_TS_BIAS: f32 = 40e-6;

    const ADC_VREF: f32 = 3.3 - R * (I_ADC + I_TS_BIAS);

    let adc_voltage = raw as f32 / 0xfff as f32 * ADC_VREF;

    27.0 - (adc_voltage - 0.706) / 0.001721
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    rprintln!("{}", info);
    loop {
        cortex_m::asm::wfi();
    }
}

#[alloc_error_handler]
fn oom(layout: Layout) -> ! {
    rprintln!(
        "failed to allocate {} bytes aligned on {} bytes)",
        layout.size(),
        layout.align()
    );
    loop {
        cortex_m::asm::wfi();
    }
}

#[global_allocator]
static ALLOCATOR: CortexMHeap = CortexMHeap::empty();

fn init_heap() {
    use core::mem::MaybeUninit;
    const HEAP_SIZE: usize = 128 * 1024;
    static mut HEAP: [MaybeUninit<u8>; HEAP_SIZE] = [MaybeUninit::uninit(); HEAP_SIZE];
    unsafe { ALLOCATOR.init(HEAP.as_ptr() as usize, HEAP_SIZE) }
}
