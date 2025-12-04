//!
//! # Blinky Task
//!

use crate::{hal, system::BlinkySrc};
use hal::gpio::{Level, Output as OP, Speed};
use hal::wdg::IndependentWatchdog as Dog;

#[embassy_executor::task]
pub async fn task(p: BlinkySrc) -> ! {
    let mut t = utils::init_ticker!(150); // 150ms

    let mut led = OP::new(p.led_pin, Level::Low, Speed::Low);
    let mut dog = Dog::new(p.iwdg_p, 200_000); // 200ms

    // dog.unleash(); // Start the WatchDog

    loop {
        (led.toggle(), dog.pet());
        t.next().await
    }
}
