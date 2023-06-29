use std::sync::{Arc};
use std::time::Duration;
use eframe::egui::Key::D;
use uuid::Uuid;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct Project {
    #[serde(with = "my_uuid")]
    id: Uuid,
    name: String,
    #[serde(with = "my_vec")]
    subjects: Vec<Arc<Subject>>,
    #[serde(skip)]
    current_subject: Option<Arc<Subject>>
}


impl Project {
    pub fn create(name: &str) -> Self {
        Project{
            id: Uuid::new_v4(),
            name: name.to_string(),
            subjects: Vec::new(),
            current_subject: None,
        }
    }

    pub fn select_subject(&mut self, index: usize) {
        if let Some(subject) = self.subjects.get(index){
            self.current_subject = Some(subject.clone());
        }

    }

    pub fn add_subject(&mut self, name: &str) {
        self.subjects.push(
            Arc::new(Subject::create(name))
        )
    }
}


#[derive(Serialize, Deserialize, Clone)]
pub struct Subject {
    #[serde(with = "my_uuid")]
    id: Uuid,
    name: String,
    duration: Duration,
}

impl Subject {
    fn create(name: &str) -> Self {
        Subject{
            id: Uuid::new_v4(),
            name: name.to_string(),
            duration: Duration::default(),
        }
    }
}


mod my_vec {
    use serde::{de::Error, Deserialize, Deserializer, Serialize, Serializer};
    use std::str::FromStr;
    use std::sync::Arc;
    use crate::project::Subject;

    pub fn serialize<S>(val: &Vec<Arc<Subject>>, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: Serializer,
    {
        let mut res = Vec::new();

        for v in val{
            let n = (**v).clone();
            res.push(n)
        }

        res.serialize(serializer)
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Vec<Arc<Subject>>, D::Error>
        where
            D: Deserializer<'de>,
    {
        let val: Vec<Subject> = Deserialize::deserialize(deserializer)?;

        Ok(val.iter().map(|v|  Arc::new(v.clone())).collect())
    }
}

mod my_uuid {
    use uuid::Uuid;
    use serde::{de::Error, Deserialize, Deserializer, Serialize, Serializer};
    use std::str::FromStr;

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