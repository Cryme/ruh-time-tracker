use chrono::{DateTime, Datelike, Duration, Local, TimeZone};
use serde::{Deserialize, Serialize};
use std::cmp::Ordering;
use std::collections::HashMap;
use std::time::SystemTime;
use uuid::Uuid;

use crate::util;
use util::{my_hash_map, my_uuid};

#[derive(Clone, Serialize, Deserialize)]
pub struct History {
    #[serde(with = "my_hash_map")]
    records: HashMap<Uuid, HistoryRecord>,
}

impl History {
    pub fn new() -> Self {
        History {
            records: HashMap::new(),
        }
    }

    pub fn update(&mut self, id: Uuid) {
        if let Some(session) = self.records.get_mut(&id) {
            session.end_date = DateTime::from(SystemTime::now());
        }
    }

    pub fn add_record(&mut self, project_id: Uuid, subject_id: Uuid) -> Uuid {
        let id = Uuid::new_v4();

        self.records.insert(
            id,
            HistoryRecord {
                id,
                start_date: DateTime::from(SystemTime::now()),
                end_date: DateTime::from(SystemTime::now()),
                project_id,
                subject_id,
            },
        );

        id
    }

    pub fn get_ordered_records(
        &self,
        date_range: (DateTime<Local>, DateTime<Local>),
    ) -> Vec<Vec<HistoryRecord>> {
        let number_of_days = date_range.1.signed_duration_since(date_range.0).num_days() + 1;

        let mut res: Vec<Vec<HistoryRecord>> = (0..number_of_days).map(|_| Vec::new()).collect();

        let mut r: Vec<HistoryRecord> = self
            .records
            .values()
            .filter(|v| v.start_date >= date_range.0 && v.start_date <= date_range.1)
            .copied()
            .collect();

        r.sort();

        for record in r {
            let ind = (record
                .start_date
                .signed_duration_since(date_range.0)
                .num_days()) as usize;

            if record.start_date.day() != record.end_date.day() {
                let mut first_record = record;
                first_record.end_date = Local
                    .with_ymd_and_hms(
                        record.start_date.year(),
                        record.start_date.month(),
                        record.start_date.day(),
                        23,
                        59,
                        59,
                    )
                    .unwrap();

                res.get_mut(ind).unwrap().push(first_record);

                if let Some(vec) = res.get_mut(ind + 1) {
                    let mut second_record = record;
                    second_record.start_date = Local
                        .with_ymd_and_hms(
                            record.end_date.year(),
                            record.end_date.month(),
                            record.end_date.day(),
                            00,
                            00,
                            00,
                        )
                        .unwrap();

                    vec.push(second_record);
                }
            } else {
                res.get_mut(ind).unwrap().push(record);
            }
        }

        res
    }

    pub fn get_records(
        &self,
        date_range: (DateTime<Local>, DateTime<Local>),
    ) -> Vec<HistoryRecord> {
        self.records
            .values()
            .filter(|v| {
                v.start_date >= date_range.0
                    && v.start_date < date_range.1
            })
            .copied()
            .collect()
    }
}

#[derive(Copy, Clone, Serialize, Deserialize, Debug)]
pub struct HistoryRecord {
    #[serde(with = "my_uuid")]
    pub id: Uuid,
    pub start_date: DateTime<Local>,
    pub end_date: DateTime<Local>,
    #[serde(with = "my_uuid")]
    pub project_id: Uuid,
    #[serde(with = "my_uuid")]
    pub subject_id: Uuid,
}

impl Eq for HistoryRecord {}

impl PartialEq<Self> for HistoryRecord {
    fn eq(&self, other: &Self) -> bool {
        self.start_date.eq(&other.start_date)
    }
}

impl PartialOrd<Self> for HistoryRecord {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.start_date.partial_cmp(&other.start_date)
    }
}

impl Ord for HistoryRecord {
    fn cmp(&self, other: &Self) -> Ordering {
        self.start_date.cmp(&other.start_date)
    }
}

impl HistoryRecord {
    pub fn get_duration(&self) -> Duration {
        self.end_date.signed_duration_since(self.start_date)
    }
}
