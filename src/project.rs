use std::fs::File;
use std::io::{Read, Write};
use std::path::Path;
use std::sync::{Arc, Mutex};
use std::time::{Duration, SystemTime};

use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Default)]
pub enum WorkingMode {
    #[default]
    Idle,
    InProgress(WorkingProgress),
}

pub struct WorkingProgress {
    subject: Arc<Mutex<Subject>>,
    previous_tick: SystemTime,
}

impl WorkingProgress {
    fn start(subject: Arc<Mutex<Subject>>) -> Self {
        Self {
            subject,
            previous_tick: SystemTime::now(),
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct Backend {
    #[serde(with = "my_vec")]
    pub(crate) projects: Vec<Arc<Mutex<Project>>>,
    #[serde(skip)]
    pub(crate) current_project: Option<Arc<Mutex<Project>>>,
    #[serde(skip)]
    pub(crate) current_subject: Option<Arc<Mutex<Subject>>>,
    #[serde(skip)]
    pub(crate) working_mode: WorkingMode,
    pub(crate) current_session_duration: Duration,
    #[serde(with = "my_uuid")]
    pub(crate) last_session_project_id: Uuid,
    last_save: SystemTime,
}

impl Backend {
    pub fn load() -> Self {
        let config = Path::new("./data.ron");

        if config.exists() {
            if let Ok(mut file) = File::open("./data.ron") {
                let mut contents = String::new();
                if let Ok(_) = file.read_to_string(&mut contents) {
                    if let Ok(data) = ron::from_str(&*contents) {
                        return data;
                    }
                }
            }
        }

        Self {
            projects: vec![],
            current_project: None,
            current_subject: None,
            working_mode: Default::default(),
            current_session_duration: Duration::default(),
            last_session_project_id: Uuid::new_v4(),
            last_save: SystemTime::now(),
        }
    }

    fn dump(&mut self) {
        let mut file = File::create("./data.ron").unwrap();
        file.write_all(
            ron::ser::to_string_pretty(self, ron::ser::PrettyConfig::default())
                .unwrap()
                .as_bytes(),
        )
        .unwrap();
        self.last_save = SystemTime::now();
    }

    pub fn update_time(&mut self) {
        if let WorkingMode::InProgress(progress) = &mut self.working_mode {
            let duration = SystemTime::now()
                .duration_since(progress.previous_tick)
                .unwrap();

            progress.previous_tick = SystemTime::now();

            self.current_session_duration += duration;

            progress.subject.lock().unwrap().duration += duration;

            if SystemTime::now().duration_since(self.last_save).unwrap() > Duration::from_secs(1) {
                self.dump();
            }
        }
    }

    pub fn get_current_work_name(&self) -> String {
        if let Some(subject) = &self.current_subject {
            if let Some(project) = &self.current_project {
                return format!(
                    " {} - {}",
                    project.lock().unwrap().name,
                    subject.lock().unwrap().name
                );
            }
        }

        return "None".to_string();
    }

    pub fn add_project(&mut self, name: &str) {
        self.projects
            .push(Arc::new(Mutex::new(Project::create(name))));

        self.dump();
    }

    pub fn add_subject(&mut self, name: &str) {
        if let Some(project) = &self.current_project {
            project.lock().unwrap().add_subject(name);

            self.dump();
        }
    }

    pub fn start_subject(&mut self) {
        if let Some(subject) = &self.current_subject {
            if self.last_session_project_id != subject.lock().unwrap().id {
                self.current_session_duration = Duration::ZERO;
            }

            self.last_session_project_id = subject.lock().unwrap().id;

            self.working_mode = WorkingMode::InProgress(WorkingProgress::start(subject.clone()));
        }
    }

    pub fn stop_subject(&mut self) {
        self.working_mode = WorkingMode::Idle;
    }
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Project {
    #[serde(with = "my_uuid")]
    pub(crate) id: Uuid,
    pub(crate) name: String,
    #[serde(with = "my_vec")]
    pub(crate) subjects: Vec<Arc<Mutex<Subject>>>,
}

impl Project {
    pub fn create(name: &str) -> Self {
        Project {
            id: Uuid::new_v4(),
            name: name.to_string(),
            subjects: Vec::new(),
        }
    }

    pub fn add_subject(&mut self, name: &str) {
        self.subjects
            .push(Arc::new(Mutex::new(Subject::create(name))))
    }

    pub fn get_time(&self) -> Duration {
        self.subjects
            .iter()
            .fold(Duration::default(), |s, e| s + e.lock().unwrap().duration)
    }
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Subject {
    #[serde(with = "my_uuid")]
    pub(crate) id: Uuid,
    pub(crate) name: String,
    pub(crate) duration: Duration,
}

impl Subject {
    fn create(name: &str) -> Self {
        Subject {
            id: Uuid::new_v4(),
            name: name.to_string(),
            duration: Duration::default(),
        }
    }
}

mod my_vec {
    use serde::{Deserialize, Deserializer, Serialize, Serializer};
    use std::sync::{Arc, Mutex};

    pub fn serialize<S, T>(val: &Vec<Arc<Mutex<T>>>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
        T: Serialize,
        T: Clone,
    {
        let mut res = Vec::new();

        for v in val {
            let n = v.lock().unwrap().clone();
            res.push(n)
        }

        res.serialize(serializer)
    }

    pub fn deserialize<'de, D, T>(deserializer: D) -> Result<Vec<Arc<Mutex<T>>>, D::Error>
    where
        D: Deserializer<'de>,
        T: Deserialize<'de>,
        T: Clone,
    {
        let val: Vec<T> = Deserialize::deserialize(deserializer)?;

        Ok(val
            .iter()
            .map(|v| Arc::new(Mutex::new(v.clone())))
            .collect())
    }
}

mod my_uuid {
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
