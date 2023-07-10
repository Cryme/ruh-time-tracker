use crate::backend::{Backend, WorkingMode};
use crate::custom_window_frame;
use crate::util::{calendar_days_count, format_duration, format_number, get_days_from_month};

use chrono::{DateTime, Datelike, Local, TimeZone, Timelike};
use eframe::egui;
use eframe::egui::scroll_area::ScrollBarVisibility;
use eframe::egui::{
    Align, Color32, FontId, Layout, RichText, Rounding, ScrollArea, Ui, Vec2, Visuals,
};
use eframe::epaint::RectShape;
use std::time::{Duration, SystemTime};
use uuid::Uuid;

const SAVE_PERIOD_SECONDS: u64 = 10_000;

#[derive(Copy, Clone, Debug, PartialEq, Default)]
pub enum DisplayMode {
    #[default]
    Time,
    Minimal,
    Statistic,
    Todo,
}

#[derive(Default)]
enum CurrentDialog {
    #[default]
    None,
    AddProject,
    AddSubject,
    AddTodoProject,
    AddTodoSubject,
}

#[derive(Default)]
pub struct Frontend {
    backend: Backend,

    current_display_mode: DisplayMode,

    dialog_options: DialogOptions,
    time_tracker_options: TimeTrackerOptions,
    minimal_time_tracker_options: MinimalTrackerOptions,
    todo_options: TodoOptions,
    statistic_options: StatisticOptions,
}

impl Frontend {
    pub fn init(cc: &eframe::CreationContext<'_>) -> Self {
        let context = cc.egui_ctx.clone();
        std::thread::spawn(move || loop {
            std::thread::sleep(Duration::from_millis(SAVE_PERIOD_SECONDS));
            context.request_repaint();
        });

        Self {
            backend: Backend::load(),
            ..Self::default()
        }
    }
}

impl eframe::App for Frontend {
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        match self.current_display_mode {
            DisplayMode::Statistic => {
                custom_window_frame(ctx, frame, "_", self.current_display_mode, |ui: &mut Ui| {
                    self.build_statistic(ui);
                });
            }

            DisplayMode::Time => {
                custom_window_frame(ctx, frame, "_", self.current_display_mode, |ui: &mut Ui| {
                    self.time_tracker_build(ui);
                });
            }

            DisplayMode::Todo => {
                custom_window_frame(ctx, frame, "_", self.current_display_mode, |ui: &mut Ui| {
                    self.todo_build(ui);
                });
            }

            DisplayMode::Minimal => {
                custom_window_frame(ctx, frame, "_", self.current_display_mode, |ui| {
                    self.minimal_time_tracker_build(ui);
                });
            }
        }

        self.backend.update_time();

        self.dialog_build(ctx);
    }

    fn clear_color(&self, _visuals: &Visuals) -> [f32; 4] {
        egui::Rgba::TRANSPARENT.to_array() // Make sure we don't paint anything behind the rounded corners
    }
}

/**
    Dialog block
**/

#[derive(Default)]
struct DialogOptions {
    current_dialog: CurrentDialog,
    buffer: String,
}

