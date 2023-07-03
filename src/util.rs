use std::ops::Rem;
use std::time::Duration;

pub fn format_duration(duration: Duration) -> String {
    const HOUR_S: f64 = 60.0 * 60.0;

    let spent = duration.as_secs_f64();

    let hours = (spent / HOUR_S).trunc();
    let minutes = (spent.rem(&HOUR_S) / 60.0).trunc();

    let hours_d = if hours > 9.0 {
        format!("{hours}")
    } else {
        format!("0{hours}")
    };
    let minutes_d = if minutes > 9.0 {
        format!("{minutes}")
    } else {
        format!("0{minutes}")
    };

    format!(" {hours_d}:{minutes_d}")
}
