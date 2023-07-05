use crate::backend::Backend;
use crate::history::HistoryRecord;
use crate::util::{format_number, get_days_from_month};
use chrono::{DateTime, Datelike, Local, Timelike};
use eframe::egui;
use eframe::egui::scroll_area::ScrollBarVisibility;
use eframe::egui::{Color32, FontId, RichText, Rounding, ScrollArea, Ui, Vec2};
use eframe::epaint::RectShape;

pub fn build_statistic(
    ui: &mut Ui,
    records: Vec<Vec<HistoryRecord>>,
    backend: &Backend,
    scroll_offset: &mut f32,
    date_range: (DateTime<Local>, DateTime<Local>),
) {
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

        ui.separator();

        let s = ScrollArea::horizontal()
            .scroll_bar_visibility(ScrollBarVisibility::AlwaysHidden)
            .scroll_offset(Vec2::new(*scroll_offset, 0.));

        s.show(ui, |ui| {
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
                ui.add_space(55.);
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

    let c = ScrollArea::both().show(ui, |ui| {
        ui.set_min_size(Vec2::new(1185.0, 340.));

        ui.horizontal(|ui| {
            ui.vertical(|ui| {
                let mut first_year_first_month = true;

                let mut i = 0usize;

                for year in date_range.0.year()..=date_range.1.year() {
                    for month in date_range.0.month()..=date_range.1.month() {
                        let from = if first_year_first_month {
                            first_year_first_month = false;

                            date_range.0.day()
                        } else {
                            1
                        };

                        let to = if year == date_range.1.year() && month == date_range.1.month() {
                            date_range.1.day()
                        } else {
                            get_days_from_month(year, month)
                        };

                        for day in from..=to {
                            let mut previous_ending = None;
                            let mut space_added = false;
                            let mut length = 0_f32;

                            ui.horizontal_centered(|ui| {
                                ui.horizontal(|ui| {
                                    ui.set_min_width(50.);

                                    ui.label(
                                        RichText::new(format!(
                                            "{}/{}",
                                            format_number(day),
                                            format_number(month)
                                        ))
                                        .font(FontId::proportional(13.0)),
                                    );
                                });

                                ui.add_space(5.);

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

                                    let (rect, response) =
                                        ui.allocate_exact_size(desired_size, egui::Sense::click());

                                    let project = backend
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

            // ui.vertical(|ui| {
            //     for val in &records {
            //         let mut previous_ending = None;
            //         let mut space_added = false;
            //         let mut length = 0. as f32;
            //
            //         ui.horizontal(|ui| {
            //             ui.add_space(55.);
            //             ui.set_min_height(15.);
            //
            //             for record in val {
            //                 if !space_added {
            //                     let d = record.start_date.hour() as f32 * 60.0 + record.start_date.minute() as f32;
            //                     ui.add_space(d);
            //                     length += d;
            //
            //                     space_added = true;
            //                 }
            //
            //                 let duration = record.get_duration();
            //
            //                 if duration.num_minutes() <= 0 {
            //                     continue;
            //                 }
            //
            //                 if let Some(prev) = previous_ending {
            //                     let dur = record.start_date.signed_duration_since(prev).num_minutes();
            //
            //                     if dur > 0 {
            //                         ui.add_space(dur as f32);
            //                         length += dur as f32;
            //                     }
            //                 }
            //
            //                 let desired_size =
            //                     egui::vec2(record.get_duration().num_minutes() as f32, 15.0);
            //
            //                 length += desired_size.x;
            //
            //                 let (rect, mut response) =
            //                     ui.allocate_exact_size(desired_size, egui::Sense::click());
            //
            //                 let project = backend
            //                     .projects
            //                     .get(&record.project_id)
            //                     .unwrap()
            //                     .lock()
            //                     .unwrap();
            //                 let subject = project
            //                     .subjects
            //                     .get(&record.subject_id)
            //                     .unwrap()
            //                     .lock()
            //                     .unwrap();
            //
            //                 response.on_hover_text(format!("{} : {}", project.name, subject.name));
            //
            //                 ui.painter().add(RectShape {
            //                     rect,
            //                     rounding: Rounding::same(4.0),
            //                     fill: Color32::from_rgb(
            //                         project.color.0,
            //                         project.color.1,
            //                         project.color.2,
            //                     ),
            //                     stroke: Default::default(),
            //                 });
            //
            //                 previous_ending = Some(record.end_date);
            //             }
            //
            //             if length < 60.0 * 24.0 {
            //                 ui.add_space(60.0 * 24.0 - length);
            //             }
            //         });
            //     }
            // });
        });
    });

    *scroll_offset = c.state.offset.x;

    ui.set_style(style);
}
