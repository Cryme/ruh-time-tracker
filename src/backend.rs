use crate::history::History;
use crate::util;

use std::collections::HashMap;
use std::fs::File;
use std::io::{Read, Write};
use std::path::Path;

use std::sync::{Arc, Mutex};

use std::time::{Duration, SystemTime};

use rand::{thread_rng, Rng};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use util::{my_hash_map_mutex, my_uuid};

#[derive(Default)]
pub enum WorkingMode {
    #[default]
    Idle,
    InProgress(WorkingProgress),
}

pub struct WorkingProgress {
    subject: Arc<Mutex<Subject>>,
    session_id: Uuid,
    previous_tick: SystemTime,
}

impl WorkingProgress {
    fn start(subject: Arc<Mutex<Subject>>, session_id: Uuid) -> Self {
        Self {
            subject,
            session_id,
            previous_tick: SystemTime::now(),
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct Backend {
    #[serde(with = "my_hash_map_mutex")]
    pub(crate) projects: HashMap<Uuid, Arc<Mutex<Project>>>,
    #[serde(with = "my_hash_map_mutex")]
    pub(crate) todos: HashMap<Uuid, Arc<Mutex<TodoProject>>>,
    #[serde(skip)]
    pub(crate) current_project: Option<Arc<Mutex<Project>>>,
    #[serde(skip)]
    pub(crate) current_subject: Option<Arc<Mutex<Subject>>>,
    #[serde(skip)]
    pub(crate) current_todo_project: Option<Arc<Mutex<TodoProject>>>,
    #[serde(skip)]
    pub(crate) working_mode: WorkingMode,
    #[serde(skip)]
    pub(crate) dirty: bool,
    pub(crate) current_session_duration: Duration,
    #[serde(with = "my_uuid")]
    pub(crate) last_session_subject_id: Uuid,
    last_save: SystemTime,
    pub(crate) history: History,
}

impl Backend {
    pub fn load() -> Self {
        let config = Path::new("./data.ron");

        if config.exists() {
            if let Ok(mut file) = File::open("./data.ron") {
                let mut contents = String::new();
                if file.read_to_string(&mut contents).is_ok() {
                    if let Ok(data) = ron::from_str::<Backend>(&contents) {
                        return data;
                    }
                }
            }
        }

        Self {
            projects: HashMap::new(),
            current_project: None,
            current_subject: None,
            current_todo_project: None,
            working_mode: Default::default(),
            current_session_duration: Duration::default(),
            last_session_subject_id: Uuid::new_v4(),
            last_save: SystemTime::now(),
            history: History::new(),
            todos: HashMap::new(),
            dirty: false,
        }
    }

    pub fn mark_dirty(&mut self){
        self.dirty = true;
    }

    pub fn set_current_subject(&mut self, subject: Option<Arc<Mutex<Subject>>>) {
        self.current_subject = subject;
    }

    pub fn set_current_project(&mut self, project: Option<Arc<Mutex<Project>>>) {
        self.current_project = project;
    }

    pub fn set_current_todo_project(&mut self, project: Option<Arc<Mutex<TodoProject>>>) {
        self.current_todo_project = project;
    }

    pub(crate) fn dump(&mut self) {
        let mut file = File::create("./data.ron").unwrap();
        file.write_all(
            ron::ser::to_string_pretty(self, ron::ser::PrettyConfig::default())
                .unwrap()
                .as_bytes(),
        )
        .unwrap();
        self.last_save = SystemTime::now();
        self.dirty = false;
    }

    pub fn update_time(&mut self) {
        if let WorkingMode::InProgress(progress) = &mut self.working_mode {
            let duration = SystemTime::now()
                .duration_since(progress.previous_tick)
                .unwrap();

            progress.previous_tick = SystemTime::now();

            self.current_session_duration += duration;

            progress.subject.lock().unwrap().duration += duration;

            self.history.update(progress.session_id);

            if SystemTime::now().duration_since(self.last_save).unwrap() > Duration::from_secs(10) {
                self.dump();
            }
        }

        if self.dirty {
            self.dump();
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

        "None".to_string()
    }

    pub fn add_todo_project(&mut self, name: &str) {
        let project = TodoProject::create(name);
        self.todos
            .insert(project.id, Arc::new(Mutex::new(project)));

        self.mark_dirty();
    }

    pub fn add_todo_subject(&mut self, name: &str) {
        if let Some(project) = &self.current_todo_project {
            project.lock().unwrap().add_subject(name);

            self.mark_dirty();
        }
    }

    pub fn add_project(&mut self, name: &str) {
        let project = Project::create(name);
        self.projects
            .insert(project.id, Arc::new(Mutex::new(project)));

        self.mark_dirty();
    }

    pub fn add_subject(&mut self, name: &str) {
        if let Some(project) = &self.current_project {
            project.lock().unwrap().add_subject(name);

            self.mark_dirty();
        }
    }

    pub fn start_subject(&mut self) {
        if let Some(subject) = &self.current_subject {
            if let Some(project) = &self.current_project {
                let subject_id = subject.lock().unwrap().id;

                if self.last_session_subject_id != subject_id {
                    self.current_session_duration = Duration::ZERO;
                }

                self.last_session_subject_id = subject_id;

                let project = project.lock().unwrap();
                self.working_mode = WorkingMode::InProgress(WorkingProgress::start(
                    subject.clone(),
                    self.history.add_record(project.id, subject_id),
                ));
            }
        }
    }

    pub fn stop_subject(&mut self, force: bool) {
        self.working_mode = WorkingMode::Idle;

        if force {
            self.current_session_duration = Duration::ZERO;
        } else if let Some(subject) = &self.current_subject {
            if self.last_session_subject_id != subject.lock().unwrap().id {
                self.current_session_duration = Duration::ZERO;
            }
        }
    }
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Project {
    #[serde(with = "my_uuid")]
    pub(crate) id: Uuid,
    pub(crate) name: String,
    #[serde(with = "my_hash_map_mutex")]
    pub(crate) subjects: HashMap<Uuid, Arc<Mutex<Subject>>>,
    pub(crate) is_deleted: bool,
    pub(crate) color: (u8, u8, u8),
}

impl Project {
    pub fn create(name: &str) -> Self {
        let mut rng = thread_rng();

        Project {
            id: Uuid::new_v4(),
            name: name.to_string(),
            subjects: HashMap::new(),
            is_deleted: false,
            color: (rng.gen(), rng.gen(), rng.gen()),
        }
    }

    pub fn add_subject(&mut self, name: &str) {
        let subject = Subject::create(name);

        self.subjects
            .insert(subject.id, Arc::new(Mutex::new(subject)));
    }

    pub fn get_time(&self) -> Duration {
        self.subjects.iter().fold(Duration::default(), |s, (_, e)| {
            s + e.lock().unwrap().duration
        })
    }
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Subject {
    #[serde(with = "my_uuid")]
    pub(crate) id: Uuid,
    pub(crate) name: String,
    pub(crate) duration: Duration,
    pub(crate) is_deleted: bool,
}

impl Subject {
    fn create(name: &str) -> Self {
        Subject {
            id: Uuid::new_v4(),
            name: name.to_string(),
            duration: Duration::default(),
            is_deleted: false,
        }
    }
}

#[derive(Serialize, Deserialize, Clone)]
pub struct TodoProject {
    #[serde(with = "my_uuid")]
    pub(crate) id: Uuid,
    pub(crate) name: String,
    #[serde(with = "my_hash_map_mutex")]
    pub(crate) subjects: HashMap<Uuid, Arc<Mutex<TodoSubject>>>,
    pub(crate) is_deleted: bool,
}

impl TodoProject {
    pub fn create(name: &str) -> Self {

        TodoProject {
            id: Uuid::new_v4(),
            name: name.to_string(),
            subjects: HashMap::new(),
            is_deleted: false,
        }
    }

    pub fn add_subject(&mut self, name: &str) {
        let subject = TodoSubject::create(name);

        self.subjects
            .insert(subject.id, Arc::new(Mutex::new(subject)));
    }
}

#[derive(Serialize, Deserialize, Clone)]
pub struct TodoSubject {
    #[serde(with = "my_uuid")]
    pub(crate) id: Uuid,
    pub(crate) name: String,
    pub(crate) is_deleted: bool,
    pub(crate) is_done: bool,
}

impl TodoSubject {
    fn create(name: &str) -> Self {
        TodoSubject {
            id: Uuid::new_v4(),
            name: name.to_string(),
            is_deleted: false,
            is_done: false,
        }
    }

    pub(crate) fn toggle(&mut self){
        self.is_done = !self.is_done;
    }
}
