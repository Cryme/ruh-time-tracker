use std::time::SystemTime;
use uuid::Uuid;

pub struct HistoryRecord {
    id: Uuid,
    start_date: SystemTime,
    end_date: SystemTime,
    project_id: Uuid,
    subject_id: Uuid,
}
