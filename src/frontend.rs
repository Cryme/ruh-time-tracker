use crate::backend::{Backend, WorkingMode};
use crate::custom_window_frame;
use crate::util::{
    calendar_days_count, format_chrono_duration, format_duration, format_number,
    get_days_from_month,
};
use std::collections::HashMap;
use std::ops::{Add, Sub};


use chrono::{DateTime, Datelike, Days, Local, LocalResult, Month, TimeZone, Timelike};
use eframe::egui;
use eframe::egui::scroll_area::ScrollBarVisibility;
use eframe::egui::{
    Align, Color32, FontId, Key, Label, Layout, RichText, Rounding, ScrollArea, TextEdit, Ui, Vec2,
    Visuals,
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

#[derive(Default, PartialEq)]
enum CurrentDialog {
    #[default]
    None,
    AddProject,
    AddSubProject,
    AddSubject,
    AddTodoProject,
    AddTodoSubProject,
    AddTodoSubject,
}

#[derive(Default)]
pub struct Frontend {
    backend: Backend,

    current_display_mode: DisplayMode,

    hotkeys_blocked: bool,

    dialog_options: DialogOptions,
    time_tracker_options: TimeTrackerOptions,
    minimal_time_tracker_options: MinimalTrackerOptions,
    todo_options: TodoOptions,
    statistic_options: StatisticOptions,
}

impl Frontend {
    fn set_display_mode(&mut self, mode: DisplayMode) {
        if mode == DisplayMode::Minimal {
            self.minimal_time_tracker_options.prev_mode = self.current_display_mode;
        }

        self.current_display_mode = mode;
    }

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

        if !self.hotkeys_blocked && self.dialog_options.current_dialog == CurrentDialog::None {
            if ctx.input(|i| i.key_pressed(Key::Q)) {
                self.set_display_mode(DisplayMode::Time);
            } else if ctx.input(|i| i.key_pressed(Key::W)) {
                self.set_display_mode(DisplayMode::Statistic);
            } else if ctx.input(|i| i.key_pressed(Key::E)) {
                self.set_display_mode(DisplayMode::Todo);
            } else if ctx.input(|i| i.key_pressed(Key::D)) {
                self.set_display_mode(DisplayMode::Minimal);
            }
        }
    }

    fn clear_color(&self, _visuals: &Visuals) -> [f32; 4] {
        egui::Rgba::TRANSPARENT.to_array() // Make sure we don't paint anything behind the rounded corners
    }
}

/**
Menu block
 **/

#[derive(Default)]
struct MenuOptions {}

