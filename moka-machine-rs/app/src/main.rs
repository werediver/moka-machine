#![no_std]
#![no_main]
#![feature(alloc_error_handler)]
#![feature(panic_info_message)]

mod uptime;
mod uptime_delay;

extern crate alloc;

use alloc::format;
use core::{alloc::Layout, fmt::Debug, panic::PanicInfo};

use alloc_cortex_m::CortexMHeap;
use app_core::controller::{Action, Controller};
use bsp::hal::{clocks::init_clocks_and_plls, entry, pac, Clock, Sio, Watchdog, I2C};
use embedded_hal::{adc::OneShot, digital::v2::OutputPin};
use rp_pico as bsp;
use rp_pico::hal;

use fugit::RateExtU32;
use libm::fabsf;
use rtt_target::{rprintln, rtt_init_print};

use mlx9061x::{Mlx9061x, SlaveAddr};

use embedded_graphics::{
    mono_font::{iso_8859_15::FONT_6X12, MonoTextStyle},
    pixelcolor::BinaryColor,
    prelude::Point,
    text::{Alignment, Text},
    Drawable,
};
use sh1106::{interface::DisplayInterface, prelude::*, Builder};

use crate::uptime::Uptime;

/// The voltage on Pico 3V3 pin as measured on a specific board
/// with the SMPS in PWM mode (GPIO32 high).
const VDD: f32 = 3.3;

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

    let mut smps_pwm_pin = pins.b_power_save.into_push_pull_output();
    smps_pwm_pin.set_high().unwrap();

    let mut dbg_pin = pins.gpio0.into_push_pull_output();
    let mut led_pin = pins.led.into_push_pull_output();
    let mut heater_a = pins.gpio10.into_push_pull_output();
    let mut heater_b = pins.gpio11.into_push_pull_output();

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

    let i2c_disp = I2C::i2c1(
        pac.I2C1,
        pins.gpio18.into_mode(),
        pins.gpio19.into_mode(),
        400.kHz(),
        &mut pac.RESETS,
        clocks.system_clock.freq(),
    );

    let mut disp: GraphicsMode<_> = Builder::new().connect_i2c(i2c_disp).into();

    disp.init().unwrap();
    // Flushing the buffer of a just-initialized display driver clears the display.
    disp.flush().unwrap();

    let i2c_ncir = I2C::i2c0(
        pac.I2C0,
        pins.gpio12.into_mode(),
        pins.gpio13.into_mode(),
        100.kHz(),
        &mut pac.RESETS,
        clocks.system_clock.freq(),
    );

    let mut adc = hal::Adc::new(pac.ADC, &mut pac.RESETS);
    let mut mcu_temp_sensor = adc.enable_temp_sensor();

    let mut ncir = Mlx9061x::new_mlx90614(i2c_ncir, SlaveAddr::default(), 100).unwrap();

    let mut controller = Controller::new(0.2);
    controller.set_target_temp(Some(45.0));

    rprintln!("Ready");
    enable_heater(false);
    const EMISSIVITY: f32 = 0.8;
    if fabsf(ncir.emissivity().unwrap() - EMISSIVITY) > 0.001 {
        rprintln!("Setting NCIR emissivity to {}", EMISSIVITY);
        ncir.set_emissivity(EMISSIVITY, &mut uptime).unwrap();
    }
    Uptime::delay_ms(2000);

    let mut state = AppState::default();

    loop {
        dbg_pin.set_high().unwrap();

        state.mcu_temp = adc.read(&mut mcu_temp_sensor).map(pico_temp).unwrap();
        state.ncir_amb_temp = ncir
            .ambient_temperature()
            .map(ncir_temp)
            .unwrap_or(f32::NAN);
        state.ncir_obj_temp = ncir
            .object1_temperature()
            .map(ncir_temp)
            .unwrap_or(f32::NAN);
        disp_state(&state, &mut disp);

        if let Some(action) = controller.update(state.ncir_obj_temp) {
            match action {
                Action::EnableHeater => enable_heater(true),
                Action::DisableHeater => enable_heater(false),
            }
        }

        dbg_pin.set_low().unwrap();

        Uptime::delay_ms(200);
    }
}

#[derive(Debug)]
struct AppState {
    mcu_temp: f32,
    ncir_amb_temp: f32,
    ncir_obj_temp: f32,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            mcu_temp: f32::NAN,
            ncir_amb_temp: f32::NAN,
            ncir_obj_temp: f32::NAN,
        }
    }
}

fn disp_state<DI>(state: &AppState, disp: &mut GraphicsMode<DI>)
where
    DI: DisplayInterface,
    DI::Error: Debug,
{
    let char_style = MonoTextStyle::new(&FONT_6X12, BinaryColor::On);

    disp.clear();

    Text::with_alignment(
        format!("MCU: {:.2}°C", state.mcu_temp).as_str(),
        Point::new(0, 12),
        char_style,
        Alignment::Left,
    )
    .draw(disp)
    .unwrap();

    Text::with_alignment(
        format!("Amb: {:.2}°C", state.ncir_amb_temp).as_str(),
        Point::new(0, 12 * 2),
        char_style,
        Alignment::Left,
    )
    .draw(disp)
    .unwrap();

    Text::with_alignment(
        format!("Obj: {:.2}°C", state.ncir_obj_temp).as_str(),
        Point::new(0, 12 * 3),
        char_style,
        Alignment::Left,
    )
    .draw(disp)
    .unwrap();

    disp.flush().unwrap();
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

    const ADC_VREF: f32 = VDD - R * (I_ADC + I_TS_BIAS);

    let adc_voltage = raw as f32 / 0xfff as f32 * ADC_VREF;

    27.0 - (adc_voltage - 0.706) / 0.001721
}

fn ncir_temp(t: f32) -> f32 {
    const VDD_CAL: f32 = 3.0;

    let is_valid = t < (0x8000 as f32) * 0.02 - 273.15;

    if is_valid {
        t - (VDD - VDD_CAL) * 0.6
    } else {
        f32::NAN
    }
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