impl Frontend {
    fn dialog_build(&mut self, ctx: &egui::Context) {
        match self.dialog_options.current_dialog {
            CurrentDialog::None => {}

            CurrentDialog::AddProject => {
                egui::Window::new("Add Project")
                    .collapsible(false)
                    .resizable(false)
                    .show(ctx, |ui| {
                        ui.horizontal(|ui| {
                            ui.add(egui::TextEdit::singleline(&mut self.dialog_options.buffer));

                            if ui.button("Cancel").clicked() {
                                self.dialog_options.current_dialog = CurrentDialog::None;
                                self.dialog_options.buffer = "".to_string();
                            }

                            if ui.button("Add").clicked() {
                                self.dialog_options.current_dialog = CurrentDialog::None;
                                self.backend.add_project(&self.dialog_options.buffer);
                                self.dialog_options.buffer = "".to_string();
                            }
                        });
                    });
            }

            CurrentDialog::AddSubject => {
                egui::Window::new("Add Project")
                    .collapsible(false)
                    .resizable(false)
                    .show(ctx, |ui| {
                        ui.horizontal(|ui| {
                            ui.add(egui::TextEdit::singleline(&mut self.dialog_options.buffer));

                            if ui.button("Cancel").clicked() {
                                self.dialog_options.current_dialog = CurrentDialog::None;
                                self.dialog_options.buffer = "".to_string();
                            }

                            if ui.button("Add").clicked() {
                                self.dialog_options.current_dialog = CurrentDialog::None;
                                self.backend.add_subject(&self.dialog_options.buffer);
                                self.dialog_options.buffer = "".to_string();
                            }
                        });
                    });
            }

            CurrentDialog::AddTodoProject => {
                egui::Window::new("Add Project")
                    .collapsible(false)
                    .resizable(false)
                    .show(ctx, |ui| {
                        ui.horizontal(|ui| {
                            ui.add(egui::TextEdit::singleline(&mut self.dialog_options.buffer));

                            if ui.button("Cancel").clicked() {
                                self.dialog_options.current_dialog = CurrentDialog::None;
                                self.dialog_options.buffer = "".to_string();
                            }

                            if ui.button("Add").clicked() {
                                self.dialog_options.current_dialog = CurrentDialog::None;
                                self.backend.add_todo_project(&self.dialog_options.buffer);
                                self.dialog_options.buffer = "".to_string();
                            }
                        });
                    });
            }

            CurrentDialog::AddTodoSubject => {
                egui::Window::new("Add Subject")
                    .collapsible(false)
                    .resizable(false)
                    .show(ctx, |ui| {
                        ui.horizontal(|ui| {
                            ui.add(egui::TextEdit::singleline(&mut self.dialog_options.buffer));

                            if ui.button("Cancel").clicked() {
                                self.dialog_options.current_dialog = CurrentDialog::None;
                                self.dialog_options.buffer = "".to_string();
                            }

                            if ui.button("Add").clicked() {
                                self.dialog_options.current_dialog = CurrentDialog::None;
                                self.backend.add_todo_subject(&self.dialog_options.buffer);
                                self.dialog_options.buffer = "".to_string();
                            }
                        });
                    });
            }
        }
    }
}

/**
    Statistics block
**/

struct StatisticOptions {
    scroll_offset_x: f32,
    scroll_offset_y: f32,
    from: DateTime<Local>,
    to: DateTime<Local>,
}

impl Default for StatisticOptions {
    fn default() -> Self {
        let s1 = DateTime::<Local>::from(SystemTime::now());
        let from = Local
            .with_ymd_and_hms(s1.year(), s1.month(), s1.day() - 1, 0, 0, 0)
            .unwrap();
        let to = Local
            .with_ymd_and_hms(s1.year(), s1.month(), s1.day() + 1, 23, 59, 59)
            .unwrap();

        StatisticOptions {
            scroll_offset_x: 0.,
            scroll_offset_y: 0.,
            from,
            to,
        }
    }
}

