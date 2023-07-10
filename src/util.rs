use chrono::{Datelike, DateTime, Local, NaiveDate};
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

pub mod my_hash_map_mutex {
    use serde::{Deserialize, Deserializer, Serialize, Serializer};
    use std::collections::HashMap;
    use std::sync::{Arc, Mutex};
    use uuid::Uuid;

    pub fn serialize<S, T>(
        val: &HashMap<Uuid, Arc<Mutex<T>>>,
        serializer: S,
    ) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
        T: Serialize,
        T: Clone,
    {
        let mut res = HashMap::new();

        for v in val {
            res.insert(v.0.to_string(), v.1.lock().unwrap().clone());
        }

        res.serialize(serializer)
    }

    pub fn deserialize<'de, D, T>(deserializer: D) -> Result<HashMap<Uuid, Arc<Mutex<T>>>, D::Error>
    where
        D: Deserializer<'de>,
        T: Deserialize<'de>,
        T: Clone,
    {
        let val: HashMap<String, T> = Deserialize::deserialize(deserializer)?;
        let mut res: HashMap<Uuid, Arc<Mutex<T>>> = HashMap::new();

        for v in &val {
            res.insert(
                Uuid::parse_str(v.0).unwrap(),
                Arc::new(Mutex::new(v.1.clone())),
            );
        }

        Ok(res)
    }
}

pub mod my_hash_map {
    use serde::{Deserialize, Deserializer, Serialize, Serializer};
    use std::collections::HashMap;
    use uuid::Uuid;

    pub fn serialize<S, T>(val: &HashMap<Uuid, T>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
        T: Serialize,
        T: Clone,
    {
        let mut res = HashMap::new();

        for v in val {
            res.insert(v.0.to_string(), v.1.clone());
        }

        res.serialize(serializer)
    }

    pub fn deserialize<'de, D, T>(deserializer: D) -> Result<HashMap<Uuid, T>, D::Error>
    where
        D: Deserializer<'de>,
        T: Deserialize<'de>,
        T: Clone,
    {
        let val: HashMap<String, T> = Deserialize::deserialize(deserializer)?;
        let mut res: HashMap<Uuid, T> = HashMap::new();

        for v in &val {
            res.insert(Uuid::parse_str(v.0).unwrap(), v.1.clone());
        }

        Ok(res)
    }
}

pub mod my_uuid {
    use serde::{de::Error, Deserialize, Deserializer, Serialize, Serializer};
    use uuid::Uuid;

    pub fn serialize<S>(val: &Uuid, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        val.to_string().serialize(serializer)
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Uuid, D::Error>
    where
        D: Deserializer<'de>,
    {
        let val: &str = Deserialize::deserialize(deserializer)?;
        Uuid::parse_str(val).map_err(D::Error::custom)
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
