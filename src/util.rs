use chrono::{DateTime, Datelike, Local, NaiveDate};
use std::ops::Rem;
use std::time::Duration;

pub fn format_duration(duration: Duration) -> String {
    const HOUR_S: f64 = 60.0 * 60.0;

    let spent = duration.as_secs_f64();

    let hours = (spent / HOUR_S).trunc();
    let minutes = (spent.rem(&HOUR_S) / 60.0).trunc();

    format!(
        " {}:{}",
        format_number(hours as u32),
        format_number(minutes as u32)
    )
}
pub fn format_chrono_duration(duration: chrono::Duration) -> String {
    const HOUR_S: f64 = 60.0 * 60.0;

    let spent = duration.num_seconds() as f64;

    let hours = (spent / HOUR_S).trunc();
    let minutes = (spent.rem(&HOUR_S) / 60.0).trunc();

    format!(
        " {}:{}",
        format_number(hours as u32),
        format_number(minutes as u32)
    )
}

pub fn format_number<T>(number: T) -> String
where
    T: Into<u32>,
{
    let number = number.into();

    if number > 9 {
        format!("{number}")
    } else {
        format!("0{number}")
    }
}

pub fn get_days_from_month(year: i32, month: u32) -> u32 {
    NaiveDate::from_ymd_opt(
        match month {
            12 => year + 1,
            _ => year,
        },
        match month {
            12 => 1,
            _ => month + 1,
        },
        1,
    )
    .unwrap()
    .signed_duration_since(NaiveDate::from_ymd_opt(year, month, 1).unwrap())
    .num_days() as u32
}

pub fn calendar_days_count(from: DateTime<Local>, to: DateTime<Local>) -> u32 {
    let rf = NaiveDate::from_ymd_opt(from.year(), from.month(), from.day()).unwrap();
    let rt = NaiveDate::from_ymd_opt(to.year(), to.month(), to.day()).unwrap();

    rt.signed_duration_since(rf).num_days() as u32
}