impl Frontend {
    fn build_statistic(&mut self, ui: &mut Ui) {
        ui.horizontal_top(|ui| {
            ui.with_layout(Layout::right_to_left(Align::Min), |ui| {
                ui.horizontal(|ui| {
                    ui.add_space(5.);
                    egui::ComboBox::from_label("")
                        .selected_text(format!("{:?}", self.current_display_mode))
                        .show_ui(ui, |ui| {
                            ui.selectable_value(
                                &mut self.current_display_mode,
                                DisplayMode::Time,
                                "Time",
                            );
                            ui.selectable_value(
                                &mut self.current_display_mode,
                                DisplayMode::Statistic,
                                "Statistic",
                            );
                            ui.selectable_value(
                                &mut self.current_display_mode,
                                DisplayMode::Todo,
                                "Todo",
                            );
                        });
                });
            });
        });

        let records = self
            .backend
            .history
            .get_ordered_records((self.statistic_options.from, self.statistic_options.to));
        let style = ui.style().clone();
        let mut new_style = (*style).clone();
        new_style.spacing.item_spacing = Vec2::new(0., 0.);

        ui.set_style(new_style);

        ui.vertical(|ui| {
            ui.push_id(3, |ui| {
                ui.set_min_height(400.0);
                ui.set_max_height(400.0);
                ScrollArea::vertical().show(ui, |_ui| {});
            });
        });

        ui.separator();

        ui.push_id(7, |ui| {
            let time_block = ScrollArea::horizontal()
                .scroll_bar_visibility(ScrollBarVisibility::AlwaysHidden)
                .scroll_offset(Vec2::new(self.statistic_options.scroll_offset_x, 0.));

            time_block.show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.add_space(50.);

                    for i in 0..=24 {
                        let c = ui.label(
                            RichText::new(if i < 10 {
                                format!("0{i}")
                            } else {
                                format!("{i}")
                            })
                            .font(FontId::proportional(12.0)),
                        );
                        if i < 24 {
                            ui.add_space(60.0 - c.rect.size().x)
                        }
                    }
                });

                ui.horizontal(|ui| {
                    ui.add_space(50.);
                    let (rect, _response) =
                        ui.allocate_exact_size(egui::vec2(2., 10.0), egui::Sense::click());

                    let mut ident = rect.size().x;

                    ui.painter().add(RectShape {
                        rect,
                        rounding: Rounding::same(1.0),
                        fill: Color32::LIGHT_GRAY,
                        stroke: Default::default(),
                    });

                    for _ in 0..24 {
                        ui.add_space(60.0 - ident);

                        let (rect, _response) =
                            ui.allocate_exact_size(egui::vec2(2., 10.0), egui::Sense::click());

                        ident = rect.size().x;

                        ui.painter().add(RectShape {
                            rect,
                            rounding: Rounding::same(1.0),
                            fill: Color32::LIGHT_GRAY,
                            stroke: Default::default(),
                        });
                    }
                });
            });
        });

        ui.horizontal(|ui| {
            ui.set_min_height(320.);
            ui.set_max_height(320.);

            ui.push_id(5, |ui| {
                ui.set_min_width(50.);
                ui.set_max_width(50.);

                let date_block = ScrollArea::vertical()
                    .scroll_bar_visibility(ScrollBarVisibility::AlwaysHidden)
                    .scroll_offset(Vec2::new(0., self.statistic_options.scroll_offset_y));

                date_block.show(ui, |ui| {
                    ui.vertical(|ui| {
                        let mut first_year_first_month = true;

                        for year in
                            self.statistic_options.from.year()..=self.statistic_options.to.year()
                        {
                            //TODO: fix month
                            for month in self.statistic_options.from.month()
                                ..=self.statistic_options.to.month()
                            {
                                let from = if first_year_first_month {
                                    first_year_first_month = false;

                                    self.statistic_options.from.day()
                                } else {
                                    1
                                };

                                let to = if year == self.statistic_options.to.year()
                                    && month == self.statistic_options.to.month()
                                {
                                    self.statistic_options.to.day()
                                } else {
                                    get_days_from_month(year, month)
                                };

                                for day in from..=to {
                                    ui.horizontal(|ui| {
                                        ui.set_min_height(25.);
                                        ui.set_max_height(25.);

                                        ui.label(
                                            RichText::new(format!(
                                                "{}/{}",
                                                format_number(day),
                                                format_number(month)
                                            ))
                                            .font(FontId::proportional(13.0)),
                                        );
                                    });
                                }
                            }
                        }
                    });
                });
            });

            ui.push_id(6, |ui| {
                let bars_block = ScrollArea::both().show(ui, |ui| {
                    ui.set_min_size(Vec2::new(
                        60.0 * 24.0,
                        315.0f32.max(
                            25. * calendar_days_count(
                                self.statistic_options.from,
                                self.statistic_options.to,
                            ) as f32,
                        ),
                    ));

                    ui.vertical(|ui| {
                        let mut first_year_first_month = true;

                        let mut i = 0usize;

                        for year in
                            self.statistic_options.from.year()..=self.statistic_options.to.year()
                        {
                            for month in self.statistic_options.from.month()
                                ..=self.statistic_options.to.month()
                            {
                                let from = if first_year_first_month {
                                    first_year_first_month = false;

                                    self.statistic_options.from.day()
                                } else {
                                    1
                                };

                                let to = if year == self.statistic_options.to.year()
                                    && month == self.statistic_options.to.month()
                                {
                                    self.statistic_options.to.day()
                                } else {
                                    get_days_from_month(year, month)
                                };

                                for _ in from..=to {
                                    let mut previous_ending = None;
                                    let mut space_added = false;
                                    let mut length = 0_f32;

                                    ui.horizontal(|ui| {
                                        ui.set_min_height(25.);
                                        ui.set_max_height(25.);

                                        for record in records.get(i).unwrap() {
                                            if !space_added {
                                                let d = record.start_date.hour() as f32 * 60.0
                                                    + record.start_date.minute() as f32;
                                                ui.add_space(d);
                                                length += d;

                                                space_added = true;
                                            }

                                            let duration = record.get_duration();

                                            if duration.num_minutes() <= 0 {
                                                continue;
                                            }

                                            if let Some(prev) = previous_ending {
                                                let dur = record
                                                    .start_date
                                                    .signed_duration_since(prev)
                                                    .num_minutes();

                                                if dur > 0 {
                                                    ui.add_space(dur as f32);
                                                    length += dur as f32;
                                                }
                                            }

                                            let desired_size = egui::vec2(
                                                record.get_duration().num_minutes() as f32,
                                                15.0,
                                            );

                                            length += desired_size.x;

                                            let (rect, response) = ui.allocate_exact_size(
                                                desired_size,
                                                egui::Sense::click(),
                                            );

                                            let project = self
                                                .backend
                                                .projects
                                                .get(&record.project_id)
                                                .unwrap()
                                                .lock()
                                                .unwrap();
                                            let subject = project
                                                .subjects
                                                .get(&record.subject_id)
                                                .unwrap()
                                                .lock()
                                                .unwrap();

                                            response.on_hover_text(format!(
                                                "{} : {}",
                                                project.name, subject.name
                                            ));

                                            ui.painter().add(RectShape {
                                                rect,
                                                rounding: Rounding::same(4.0),
                                                fill: Color32::from_rgb(
                                                    project.color.0,
                                                    project.color.1,
                                                    project.color.2,
                                                ),
                                                stroke: Default::default(),
                                            });

                                            previous_ending = Some(record.end_date);
                                        }

                                        if length < 60.0 * 24.0 {
                                            ui.add_space(60.0 * 24.0 - length);
                                        }
                                    });

                                    i += 1;
                                }
                            }
                        }
                    });
                });

                self.statistic_options.scroll_offset_x = bars_block.state.offset.x;
                self.statistic_options.scroll_offset_y = bars_block.state.offset.y;
            });
        });

        ui.set_style(style);
    }
}

