//! Show a custom window frame instead of the default OS window chrome decorations.

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

mod project;

use crate::project::{Backend, WorkingMode};
use eframe::egui;
use eframe::egui::{Align, Layout, RichText, Ui, Visuals};
use std::ops::Rem;
use std::time::Duration;
use uuid::Uuid;

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
    current_name: String,
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
            std::thread::sleep(Duration::from_millis(1000));
            context.request_repaint();
        });

        Self {
            backend: Backend::load(),
            current_dialog: CurrentDialog::None,
            current_name: "".to_string(),
        }
    }

    fn build_projects(&mut self, ui: &mut Ui) {
        ui.set_min_width(400.0);
        ui.set_max_width(400.0);

        ui.vertical(|ui| {
            for project in &self.backend.projects {
                ui.horizontal(|ui| {
                    let current_id = if let Some(cur_project) = &self.backend.current_project {
                        cur_project.lock().unwrap().id
                    } else {
                        Uuid::new_v4()
                    };

                    let r_project = project.lock().unwrap();

                    let mut text = RichText::new(&r_project.name);

                    if &r_project.id == &current_id {
                        text = text.strong();
                    }

                    if ui.button(text).clicked() {
                        self.backend.current_project = Some(project.clone());
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

        let Some(current_project) = &self.backend.current_project else {
            return;
        };

        ui.vertical(|ui| {
            let project = current_project.lock().unwrap();

            for subject in &project.subjects {
                ui.horizontal(|ui| {
                    let current_id = if let Some(cur_subject) = &self.backend.current_subject {
                        cur_subject.lock().unwrap().id
                    } else {
                        Uuid::new_v4()
                    };

                    let r_subject = subject.lock().unwrap();

                    let mut text = RichText::new(&r_subject.name);

                    if &r_subject.id == &current_id {
                        text = text.strong();
                    }

                    if ui.button(text).clicked() {
                        self.backend.current_subject = Some(subject.clone());
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
}

fn format_duration(duration: Duration) -> String {
    const HOUR_S: f64 = 60.0 * 60.0;

    let spent = duration.as_secs_f64();

    let hours = (spent / HOUR_S).round();
    let minutes = (spent.rem(&HOUR_S) / 60.0).round();

    let hours_d = if hours > 9.0 {
        format!("{hours}")
    } else {
        format!("0{hours}")
    };
    let minutes_d = if minutes > 9.0 {
        format!("{minutes}")
    } else {
        format!("0{minutes}")
    };

    format!(" {hours_d}:{minutes_d}")
}

fn custom_global_dark_light_mode_buttons(ui: &mut Ui) {
    let mut visuals = ui.ctx().style().visuals.clone();
    custom_light_dark_radio_buttons(&mut visuals, ui);
    ui.ctx().set_visuals(visuals);
}

fn custom_light_dark_radio_buttons(vis: &mut Visuals, ui: &mut Ui) {
    ui.with_layout(Layout::right_to_left(Align::Min), |ui| {
        ui.horizontal(|ui| {
            ui.selectable_value(vis, Visuals::light(), "☀");
            ui.selectable_value(vis, Visuals::dark(), "🌙");
        });
    });
}

fn custom_window_frame(
    ctx: &egui::Context,
    _frame: &mut eframe::Frame,
    _title: &str,
    add_contents: impl FnOnce(&mut egui::Ui),
) {
    use egui::*;

    let panel_frame = Frame {
        fill: ctx.style().visuals.window_fill(),
        rounding: 10.0.into(),
        stroke: ctx.style().visuals.widgets.noninteractive.fg_stroke,
        outer_margin: 0.5.into(), // so the stroke is within the bounds
        ..Default::default()
    };

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
    fn clear_color(&self, _visuals: &egui::Visuals) -> [f32; 4] {
        egui::Rgba::TRANSPARENT.to_array() // Make sure we don't paint anything behind the rounded corners
    }

    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        custom_window_frame(ctx, frame, "_", |ui| {
            ui.horizontal_top(|ui| {
                ui.label(format!(
                    "Current work: {}",
                    self.backend.get_current_work_name()
                ));
                custom_global_dark_light_mode_buttons(ui);
            });

            ui.horizontal(|ui| {
                ui.set_min_height(55.0);
                ui.set_max_height(55.0);

                if self.backend.current_subject.is_some() {
                    match self.backend.working_mode {
                        WorkingMode::Idle => {
                            if ui.button("START").clicked() {
                                self.backend.start_subject();
                            }
                        }
                        WorkingMode::InProgress(_) => {
                            if ui.button("PAUSE").clicked() {
                                self.backend.stop_subject();
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

        self.backend.update_time();

        match self.current_dialog {
            CurrentDialog::None => {}
            CurrentDialog::AddProject => {
                egui::Window::new("Add Project")
                    .collapsible(false)
                    .resizable(false)
                    .show(ctx, |ui| {
                        ui.horizontal(|ui| {
                            ui.add(egui::TextEdit::singleline(&mut self.current_name));

                            if ui.button("Cancel").clicked() {
                                self.current_dialog = CurrentDialog::None;
                                self.current_name = "".to_string();
                            }

                            if ui.button("Add").clicked() {
                                self.current_dialog = CurrentDialog::None;
                                self.backend.add_project(&*self.current_name);
                                self.current_name = "".to_string();
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
                            ui.add(egui::TextEdit::singleline(&mut self.current_name));

                            if ui.button("Cancel").clicked() {
                                self.current_dialog = CurrentDialog::None;
                                self.current_name = "".to_string();
                            }

                            if ui.button("Add").clicked() {
                                self.current_dialog = CurrentDialog::None;
                                self.backend.add_subject(&*self.current_name);
                                self.current_name = "".to_string();
                            }
                        });
                    });
            }
        }
    }
}
