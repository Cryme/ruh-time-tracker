//! Show a custom window frame instead of the default OS window chrome decorations.

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

mod backend;
mod frontend;
mod history;
mod util;

use crate::frontend::{DisplayMode, Frontend};
use eframe::egui;
use eframe::egui::Ui;

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
        Box::new(|cc| Box::<Frontend>::new(Frontend::init(cc))),
    )
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
        rounding: 8.0.into(),
        stroke: ctx.style().visuals.widgets.noninteractive.fg_stroke,
        outer_margin: 0.5.into(), // so the stroke is within the bounds
        ..Default::default()
    };

    match display_mode {
        DisplayMode::Statistic => {
            frame.set_window_size(Vec2::new(1200., 800.));
        }

        DisplayMode::Time | DisplayMode::Todo => {
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
