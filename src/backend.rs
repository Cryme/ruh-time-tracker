use crate::history::History;
use std::cmp::Ordering;

use std::collections::HashMap;
use std::fs::File;
use std::hash::Hash;
use std::io::{Read, Write};
use std::path::Path;

use std::sync::{Arc, Mutex};

use std::time::{Duration, SystemTime};

use rand::{thread_rng, Rng};
use serde::de::DeserializeOwned;
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

pub trait PreferVariant {
    fn get_prefer() -> Self;
}

impl PreferVariant for Uuid {
    fn get_prefer() -> Self {
        Uuid::new_v4()
    }
}

#[derive(Serialize, Deserialize, Clone)]
pub struct PContainer<T, K: Eq + Hash> {
    pub(crate) id: K,
    pub(crate) name: String,
    pub(crate) created_at: SystemTime,
    pub(crate) is_deleted: bool,
    pub(crate) color: (u8, u8, u8),
    pub(crate) inner: HashMap<K, T>,
    pub(crate) current_inner_id: Option<K>,
}

impl<T: Serialize + DeserializeOwned + Clone, K: PreferVariant + Eq + Hash + Serialize + DeserializeOwned + Copy + Clone> PContainer<T, K> {
    fn new(name: &str) -> Self {
        let mut rng = thread_rng();
        Self {
            id: K::get_prefer(),
            name: name.to_string(),
            created_at: SystemTime::now(),
            is_deleted: false,
            color: (rng.gen(), rng.gen(), rng.gen()),
            inner: HashMap::new(),
            current_inner_id: None,
        }
    }

    pub(crate) fn get_inner_sorted<F>(&self, sort_f: F) -> Vec<T>
    where
        F: FnMut(&T, &T) -> Ordering,
    {
        let mut c: Vec<T> = self.inner.values().cloned().collect();

        c.sort_by(sort_f);

        c
    }

    fn get_current_mut(&mut self) -> Option<&mut T> {
        if let Some(id) = &self.current_inner_id {
            let Some(pr) = self.inner.get_mut(id) else {
                self.current_inner_id = None;

                return None;
            };

            return Some(pr);
        }

        None
    }
    fn get_current(&self) -> Option<&T> {
        if let Some(id) = &self.current_inner_id {
            return self.inner.get(id);
        }

        None
    }

    fn set_current(&mut self, key: Option<K>) {
        if let Some(key) = &key {
            if !self.inner.contains_key(key) {
                return;
            }
        }

        self.current_inner_id = key;
    }
}

pub type IdType = Uuid;
pub type ProjectChain = PContainer<PContainer<PContainer<Arc<Mutex<Subject>>, IdType>, IdType>, IdType>;
pub type TodoChain = PContainer<PContainer<PContainer<Arc<Mutex<TodoSubject>>, IdType>, IdType>, IdType>;

#[derive(Serialize, Deserialize)]
pub struct Backend {
    pub(crate) projects: ProjectChain,
    pub(crate) todos: TodoChain,
    #[serde(skip)]
    pub(crate) working_mode: WorkingMode,
    #[serde(skip)]
    pub(crate) dirty: bool,
    pub(crate) current_session_duration: Duration,
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

