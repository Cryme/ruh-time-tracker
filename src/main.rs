//! Show a custom window frame instead of the default OS window chrome decorations.

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

mod backend;
mod history;
mod util;

use crate::backend::{Backend, WorkingMode};
use crate::util::format_duration;
use eframe::egui;
use eframe::egui::{Align, Layout, RichText, Ui, Visuals};
use std::time::Duration;
use uuid::Uuid;

const SAVE_PERIOD_SECONDS: u64 = 10_000;

fn main() -> Result<(), eframe::Error> {
    env_logger::init(); // Log to stderr (if you run with `RUST_LOG=debug`).
    let options = eframe::NativeOptions {
        // Hide the OS-specific "chrome" around the window:
        decorated: false,
        // To have rounded corners we need transparency:
        transparent: true,
        resizable: false,
        initial_window_size: Some(egui::vec2(800.0, 400.0)),
        ..Default::default()
    };

    eframe::run_native(
        "Ruh Time Tracker", // unused title
        options,
        Box::new(|cc| Box::<MyApp>::new(MyApp::init(cc))),
    )
}

struct MyApp {
    backend: Backend,
    current_dialog: CurrentDialog,
    current_label: String,
    dialog_buffer: String,
    current_display_mode: DisplayMode,
}

#[derive(Copy, Clone)]
enum DisplayMode {
    Full,
    Minimal,
}

enum CurrentDialog {
    None,
    AddProject,
    AddSubject,
}

impl MyApp {
    fn init(cc: &eframe::CreationContext<'_>) -> Self {
        let context = cc.egui_ctx.clone();
        std::thread::spawn(move || loop {
            std::thread::sleep(Duration::from_millis(SAVE_PERIOD_SECONDS));
            context.request_repaint();
        });

        Self {
            backend: Backend::load(),
            current_display_mode: DisplayMode::Full,
            current_dialog: CurrentDialog::None,
            dialog_buffer: "".to_string(),
            current_label: "".to_string(),
        }
    }