/**
    Maximized Time Tracker block
**/

#[derive(Default)]
struct TimeTrackerOptions {
    current_label: String,
}

impl Frontend {
    fn time_tracker_build(&mut self, ui: &mut Ui) {
        ui.horizontal_top(|ui| {
            ui.label(format!(
                "Current work: {}",
                self.time_tracker_options.current_label
            ));

            let mut visuals = ui.ctx().style().visuals.clone();

            ui.with_layout(Layout::right_to_left(Align::Min), |ui| {
                ui.horizontal(|ui| {
                    ui.selectable_value(&mut visuals, Visuals::light(), "☀");
                    ui.selectable_value(&mut visuals, Visuals::dark(), "🌙");

                    if self.backend.current_subject.is_some() && ui.button("⬇").clicked() {
                        self.current_display_mode = DisplayMode::Minimal;
                    }

                    egui::ComboBox::from_label("")
                        .selected_text(format!("{:?}", self.current_display_mode))
                        .show_ui(ui, |ui| {
                            ui.selectable_value(
                                &mut self.current_display_mode,
                                DisplayMode::Time,
                                "Time",
                            );
                            ui.selectable_value(
                                &mut self.current_display_mode,
                                DisplayMode::Statistic,
                                "Statistic",
                            );
                            ui.selectable_value(
                                &mut self.current_display_mode,
                                DisplayMode::Todo,
                                "Todo",
                            );
                        });
                });
            });

            ui.ctx().set_visuals(visuals);
        });

        ui.horizontal(|ui| {
            ui.set_min_height(55.0);
            ui.set_max_height(55.0);

            if self.backend.current_subject.is_some() {
                match self.backend.working_mode {
                    WorkingMode::Idle => {
                        if ui.button("START").clicked() {
                            self.time_tracker_start_subject()
                        }
                    }
                    WorkingMode::InProgress(_) => {
                        if ui.button("PAUSE").clicked() {
                            self.time_tracker_stop_subject(false);
                        }
                    }
                }
                ui.label(format_duration(self.backend.current_session_duration));
            }
        });

        ui.separator();

        ui.horizontal(|ui| {
            ui.set_min_height(290.0);
            ui.set_max_height(290.0);

            ui.push_id(1, |ui| {
                ScrollArea::vertical().show(ui, |ui| {
                    self.time_tracker_build_projects(ui);
                });
            });

            ui.separator();

            ui.push_id(2, |ui| {
                ScrollArea::vertical().show(ui, |ui| {
                    self.time_tracker_build_subjects(ui);
                });
            });
        });
    }