        Self::default()
    }

    pub fn dirty(&mut self) {
        self.dirty = true;
    }

    pub fn get_current_subject(&self) -> Option<Arc<Mutex<Subject>>> {
        if let Some(project) = self.projects.get_current() {
            if let Some(sub_project) = project.get_current() {
                if let Some(subject) = sub_project.get_current() {
                    return Some(subject.clone());
                }
            }
        }

        None
    }

    pub fn get_current_sub_project(&self) -> Option<&PContainer<Arc<Mutex<Subject>>, IdType>> {
        let Some(current_project) = self.projects.get_current() else {
            return None;
        };

        current_project.get_current()
    }

    pub fn get_current_project(&self) -> Option<&PContainer<PContainer<Arc<Mutex<Subject>>, IdType>, IdType>> {
        self.projects.get_current()
    }

    pub fn set_current_subject(&mut self, subject_key: Option<Uuid>) {
        let Some(current_project) = self.projects.get_current_mut() else {
            return;
        };

        let Some(current_sub_project) = current_project.get_current_mut() else {
            return;
        };

        current_sub_project.set_current(subject_key);
    }

    pub fn set_current_sub_project(&mut self, sub_project_key: Option<Uuid>) {
        let Some(current_project) = self.projects.get_current_mut() else {
            return;
        };

        current_project.set_current(sub_project_key);
    }

    pub fn set_current_project(&mut self, project_key: Option<Uuid>) {
        self.projects.set_current(project_key)
    }

    pub fn get_current_todo_subject(&self) -> Option<Arc<Mutex<TodoSubject>>> {
        if let Some(project) = self.todos.get_current() {
            if let Some(sub_project) = project.get_current() {
                if let Some(subject) = sub_project.get_current() {
                    return Some(subject.clone());
                }
            }
        }

        None
    }

    pub fn get_current_todo_sub_project(&self) -> Option<&PContainer<Arc<Mutex<TodoSubject>>, IdType>> {
        let Some(current_project) = self.todos.get_current() else {
            return None;
        };

        current_project.get_current()
    }

    pub fn get_current_todo_project(
        &self,
    ) -> Option<&PContainer<PContainer<Arc<Mutex<TodoSubject>>, IdType>, IdType>> {
        self.todos.get_current()
    }

    pub fn set_current_todo_sub_project(&mut self, sub_project_key: Option<Uuid>) {
        let Some(current_project) = self.todos.get_current_mut() else {
            return;
        };

        current_project.set_current(sub_project_key);
    }

    pub fn set_current_todo_project(&mut self, project_key: Option<Uuid>) {
        self.todos.set_current(project_key)
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

    pub fn get_project_time(&self, key: &Uuid) -> Option<Duration> {
        if let Some(project) = self.projects.inner.get(key) {
            return Some(
                project
                    .inner
                    .values()
                    .fold(Duration::default(), |s, inner| {
                        s + inner
                            .inner
                            .values()
                            .fold(Duration::default(), |v, iv| v + iv.lock().unwrap().duration)
                    }),
            );
        }

        None
    }

    pub fn get_sub_project_time(&self, key: &Uuid) -> Option<Duration> {
        if let Some(project) = self.projects.get_current() {
            if let Some(sub_project) = project.inner.get(key) {
                return Some(
                    sub_project
                        .inner
                        .values()
                        .fold(Duration::default(), |v, iv| v + iv.lock().unwrap().duration),
                );
            }
        }

        None
    }

    pub fn get_current_work_name(&self) -> String {
        if let Some(project) = self.projects.get_current() {
            if let Some(sub_project) = project.get_current() {
                if let Some(subject) = sub_project.get_current() {
                    return format!(
                        " {}/{}/{}",
                        project.name,
                        sub_project.name,
                        subject.lock().unwrap().name,
                    );
                }
            }
        }

        "None".to_string()
    }
    pub fn add_todo_project(&mut self, name: &str) {
        let project = PContainer::new(name);

        self.todos.inner.insert(project.id, project);

        self.dirty();
    }

    pub fn add_todo_sub_project(&mut self, name: &str) {
        let Some(project) = self.todos.get_current_mut() else {
            return;
        };

        let sub_project = PContainer::new(name);

        project.inner.insert(sub_project.id, sub_project);

        self.dirty();
    }

    pub fn add_todo_subject(&mut self, name: &str) {
        let Some(project) = self.todos.get_current_mut() else {
            return;
        };

        let Some(sub_project) = project.get_current_mut() else {
            return;
        };

        let subject = TodoSubject::create(name);

        sub_project
            .inner
            .insert(subject.id, Arc::new(Mutex::new(subject)));

        self.dirty();
    }

    pub fn add_project(&mut self, name: &str) {
        let project = PContainer::new(name);

        self.projects.inner.insert(project.id, project);

        self.dirty();
    }

    pub fn add_sub_project(&mut self, name: &str) {
        let Some(project) = self.projects.get_current_mut() else {
            return;
        };

        let sub_project = PContainer::new(name);

        project.inner.insert(sub_project.id, sub_project);

        self.dirty();
    }

    pub fn add_subject(&mut self, name: &str) {
        let Some(project) = self.projects.get_current_mut() else {
            return;
        };

        let Some(sub_project) = project.get_current_mut() else {
            return;
        };

        let subject = Subject::create(name);

        sub_project
            .inner
            .insert(subject.id, Arc::new(Mutex::new(subject)));

        self.dirty();
    }

    pub fn start_subject(&mut self) {
        let Some(project) = self.projects.get_current_mut() else {
            return;
        };

        let project_id = project.id;

        let Some(sub_project) = project.get_current_mut() else {
            return;
        };

        let sub_project_id = sub_project.id;

        let Some(subject) = sub_project.get_current_mut() else {
            return;
        };

        let subject_id = subject.lock().unwrap().id;

        if self.last_session_subject_id != subject_id {
            self.current_session_duration = Duration::ZERO;
        }

        self.last_session_subject_id = subject_id;

        self.working_mode = WorkingMode::InProgress(WorkingProgress::start(
            subject.clone(),
            self.history
                .add_record(project_id, sub_project_id, subject_id),
        ));
    }

    pub fn stop_subject(&mut self, force: bool) {
        self.working_mode = WorkingMode::Idle;

        if force {
            self.current_session_duration = Duration::ZERO;
        } else if let Some(subject) = self.get_current_subject() {
            if self.last_session_subject_id != subject.lock().unwrap().id {
                self.current_session_duration = Duration::ZERO;
            }
        }
    }
}

impl Default for Backend {
    fn default() -> Self {
        Self {
            projects: PContainer::new("root"),
            working_mode: Default::default(),
            current_session_duration: Duration::default(),
            last_session_subject_id: Uuid::new_v4(),
            last_save: SystemTime::now(),
            history: History::new(),
            todos: PContainer::new("root"),
            dirty: false,
        }
    }
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Subject {
    pub(crate) id: Uuid,
    pub(crate) name: String,
    pub(crate) created_at: SystemTime,
    pub(crate) duration: Duration,
    pub(crate) is_deleted: bool,
}

impl Subject {
    fn create(name: &str) -> Self {
        Subject {
            id: Uuid::new_v4(),
            name: name.to_string(),
            created_at: SystemTime::now(),
            duration: Duration::default(),
            is_deleted: false,
        }
    }
}

#[derive(Serialize, Deserialize, Clone)]
pub struct TodoSubject {
    pub(crate) id: Uuid,
    pub(crate) name: String,
    pub(crate) created_at: SystemTime,
    pub(crate) is_deleted: bool,
    pub(crate) is_done: bool,
}

impl TodoSubject {
    fn create(name: &str) -> Self {
        TodoSubject {
            id: Uuid::new_v4(),
            name: name.to_string(),
            created_at: SystemTime::now(),
            is_deleted: false,
            is_done: false,
        }
    }

    pub(crate) fn toggle(&mut self) {
        self.is_done = !self.is_done;
    }
}