    fn build_projects(&mut self, ui: &mut Ui) {
        ui.set_min_width(400.0);
        ui.set_max_width(400.0);

        ui.vertical(|ui| {
            let projects = self.backend.projects.clone();
            for project in projects {
                ui.horizontal(|ui| {
                    let current_id = if let Some(cur_project) = &self.backend.current_project {
                        cur_project.lock().unwrap().id
                    } else {
                        Uuid::new_v4()
                    };

                    let r_project = project.lock().unwrap();

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
                self.current_dialog = CurrentDialog::AddProject;
            }
        });
    }

    fn build_subjects(&mut self, ui: &mut Ui) {
        ui.set_min_width(400.0);
        ui.set_max_width(400.0);

        let Some(current_project) = self.backend.current_project.clone() else {
            return;
        };

        ui.vertical(|ui| {
            for subject in &current_project.lock().unwrap().subjects {
                ui.horizontal(|ui| {
                    let current_id = if let Some(cur_subject) = &self.backend.current_subject {
                        cur_subject.lock().unwrap().id
                    } else {
                        Uuid::new_v4()
                    };

                    let r_subject = subject.lock().unwrap();

                    let mut text = RichText::new(&r_subject.name);

                    if r_subject.id == current_id {
                        text = text.strong();
                    }

                    if ui.button(text).clicked() {
                        if current_id != r_subject.id {
                            self.stop_subject(true);
                        }
                        self.backend.set_current_subject(Some(subject.clone()));
                    }

                    ui.label(format_duration(r_subject.duration));
                });

                ui.add_space(5.0);
            }

            if ui.button("   +   ").clicked() {
                self.current_dialog = CurrentDialog::AddSubject;
            }
        });
    }

    fn start_subject(&mut self) {
        self.backend.start_subject();
        self.current_label = self.backend.get_current_work_name();
    }

    fn stop_subject(&mut self, force: bool) {
        self.backend.stop_subject(force);
        self.current_label = "".to_string();
    }
}

fn custom_window_frame(
    ctx: &egui::Context,
    frame: &mut eframe::Frame,
    _title: &str,
    display_mode: DisplayMode,
    add_contents: impl FnOnce(&mut Ui),
) {
    use egui::*;

    let panel_frame = Frame {
        fill: ctx.style().visuals.window_fill(),
        rounding: 10.0.into(),
        stroke: ctx.style().visuals.widgets.noninteractive.fg_stroke,
        outer_margin: 0.5.into(), // so the stroke is within the bounds
        ..Default::default()
    };

    match display_mode {
        DisplayMode::Full => {
            frame.set_window_size(Vec2::new(800., 400.));
        }

        DisplayMode::Minimal => {
            frame.set_window_size(Vec2::new(105., 60.));
            frame.set_always_on_top(true);
        }
    }

    CentralPanel::default().frame(panel_frame).show(ctx, |ui| {
        let app_rect = ui.max_rect();

        let mut c = (*ctx.style()).clone();
        c.text_styles.insert(
            TextStyle::Button,
            FontId::new(18.0, FontFamily::Proportional),
        );
        c.text_styles
            .insert(TextStyle::Body, FontId::new(18.0, FontFamily::Proportional));
        ctx.set_style(c);

        // Add the contents:
        let content_rect = { app_rect }.shrink(4.0);

        let mut content_ui = ui.child_ui(content_rect, *ui.layout());
        add_contents(&mut content_ui);
    });
}

impl eframe::App for MyApp {
    fn clear_color(&self, _visuals: &Visuals) -> [f32; 4] {
        egui::Rgba::TRANSPARENT.to_array() // Make sure we don't paint anything behind the rounded corners
    }

    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        let mode = self.current_display_mode;

        match self.current_display_mode {
            DisplayMode::Full => {
                custom_window_frame(ctx, frame, "_", mode, |ui: &mut Ui| {
                    ui.horizontal_top(|ui| {
                        ui.label(format!("Current work: {}", self.current_label,));

                        let mut visuals = ui.ctx().style().visuals.clone();

                        ui.with_layout(Layout::right_to_left(Align::Min), |ui| {
                            ui.horizontal(|ui| {
                                ui.selectable_value(&mut visuals, Visuals::light(), "☀");
                                ui.selectable_value(&mut visuals, Visuals::dark(), "🌙");

                                if self.backend.current_subject.is_some()
                                    && ui.button("⬇").clicked()
                                {
                                    self.current_display_mode = DisplayMode::Minimal;
                                }
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
                                        self.start_subject()
                                    }
                                }
                                WorkingMode::InProgress(_) => {
                                    if ui.button("PAUSE").clicked() {
                                        self.stop_subject(false);
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
                            egui::ScrollArea::vertical().show(ui, |ui| {
                                self.build_projects(ui);
                            });
                        });

                        ui.separator();

                        ui.push_id(2, |ui| {
                            egui::ScrollArea::vertical().show(ui, |ui| {
                                self.build_subjects(ui);
                            });
                        });
                    });
                });
            }
            DisplayMode::Minimal => {
                custom_window_frame(ctx, frame, "_", mode, |ui| {
                    ui.vertical_centered(|ui| {
                        ui.label(format_duration(self.backend.current_session_duration));
                        ui.horizontal_centered(|ui| {
                            match self.backend.working_mode {
                                WorkingMode::Idle => {
                                    if ui.button("START").clicked() {
                                        self.start_subject()
                                    }
                                }
                                WorkingMode::InProgress(_) => {
                                    if ui.button("PAUSE").clicked() {
                                        self.stop_subject(false);
                                    }
                                }
                            }

                            if ui.button("⬆").clicked() {
                                self.current_display_mode = DisplayMode::Full;
                            }
                        });
                    });
                });
            }
        }

        self.backend.update_time();

        match self.current_dialog {
            CurrentDialog::None => {}
            CurrentDialog::AddProject => {
                egui::Window::new("Add Project")
                    .collapsible(false)
                    .resizable(false)
                    .show(ctx, |ui| {
                        ui.horizontal(|ui| {
                            ui.add(egui::TextEdit::singleline(&mut self.dialog_buffer));

                            if ui.button("Cancel").clicked() {
                                self.current_dialog = CurrentDialog::None;
                                self.dialog_buffer = "".to_string();
                            }

                            if ui.button("Add").clicked() {
                                self.current_dialog = CurrentDialog::None;
                                self.backend.add_project(&self.dialog_buffer);
                                self.dialog_buffer = "".to_string();
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
                            ui.add(egui::TextEdit::singleline(&mut self.dialog_buffer));

                            if ui.button("Cancel").clicked() {
                                self.current_dialog = CurrentDialog::None;
                                self.dialog_buffer = "".to_string();
                            }

                            if ui.button("Add").clicked() {
                                self.current_dialog = CurrentDialog::None;
                                self.backend.add_subject(&self.dialog_buffer);
                                self.dialog_buffer = "".to_string();
                            }
                        });
                    });
            }
        }
    }
}
