//!
//! # System Resources
//!
//! ## Reserved Resources
//!

use super::private::*;

assign_resources! {
    /// for `Blinky` task.
    blinky: BlinkySrc {
        iwdg_p: IWDG,
        led_pin: PC13,
    }
}