    fn time_tracker_build_projects(&mut self, ui: &mut Ui) {
        ui.set_min_width(400.0);
        ui.set_max_width(400.0);

        let current_id = if let Some(cur_project) = &self.backend.current_project {
            cur_project.lock().unwrap().id
        } else {
            Uuid::new_v4()
        };

        ui.vertical(|ui| {
            let projects = self.backend.projects.clone();
            for (_, project) in projects {
                let r_project = project.lock().unwrap();

                if r_project.is_deleted {
                    continue;
                }

                ui.horizontal(|ui| {
                    let mut text = RichText::new(&r_project.name);

                    if r_project.id == current_id {
                        text = text.strong();
                    }

                    if ui.button(text).clicked() {
                        self.backend.set_current_project(Some(project.clone()));
                    }

                    ui.label(format_duration(r_project.get_time()));
                });

                ui.add_space(5.0);
            }

            if ui.button("   +   ").clicked() {
                self.dialog_options.current_dialog = CurrentDialog::AddProject;
            }
        });
    }

    fn time_tracker_build_subjects(&mut self, ui: &mut Ui) {
        ui.set_min_width(400.0);
        ui.set_max_width(400.0);

        let Some(current_project) = self.backend.current_project.clone() else {
            return;
        };

        let current_id = if let Some(cur_subject) = &self.backend.current_subject {
            cur_subject.lock().unwrap().id
        } else {
            Uuid::new_v4()
        };

        ui.vertical(|ui| {
            for subject in current_project.lock().unwrap().subjects.values() {
                let r_subject = subject.lock().unwrap();

                if r_subject.is_deleted {
                    continue;
                }

                ui.horizontal(|ui| {
                    let mut text = RichText::new(&r_subject.name);

                    if r_subject.id == current_id {
                        text = text.strong();
                    }

                    if ui.button(text).clicked() {
                        if current_id != r_subject.id {
                            self.time_tracker_stop_subject(true);
                        }
                        self.backend.set_current_subject(Some(subject.clone()));
                    }

                    ui.label(format_duration(r_subject.duration));
                });

                ui.add_space(5.0);
            }

            if ui.button("   +   ").clicked() {
                self.dialog_options.current_dialog = CurrentDialog::AddSubject;
            }
        });
    }

    fn time_tracker_start_subject(&mut self) {
        self.backend.start_subject();
        self.time_tracker_options.current_label = self.backend.get_current_work_name();
    }

    fn time_tracker_stop_subject(&mut self, force: bool) {
        self.backend.stop_subject(force);
        self.time_tracker_options.current_label = "".to_string();
    }
}

/**
    Minimized Time Tracker block
**/

#[derive(Default)]
struct MinimalTrackerOptions {}