impl Frontend {
    fn build_menu(&mut self, ui: &mut Ui) {
        match self.current_display_mode {
            DisplayMode::Todo | DisplayMode::Statistic | DisplayMode::Time => {
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
            }

            DisplayMode::Minimal => {
                if ui.button("⬆").clicked() {
                    self.set_display_mode(self.minimal_time_tracker_options.prev_mode);
                }
            }
        }
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

            CurrentDialog::AddSubProject => {
                egui::Window::new("Add Sub Project")
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
                                self.backend.add_sub_project(&self.dialog_options.buffer);
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

            CurrentDialog::AddTodoSubProject => {
                egui::Window::new("Add Sub Project")
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
                                self.backend
                                    .add_todo_sub_project(&self.dialog_options.buffer);
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
    label_from: SimpleDate,
    label_to: SimpleDate,
    from: DateTime<Local>,
    to: DateTime<Local>,
    current_project_id: Option<Uuid>,
    current_sub_project_id: Option<Uuid>,
    invalid_from: bool,
    invalid_to: bool,
}

struct SimpleDate {
    year: String,
    month: Month,
    day: String,
}

impl TryInto<DateTime<Local>> for &SimpleDate {
    type Error = ();

    fn try_into(self) -> Result<DateTime<Local>, Self::Error> {
        let Ok(year) = self.year.parse::<i32>() else {
            return Err(());
        };
        let Ok(day) = self.day.parse::<u32>() else {
            return Err(());
        };

        let LocalResult::Single(res) =
            Local.with_ymd_and_hms(year, self.month.number_from_month(), day, 0, 0, 0)
        else {
            return Err(());
        };

        Ok(res)
    }
}

impl StatisticOptions {
    fn update_from_labels(&mut self) {
        let from: Result<DateTime<Local>, ()> = (&self.label_from).try_into();
        let to: Result<DateTime<Local>, ()> = (&self.label_to).try_into();

        self.invalid_from = from.is_err();
        self.invalid_to = to.is_err();

        if let Ok(f) = from {
            if let Ok(t) = to {
                self.from = f;
                self.to = t
                    .checked_add_days(Days::new(1))
                    .unwrap()
                    .sub(chrono::Duration::milliseconds(100));
            }
        }
    }
}

impl Default for StatisticOptions {
    fn default() -> Self {
        let s1 = DateTime::<Local>::from(SystemTime::now());
        let days = get_days_from_month(s1.year(), s1.month());

        let from;
        {
            let mut day = s1.day();
            let mut month = s1.month();
            let mut year = s1.year();

            if days == 1 {
                if month == 1 {
                    year -= 1;
                    month = 12;
                } else {
                    month -= 1;
                }
                day = get_days_from_month(year, month);
            }

            from = Local.with_ymd_and_hms(year, month, day, 0, 0, 0).unwrap();
        }

        let to;
        {
            let mut day = s1.day();
            let mut month = s1.month();
            let mut year = s1.year();

            if days == s1.day() {
                if month == 12 {
                    year += 1;
                    month = 1;
                    day = 1;
                } else {
                    month += 1;
                    day = 1;
                }
            }

            to = Local
                .with_ymd_and_hms(year, month, day, 23, 59, 59)
                .unwrap();
        }

        StatisticOptions {
            scroll_offset_x: 0.,
            scroll_offset_y: 0.,
            label_from: SimpleDate {
                year: from.year().to_string(),
                month: Month::try_from(from.month() as u8).unwrap(),
                day: from.day().to_string(),
            },
            label_to: SimpleDate {
                year: to.year().to_string(),
                month: Month::try_from(to.month() as u8).unwrap(),
                day: to.day().to_string(),
            },
            invalid_from: false,
            invalid_to: false,
            from,
            to,
            current_project_id: None,
            current_sub_project_id: None,
        }
    }
}

impl Frontend {
    fn build_statistic(&mut self, ui: &mut Ui) {
        self.build_menu(ui);

        let style = ui.style().clone();
        let mut new_style = (*style).clone();
        new_style.spacing.item_spacing = Vec2::new(0., 0.);

        ui.set_style(new_style);

        ui.horizontal_top(|ui| {
            ui.add_space(400.);

            ui.set_max_height(30.);
            {
                let y = ui.add_sized(
                    (50., 15.),
                    TextEdit::singleline(&mut self.statistic_options.label_from.year),
                );

                if y.gained_focus() {
                    self.hotkeys_blocked = true;
                }

                if y.lost_focus() {
                    self.hotkeys_blocked = false;
                    self.statistic_options.update_from_labels();
                }

                ui.add_space(2.);

                ui.push_id(9, |ui| {
                    if egui::ComboBox::from_label("")
                        .selected_text(self.statistic_options.label_from.month.name())
                        .show_ui(ui, |ui| {
                            for month in 1..=12 {
                                let m = Month::try_from(month).unwrap();
                                ui.selectable_value(
                                    &mut self.statistic_options.label_from.month,
                                    m,
                                    m.name(),
                                );
                            }
                        })
                        .response
                        .changed()
                    {
                        self.statistic_options.update_from_labels();
                    };
                });

                ui.add_space(2.);

                let d = ui.add_sized(
                    (30., 15.),
                    TextEdit::singleline(&mut self.statistic_options.label_from.day),
                );

                if d.gained_focus() {
                    self.hotkeys_blocked = true;
                }

                if d.lost_focus() {
                    self.hotkeys_blocked = false;
                    self.statistic_options.update_from_labels();
                }
            }

            ui.add_space(5.);
            ui.add_sized((5., 15.), Label::new(":"));
            ui.add_space(5.);

            {
                let y = ui.add_sized(
                    (50., 15.),
                    TextEdit::singleline(&mut self.statistic_options.label_to.year),
                );

                if y.gained_focus() {
                    self.hotkeys_blocked = true;
                }

                if y.lost_focus() {
                    self.hotkeys_blocked = false;
                    self.statistic_options.update_from_labels();
                }

                ui.add_space(2.);

                ui.push_id(10, |ui| {
                    if egui::ComboBox::from_label("")
                        .selected_text(self.statistic_options.label_to.month.name())
                        .show_ui(ui, |ui| {
                            for month in 1..=12 {
                                let m = Month::try_from(month).unwrap();
                                ui.selectable_value(
                                    &mut self.statistic_options.label_to.month,
                                    m,
                                    m.name(),
                                );
                            }
                        })
                        .response
                        .changed()
                    {
                        self.statistic_options.update_from_labels();
                    };
                });

                ui.add_space(2.);

                let d = ui.add_sized(
                    (30., 15.),
                    TextEdit::singleline(&mut self.statistic_options.label_to.day),
                );

                if d.gained_focus() {
                    self.hotkeys_blocked = true;
                }

                if d.lost_focus() {
                    self.hotkeys_blocked = false;
                    self.statistic_options.update_from_labels();
                }
            }
        });

        ui.add_space(10.);

        let records = self
            .backend
            .history
            .get_ordered_records((self.statistic_options.from, self.statistic_options.to));

        ui.vertical(|ui| {
            ui.push_id(3, |ui| {
                ui.set_min_height(400.0);
                ui.set_max_height(400.0);
                ScrollArea::vertical().show(ui, |ui| {
                    struct Summary {
                        title: String,
                        duration: chrono::Duration,
                    }

                    let mut projects_summary: HashMap<Uuid, Summary> = HashMap::new();
                    let mut sub_projects_summary: HashMap<Uuid, Summary> = HashMap::new();
                    let mut subjects_summary: HashMap<Uuid, Summary> = HashMap::new();

                    for record in self
                        .backend
                        .history
                        .get_records((self.statistic_options.from, self.statistic_options.to))
                    {
                        if let Some(v) = projects_summary.get_mut(&record.project_id) {
                            v.duration = v.duration.add(record.get_duration());
                        } else {
                            projects_summary.insert(
                                record.project_id,
                                Summary {
                                    title: self
                                        .backend
                                        .projects
                                        .inner
                                        .get(&record.project_id)
                                        .unwrap()
                                        .name
                                        .clone(),
                                    duration: record.get_duration(),
                                },
                            );
                        }

                        if let Some(id) = self.statistic_options.current_project_id {
                            if id == record.project_id {
                                if let Some(v) =
                                    sub_projects_summary.get_mut(&record.sub_project_id)
                                {
                                    v.duration = v.duration.add(record.get_duration());
                                } else {
                                    sub_projects_summary.insert(
                                        record.sub_project_id,
                                        Summary {
                                            title: self
                                                .backend
                                                .projects
                                                .inner
                                                .get(&record.project_id)
                                                .unwrap_or_else(|| panic!("bad project id {}",
                                                    record.subject_id))
                                                .inner
                                                .get(&record.sub_project_id)
                                                .unwrap_or_else(|| panic!("bad sub-project id {}",
                                                    record.subject_id))
                                                .name
                                                .clone(),
                                            duration: record.get_duration(),
                                        },
                                    );
                                }
                            }
                        }

                        if let Some(id) = self.statistic_options.current_sub_project_id {
                            if id == record.sub_project_id {
                                if let Some(v) = subjects_summary.get_mut(&record.subject_id) {
                                    v.duration = v.duration.add(record.get_duration());
                                } else {
                                    subjects_summary.insert(
                                        record.subject_id,
                                        Summary {
                                            title: self
                                                .backend
                                                .projects
                                                .inner
                                                .get(&record.project_id)
                                                .unwrap_or_else(|| panic!("bad project id {}",
                                                    record.subject_id))
                                                .inner
                                                .get(&record.sub_project_id)
                                                .unwrap_or_else(|| panic!("bad sub-project id {}",
                                                    record.subject_id))
                                                .inner
                                                .get(&record.subject_id)
                                                .unwrap_or_else(|| panic!("bad subject id {}",
                                                    record.subject_id))
                                                .lock()
                                                .unwrap()
                                                .name
                                                .clone(),
                                            duration: record.get_duration(),
                                        },
                                    );
                                }
                            }
                        }
                    }

                    ui.horizontal(|ui| {
                        ui.vertical(|ui| {
                            let mut c: Vec<(&Uuid, &Summary)> = projects_summary.iter().collect();
                            c.sort_by(|a, b| a.1.duration.cmp(&b.1.duration));

                            for v in c {
                                let mut text = RichText::new(&v.1.title);

                                if self.statistic_options.current_project_id == Some(*v.0) {
                                    text = text.strong();
                                }

                                ui.horizontal(|ui| {
                                    if ui.button(text).clicked() {
                                        self.statistic_options.current_project_id = Some(*v.0);
                                        self.statistic_options.current_sub_project_id = None;
                                    }

                                    ui.label(format!(
                                        " - {}",
                                        format_chrono_duration(v.1.duration)
                                    ));
                                });
                            }
                        });

                        ui.add_space(215.);

                        ui.vertical(|ui| {
                            let mut c: Vec<(&Uuid, &Summary)> =
                                sub_projects_summary.iter().collect();
                            c.sort_by(|a, b| a.1.duration.cmp(&b.1.duration));

                            for v in c {
                                let mut text = RichText::new(&v.1.title);

                                if self.statistic_options.current_sub_project_id == Some(*v.0) {
                                    text = text.strong();
                                }

                                ui.horizontal(|ui| {
                                    if ui.button(text).clicked() {
                                        self.statistic_options.current_sub_project_id = Some(*v.0);
                                    }

                                    ui.label(format!(
                                        " - {}",
                                        format_chrono_duration(v.1.duration)
                                    ));
                                });
                            }
                        });

                        ui.add_space(215.);

                        ui.vertical(|ui| {
                            let mut c: Vec<&Summary> = subjects_summary.values().collect();
                            c.sort_by(|a, b| a.duration.cmp(&b.duration));

                            for v in c {
                                ui.label(format!(
                                    "{} - {}",
                                    v.title,
                                    format_chrono_duration(v.duration)
                                ));
                                ui.add_space(4.);
                            }
                        });
                    });
                });
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
                                                .inner
                                                .get(&record.project_id)
                                                .unwrap();

                                            let sub_project =
                                                project.inner.get(&record.sub_project_id).unwrap();

                                            let subject = sub_project
                                                .inner
                                                .get(&record.subject_id)
                                                .unwrap()
                                                .lock()
                                                .unwrap();

                                            response.on_hover_text(format!(
                                                "{}/{}/{}",
                                                project.name, sub_project.name, subject.name
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

            self.build_menu(ui);
        });

        ui.horizontal(|ui| {
            ui.set_min_height(55.0);
            ui.set_max_height(55.0);

            if self.backend.get_current_subject().is_some() {
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
                ScrollArea::both().show(ui, |ui| {
                    self.time_tracker_build_projects(ui);
                });
            });

            ui.separator();

            ui.push_id(2, |ui| {
                ScrollArea::both().show(ui, |ui| {
                    self.time_tracker_build_sub_projects(ui);
                });
            });

            ui.separator();

            ui.push_id(3, |ui| {
                ScrollArea::both().show(ui, |ui| {
                    self.time_tracker_build_subjects(ui);
                });
            });
        });
    }

    fn time_tracker_build_sub_projects(&mut self, ui: &mut Ui) {
        ui.set_min_width(300.0);
        ui.set_max_width(300.0);

        let Some(current_project) = self.backend.get_current_project() else {
            return;
        };

        let current_id = if let Some(cur_project) = self.backend.get_current_sub_project() {
            cur_project.id
        } else {
            Uuid::new_v4()
        };

        let c = current_project.get_inner_sorted(|a, b| a.created_at.cmp(&b.created_at));

        ui.vertical(|ui| {
            for sub_project in c {
                if sub_project.is_deleted {
                    continue;
                }

                ui.horizontal(|ui| {
                    let mut text = RichText::new(&sub_project.name);

                    if sub_project.id == current_id {
                        text = text.strong();
                    }

                    if ui.button(text).clicked() {
                        self.backend.set_current_sub_project(Some(sub_project.id));
                    }

                    ui.label(format_duration(
                        self.backend.get_sub_project_time(&sub_project.id).unwrap(),
                    ));
                });

                ui.add_space(5.0);
            }

            if ui.button("   +   ").clicked() {
                self.dialog_options.current_dialog = CurrentDialog::AddSubProject;
            }
        });
    }

    fn time_tracker_build_projects(&mut self, ui: &mut Ui) {
        ui.set_min_width(300.0);
        ui.set_max_width(300.0);

        let current_id = if let Some(cur_project) = self.backend.get_current_project() {
            cur_project.id
        } else {
            Uuid::new_v4()
        };

        let c = self
            .backend
            .projects
            .get_inner_sorted(|a, b| a.created_at.cmp(&b.created_at));

        ui.vertical(|ui| {
            for project in c {
                if project.is_deleted {
                    continue;
                }

                ui.horizontal(|ui| {
                    let mut text = RichText::new(&project.name);

                    if project.id == current_id {
                        text = text.strong();
                    }

                    if ui.button(text).clicked() {
                        self.backend.set_current_project(Some(project.id));
                    }

                    ui.label(format_duration(
                        self.backend.get_project_time(&project.id).unwrap(),
                    ));
                });

                ui.add_space(5.0);
            }

            if ui.button("   +   ").clicked() {
                self.dialog_options.current_dialog = CurrentDialog::AddProject;
            }
        });
    }

    fn time_tracker_build_subjects(&mut self, ui: &mut Ui) {
        ui.set_min_width(300.0);
        ui.set_max_width(300.0);

        let Some(current_sub_project) = self.backend.get_current_sub_project() else {
            return;
        };

        let current_id = if let Some(cur_subject) = self.backend.get_current_subject() {
            cur_subject.lock().unwrap().id
        } else {
            Uuid::new_v4()
        };

        let c = current_sub_project.get_inner_sorted(|a, b| {
            a.lock()
                .unwrap()
                .created_at
                .cmp(&b.lock().unwrap().created_at)
        });

        ui.vertical(|ui| {
            for subject in c {
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
                        self.backend.set_current_subject(Some(r_subject.id));
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
struct MinimalTrackerOptions {
    prev_mode: DisplayMode,
}

impl Frontend {
    fn minimal_time_tracker_build(&mut self, ui: &mut Ui) {
        let current_subject = self.backend.get_current_subject();
        ui.vertical_centered(|ui| {
            ui.horizontal(|ui| {
                if current_subject.is_some() {
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
                }

                self.build_menu(ui);
            });

            if current_subject.is_some() {
                ui.label(format_duration(self.backend.current_session_duration));
            }
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
        self.build_menu(ui);

        ui.separator();

        ui.horizontal(|ui| {
            ui.set_min_height(353.0);
            ui.set_max_height(353.0);

            ui.push_id(1, |ui| {
                ScrollArea::both().show(ui, |ui| {
                    self.todo_build_projects(ui);
                });
            });

            ui.separator();

            ui.push_id(2, |ui| {
                ScrollArea::both().show(ui, |ui| {
                    self.todo_build_sub_projects(ui);
                });
            });

            ui.separator();

            ui.push_id(3, |ui| {
                ScrollArea::both().show(ui, |ui| {
                    self.todo_build_subjects(ui);
                });
            });
        });
    }

    fn todo_build_projects(&mut self, ui: &mut Ui) {
        ui.set_min_width(300.0);
        ui.set_max_width(300.0);

        let current_id = if let Some(cur_project) = self.backend.get_current_todo_project() {
            cur_project.id
        } else {
            Uuid::new_v4()
        };

        let c = self
            .backend
            .todos
            .get_inner_sorted(|a, b| a.created_at.cmp(&b.created_at));

        ui.vertical(|ui| {
            for project in c {
                if project.is_deleted {
                    continue;
                }

                ui.horizontal(|ui| {
                    let mut text = RichText::new(&project.name);

                    if project.id == current_id {
                        text = text.strong();
                    }

                    if ui.button(text).clicked() {
                        self.backend.set_current_todo_project(Some(project.id));
                    }
                });

                ui.add_space(5.0);
            }

            if ui.button("   +   ").clicked() {
                self.dialog_options.current_dialog = CurrentDialog::AddTodoProject;
            }
        });
    }

    fn todo_build_sub_projects(&mut self, ui: &mut Ui) {
        ui.set_min_width(300.0);
        ui.set_max_width(300.0);

        let Some(current_project) = self.backend.get_current_todo_project() else {
            return;
        };

        let current_id = if let Some(cur_project) = self.backend.get_current_todo_sub_project() {
            cur_project.id
        } else {
            Uuid::new_v4()
        };

        let c = current_project.get_inner_sorted(|a, b| a.created_at.cmp(&b.created_at));

        ui.vertical(|ui| {
            for sub_project in c {
                if sub_project.is_deleted {
                    continue;
                }

                ui.horizontal(|ui| {
                    let mut text = RichText::new(&sub_project.name);

                    if sub_project.id == current_id {
                        text = text.strong();
                    }

                    if ui.button(text).clicked() {
                        self.backend
                            .set_current_todo_sub_project(Some(sub_project.id));
                    }
                });

                ui.add_space(5.0);
            }

            if ui.button("   +   ").clicked() {
                self.dialog_options.current_dialog = CurrentDialog::AddTodoSubProject;
            }
        });
    }

    fn todo_build_subjects(&mut self, ui: &mut Ui) {
        ui.set_min_width(300.0);
        ui.set_max_width(300.0);

        let Some(current_todo_sub_project) = self.backend.get_current_todo_sub_project() else {
            return;
        };

        let c = current_todo_sub_project.get_inner_sorted(|a, b| {
            a.lock()
                .unwrap()
                .created_at
                .cmp(&b.lock().unwrap().created_at)
        });

        ui.vertical(|ui| {
            for subject in c {
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
                        self.backend.dirty();
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
