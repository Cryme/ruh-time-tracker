use chrono::{DateTime, Local};
use eframe::epaint::ahash::HashMap;
use std::cmp::Ordering;
use uuid::Uuid;

pub struct History {
    records: HashMap<Uuid, HistoryRecord>,
}

impl History {
    pub fn get_ordered_records(
        &self,
        date_range: (DateTime<Local>, DateTime<Local>),
    ) -> Vec<HistoryRecord> {
        let mut r: Vec<HistoryRecord> = self
            .records
            .values()
            .filter(|v| v.start_date >= date_range.0 && v.start_date <= date_range.1)
            .map(|v| *v)
            .collect();

        r.sort();

        r
    }
}

#[derive(Copy, Clone)]
pub struct HistoryRecord {
    id: Uuid,
    start_date: DateTime<Local>,
    end_date: DateTime<Local>,
    project_id: Uuid,
    subject_id: Uuid,
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

impl HistoryRecord {}
