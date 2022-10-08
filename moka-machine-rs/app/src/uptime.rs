use cortex_m::peripheral::{syst::SystClkSource, SYST};
use cortex_m_rt::exception;

use app_core::common::Instant;
use fugit::Duration;

pub struct Uptime {
    _syst: SYST,
}

impl Uptime {
    pub fn new(mut syst: SYST) -> Self {
        syst.set_clock_source(SystClkSource::External);

        let ticks_per_1s = (SYST::get_ticks_per_10ms() + 1) * 100 - 1;

        syst.set_reload(ticks_per_1s);
        syst.clear_current();
        syst.enable_counter();
        syst.enable_interrupt();

        Uptime { _syst: syst }
    }

    pub fn get_us() -> u64 {
        let uptime_s = cortex_m::interrupt::free(|_| unsafe { SYST_RELOAD_COUNT });
        u64::from(uptime_s) * 1_000_000 + u64::from(SYST::get_reload() - SYST::get_current())
    }

    pub fn get_instant() -> Instant {
        Instant::from_ticks(Self::get_us())
    }

    pub fn delay_us(us: u64) {
        let start = Self::get_instant();
        let delay = Duration::<u64, 1, 1_000_000>::from_ticks(us);
        let end = start
            .checked_add_duration(delay)
            .expect("uptime must not overflow during the delay");
        while Self::get_instant() < end {}
    }

    pub fn delay_ms(ms: u64) {
        let start = Self::get_instant();
        let delay = Duration::<u64, 1, 1_000>::from_ticks(ms);
        let end = start
            .checked_add_duration(delay)
            .expect("uptime must not overflow during the delay");
        while Self::get_instant() < end {}
    }
}

#[exception]
fn SysTick() {
    unsafe {
        SYST_RELOAD_COUNT += 1;
    }
}

static mut SYST_RELOAD_COUNT: u32 = 0;