impl Frontend {
    fn minimal_time_tracker_build(&mut self, ui: &mut Ui) {
        ui.vertical_centered(|ui| {
            ui.horizontal(|ui| {
                match self.backend.working_mode {
                    WorkingMode::Idle => {
                        if ui.button("START").clicked() {
                            self.time_tracker_start_subject()
                        }
                    }
                    WorkingMode::InProgress(_) => {
                        if ui.button("PAUSE").clicked() {
                            self.time_tracker_stop_subject(false);
                        }
                    }
                }

                if ui.button("⬆").clicked() {
                    self.current_display_mode = DisplayMode::Time;
                }
            });
            ui.label(format_duration(self.backend.current_session_duration));
        });
    }
}

/**
    TO DO block
**/

#[derive(Default)]
struct TodoOptions {}

impl Frontend {
    fn todo_build(&mut self, ui: &mut Ui) {
        ui.horizontal_top(|ui| {
            ui.with_layout(Layout::right_to_left(Align::Min), |ui| {
                ui.horizontal(|ui| {
                    egui::ComboBox::from_label("")
                        .selected_text(format!("{:?}", self.current_display_mode))
                        .show_ui(ui, |ui| {
                            ui.selectable_value(
                                &mut self.current_display_mode,
                                DisplayMode::Time,
                                "Time",
                            );
                            ui.selectable_value(
                                &mut self.current_display_mode,
                                DisplayMode::Statistic,
                                "Statistic",
                            );
                            ui.selectable_value(
                                &mut self.current_display_mode,
                                DisplayMode::Todo,
                                "Todo",
                            );
                        });
                });
            });
        });

        ui.separator();

        ui.horizontal(|ui| {
            ui.set_min_height(353.0);
            ui.set_max_height(353.0);

            ui.push_id(1, |ui| {
                ScrollArea::vertical().show(ui, |ui| {
                    self.todo_build_projects(ui);
                });
            });

            ui.separator();

            ui.push_id(2, |ui| {
                ScrollArea::vertical().show(ui, |ui| {
                    self.todo_build_subjects(ui);
                });
            });
        });
    }

    fn todo_build_projects(&mut self, ui: &mut Ui) {
        ui.set_min_width(400.0);
        ui.set_max_width(400.0);

        let current_id = if let Some(cur_project) = &self.backend.current_todo_project {
            cur_project.lock().unwrap().id
        } else {
            Uuid::new_v4()
        };

        ui.vertical(|ui| {
            let projects = self.backend.todos.clone();
            for (_, project) in projects {
                let r_project = project.lock().unwrap();

                if r_project.is_deleted {
                    continue;
                }

                ui.horizontal(|ui| {
                    let mut text = RichText::new(&r_project.name);

                    if r_project.id == current_id {
                        text = text.strong();
                    }

                    if ui.button(text).clicked() {
                        self.backend.set_current_todo_project(Some(project.clone()));
                    }
                });

                ui.add_space(5.0);
            }

            if ui.button("   +   ").clicked() {
                self.dialog_options.current_dialog = CurrentDialog::AddTodoProject;
            }
        });
    }

    fn todo_build_subjects(&mut self, ui: &mut Ui) {
        ui.set_min_width(400.0);
        ui.set_max_width(400.0);

        let Some(current_project) = self.backend.current_todo_project.clone() else {
            return;
        };

        ui.vertical(|ui| {
            for subject in current_project.lock().unwrap().subjects.values() {
                let text;
                let mut is_done;
                {
                    let r_subject = subject.lock().unwrap();

                    if r_subject.is_deleted {
                        continue;
                    }
                    text = RichText::new(&r_subject.name);
                    is_done = r_subject.is_done;
                }

                ui.horizontal(|ui| {
                    if ui.checkbox(&mut is_done, text).clicked() {
                        subject.lock().unwrap().toggle();
                        self.backend.mark_dirty();
                    };
                });

                ui.add_space(5.0);
            }

            if ui.button("   +   ").clicked() {
                self.dialog_options.current_dialog = CurrentDialog::AddTodoSubject;
            }
        });
    }
}
