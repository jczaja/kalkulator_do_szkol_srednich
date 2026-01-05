#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release
#![allow(rustdoc::missing_crate_level_docs)] // it's an example

//use eframe::egui;
// Ref: https://www.otouczelnie.pl/kalkulator/osmoklasista

// I LO Szczecinek : https://cloud-d.edupage.org/cloud/Regulamin_i_harmonogram_rekrutacji_2025_2026_do_I_LO_Szczecinek.pdf?z%3A0mYjQe50qOoFEVT1pUcg5F%2BU1Qs%2BV%2FKMV5Rnq4AztxBI4PNFFctdFVyIc46bQWYEnD07Yx83qP7RLhSDLOMznQ%3D%3D
// XV LO Gdansk : https://lo15.edu.gdansk.pl/Content/pub/452/rekrutacja%202025-26/regulamin_rekrutacji_2025_26.pdf

// TODO: zwolnienie z egzaminu

use egui_plot::{Bar, BarChart, Line, Plot, PlotPoints};
use macroquad::prelude::*; // Import necessary components
                           //
struct Threshold<'a> {
    base_name: &'a str,
    points: f32,
    second_course: &'a str,
}
impl<'a> Threshold<'a> {
    pub fn new(base_name: &'a str, points: f32, second_course: &'a str) -> Threshold<'a> {
        Threshold {
            base_name,
            points,
            second_course,
        }
    }
    pub fn get_full_name(&self) -> String {
        format!(
            "{} (przedmiot: {}) - {} pkt",
            self.base_name, self.second_course, self.points
        )
    }
}

// Each tuple is representing score (in percentage) of given exam and name of topic
struct ExamResults<'a> {
    polish: (u8, &'a str),
    math: (u8, &'a str),
    second_language: (u8, &'a str),
}

impl ExamResults<'_> {
    pub fn calculate_points(&self) -> Result<f32, &str> {
        if self.polish.0 > 100 || self.math.0 > 100 || self.second_language.0 > 100 {
            return Err("Score cannot be greater than 100");
        }
        Ok(self.polish.0 as f32 * 0.35
            + self.math.0 as f32 * 0.35
            + self.second_language.0 as f32 * 0.3)
    }
    pub fn polish_as_points(&self) -> f32 {
        self.polish.0 as f32 * 0.35
    }
    pub fn math_as_points(&self) -> f32 {
        self.math.0 as f32 * 0.35
    }
    pub fn second_language_as_points(&self) -> f32 {
        self.second_language.0 as f32 * 0.30
    }
}
// finalist of the subject competition
//laureate of the thematic or interdisciplinary competition (7 points)
// finalist of the thematic or interdisciplinary competition (5 points)

#[repr(u8)]
enum ContestCuratorOverVoidship<'a> {
    None = 0,
    SubjectFinalist(&'a str) = 10,
    ThematicLaureate(&'a str) = 7,
    ThematicFinalist(&'a str) = 5,
}

//tytułu finalisty konkursu z przedmiotu lub przedmiotów artystycznych objętych ramowym planem nauczania szkoły artystycznej (10 pkt)

//tytułu laureata turnieju z przedmiotu lub przedmiotów artystycznych nieobjętych ramowym planem nauczania szkoły artystycznej (4 pkt)

//tytułu finalisty turnieju z przedmiotu lub przedmiotów artystycznych nieobjętych ramowym planem nauczania szkoły artystycznej (3 pkt)

#[repr(u8)]
enum ContestArtisticNational {
    None = 0,
    MandatoryFinalist = 10,
    PeriferialLaureate = 4,
    PeriferialFinalist = 3,
}

//brak (0 pkt)

//dwóch lub więcej tytułów finalisty konkursu przedmiotowego (10 pkt)
//dwóch lub więcej tytułów laureata konkursu tematycznego lub interdyscyplinarnego (7 pkt)
//dwóch lub więcej tytułów finalisty konkursu tematycznego lub interdyscyplinarnego (5 pkt)
//tytułu finalisty konkursu przedmiotowego (7 pkt)
//tytułu laureata konkursu tematycznego lub interdyscyplinarnego (5 pkt)
//tytułu finalisty konkursu tematycznego lub interdyscyplinarnego (3 pkt)

#[repr(u8)]
enum ContestCuratorVoidship {
    None = 0,
    AtLeastTwoTimesSubjectFinalist = 10, // Can it be two times the same subject?
    AtLeastTwoTimesThematicLaureate = 7,
    AtLeastTwoTimesThematicFinalist = 5,
}

impl ContestCuratorVoidship {
    pub const SubjectFinalist: ContestCuratorVoidship =
        ContestCuratorVoidship::AtLeastTwoTimesThematicLaureate; // Can it be two times the same subject?
    pub const ThematicFinalist: ContestCuratorVoidship =
        ContestCuratorVoidship::AtLeastTwoTimesThematicFinalist;
}

struct CertificateResults<'a> {
    polish: (u8, &'a str),
    math: (u8, &'a str),
    first_addtional_course: (u8, &'a str),
    second_addtional_course: (u8, &'a str),
    achievements: u8,
    honors: bool,
    volounteering: bool,
}

impl CertificateResults<'_> {
    pub fn calculate_points(&self) -> Result<f32, &str> {
        let get_course_points = |grade: u8| -> f32 {
            match grade {
                6 => 18.0,
                5 => 17.0,
                4 => 14.0,
                3 => 8.0,
                2 => 2.0,
                _ => panic!("Grade needs to be between 2 and 6"),
            }
        };

        let get_volunteering_points = |is_voluntering: bool| -> f32 {
            if is_voluntering {
                3.0
            } else {
                0.0
            }
        };

        let get_honors_points = |with_honors: bool| -> f32 {
            if with_honors {
                7.0
            } else {
                0.0
            }
        };

        if (2..=6).contains(&self.polish.0)
            && (2..=6).contains(&self.math.0)
            && (2..=6).contains(&self.first_addtional_course.0)
            && (2..=6).contains(&self.second_addtional_course.0)
            && (0..=18).contains(&self.achievements)
        {
            Ok(get_course_points(self.polish.0)
                + get_course_points(self.math.0)
                + get_course_points(self.first_addtional_course.0)
                + get_course_points(self.second_addtional_course.0)
                + self.achievements as f32
                + get_honors_points(self.honors)
                + get_volunteering_points(self.volounteering))
        } else {
            Err("Grade needs to be between 2 and 6 and achievemnts needs to be between 0 and 18 points")
        }
    }
}

#[macroquad::main("kalkulator punktów do szkoły średniej")]
async fn main() {
    let mut pol_value: u8 = 53;
    let mut ang_value: u8 = 96;
    let mut mat_value: u8 = 40;
    let mut initialization = true;

    let mut cpol_value: u8 = 4;
    let mut cang_value: u8 = 6;
    let mut cmat_value: u8 = 5;
    let mut cinf_value: u8 = 6;
    let mut achv_value: u8 = 0;
    let mut vol_value: bool = true;
    let mut hon_value: bool = true;

    let mut selected = 0;
    let mut should_exit = false;
    let schools = vec![
        Threshold::new("LO II Gdansk - Klasa 1A (politechniczna)", 168.75, "Fizyka"),
        Threshold::new(
            "LO III Gdansk - Klasa 1A (ekonomiczna)",
            166.74,
            "Geografia",
        ),
        Threshold::new(
            "LO III Gdansk - Klasa 1D-1 (politechniczna)",
            171.64,
            "Informatyka",
        ),
        Threshold::new(
            "LO III Gdansk - Klasa 1D-2 (politechniczna)",
            167.17,
            "Informatyka",
        ),
        Threshold::new(
            "LO VIII Gdansk - Klasa 1BD-1 (mat-geo-ang)",
            170.6,
            "Geografia",
        ),
        Threshold::new(
            "LO VIII Gdansk - Klasa 1BD-2 (mat-inf-ang)",
            170.3,
            "Informatyka",
        ),
        Threshold::new("LO IX Gdansk - Klasa 1A (mat-fiz)", 161.90, "Fizyka"),
        Threshold::new(
            "LO IX Gdansk - Klasa 1C (mat-fiz-inf)",
            144.90,
            "Informatyka",
        ),
        Threshold::new("LO IX Gdansk - Klasa 1E (mat-fiz-inf)", 157.80, "WOS"),
        Threshold::new(
            "LO X Gdansk - Klasa 1A (dwujezyczna politechniczna)",
            160.87,
            "Fizyka",
        ),
        Threshold::new("LO XV Gdansk - Klasa 1A (mat-inf-ang)", 147.65, "Fizyka"),
        Threshold::new("LO XV Gdansk - Klasa 1D (mat-geo-ang)", 155.25, "Geografia"),
        Threshold::new(
            "LO XIX Gdansk - Klasa 1A (ekonomiczna)",
            162.70,
            "Geografia",
        ),
        Threshold::new(
            "LO XIX Gdansk - Klasa 1B (artystyczna-muzyczna)",
            136.20,
            "Historia",
        ),
        Threshold::new("LO XIX Gdansk - Klasa 1E (mat-fiz-ang)", 167.70, "Fizyka"),
    ];
    let mut total_points: f32 = 0.0;

    loop {
        if should_exit {
            break; // lub return, zależnie od struktury programu
        }
        clear_background(WHITE);

        egui_macroquad::ui(|egui_ctx| {
            egui_macroquad::egui::CentralPanel::default().show(egui_ctx, |ui| {
                let window_width = ui.available_width();
                let window_height = ui.available_height();
                let widget_width = window_width / 5.0;
                let widget_height = window_height / 10.0;
                let font_size = widget_height / 4.0;
                ui.style_mut().spacing.slider_width = widget_width;
                ui.style_mut().spacing.interact_size.y = widget_height;
                ui.style_mut().text_styles.insert(
                    egui_macroquad::egui::TextStyle::Body,
                    egui_macroquad::egui::FontId::new(
                        font_size,
                        egui_macroquad::egui::FontFamily::Proportional,
                    ),
                );

                // język polski
                ui.horizontal(|ui| {
                    ui.vertical(|ui| {
                        ui.horizontal(|ui| {
                            ui.add(egui_macroquad::egui::Label::new(
                                egui_macroquad::egui::RichText::new(format!(
                                    "Egzamin Język polski:      "
                                ))
                                .size(font_size),
                            ));
                            let pol_slider = ui.add_sized(
                                [widget_width, widget_height * 0.5],
                                egui_macroquad::egui::Slider::new(&mut pol_value, 0..=100)
                                    .step_by(1.0)
                                    .show_value(false),
                            );
                            ui.add(egui_macroquad::egui::Label::new(
                                egui_macroquad::egui::RichText::new(format!("{} %", pol_value))
                                    .size(font_size),
                            ));

                            if initialization {
                                pol_slider.request_focus();
                                initialization = false;
                            }
                        });
                        ui.horizontal(|ui| {
                            ui.add(egui_macroquad::egui::Label::new(
                                egui_macroquad::egui::RichText::new(format!(
                                    "Egzamin Matematyka:     "
                                ))
                                .size(font_size),
                            ));
                            let mat_slider = ui.add_sized(
                                [widget_width, widget_height * 0.5],
                                egui_macroquad::egui::Slider::new(&mut mat_value, 0..=100)
                                    .step_by(1.0)
                                    .show_value(false),
                            );
                            ui.add(egui_macroquad::egui::Label::new(
                                egui_macroquad::egui::RichText::new(format!("{} %", mat_value))
                                    .size(font_size),
                            ));
                        });
                        ui.horizontal(|ui| {
                            ui.add(egui_macroquad::egui::Label::new(
                                egui_macroquad::egui::RichText::new(format!(
                                    "Egzamin Język angielski: "
                                ))
                                .size(font_size),
                            ));
                            let ang_slider = ui.add_sized(
                                [widget_width, widget_height * 0.5],
                                egui_macroquad::egui::Slider::new(&mut ang_value, 0..=100)
                                    .step_by(1.0)
                                    .show_value(false),
                            );
                            ui.add(egui_macroquad::egui::Label::new(
                                egui_macroquad::egui::RichText::new(format!("{} %", ang_value))
                                    .size(font_size),
                            ));
                        });
                        // Świadectwo
                        ui.horizontal(|ui| {
                            ui.add(egui_macroquad::egui::Label::new(
                                egui_macroquad::egui::RichText::new(format!(
                                    "Świadectwo Język polski:   "
                                ))
                                .size(font_size),
                            ));
                            let cpol_slider = ui.add_sized(
                                [widget_width, widget_height * 0.5],
                                egui_macroquad::egui::Slider::new(&mut cpol_value, 2..=6)
                                    .step_by(1.0)
                                    .show_value(false),
                            );
                            ui.add(egui_macroquad::egui::Label::new(
                                egui_macroquad::egui::RichText::new(format!("{} ", cpol_value))
                                    .size(font_size),
                            ));
                        });
                        ui.horizontal(|ui| {
                            ui.add(egui_macroquad::egui::Label::new(
                                egui_macroquad::egui::RichText::new(format!(
                                    "Świadectwo Matematyka:   "
                                ))
                                .size(font_size),
                            ));
                            let cmat_slider = ui.add_sized(
                                [widget_width, widget_height * 0.5],
                                egui_macroquad::egui::Slider::new(&mut cmat_value, 2..=6)
                                    .step_by(1.0)
                                    .show_value(false),
                            );
                            ui.add(egui_macroquad::egui::Label::new(
                                egui_macroquad::egui::RichText::new(format!("{} ", cmat_value))
                                    .size(font_size),
                            ));
                        });
                        ui.horizontal(|ui| {
                            ui.add(egui_macroquad::egui::Label::new(
                                egui_macroquad::egui::RichText::new(format!(
                                    "Świadectwo Język angielski: "
                                ))
                                .size(font_size),
                            ));
                            let cang_slider = ui.add_sized(
                                [widget_width, widget_height * 0.5],
                                egui_macroquad::egui::Slider::new(&mut cang_value, 2..=6)
                                    .step_by(1.0)
                                    .show_value(false),
                            );
                            ui.add(egui_macroquad::egui::Label::new(
                                egui_macroquad::egui::RichText::new(format!("{} ", cang_value))
                                    .size(font_size),
                            ));
                        });
                        ui.horizontal(|ui| {
                            ui.add(egui_macroquad::egui::Label::new(
                                egui_macroquad::egui::RichText::new(format!(
                                    "Świadectwo {}:              ",
                                    schools[selected].second_course
                                ))
                                .size(font_size),
                            ));
                            let cinf_slider = ui.add_sized(
                                [widget_width, widget_height * 0.5],
                                egui_macroquad::egui::Slider::new(&mut cinf_value, 2..=6)
                                    .step_by(1.0)
                                    .show_value(false),
                            );
                            ui.add(egui_macroquad::egui::Label::new(
                                egui_macroquad::egui::RichText::new(format!("{} ", cinf_value))
                                    .size(font_size),
                            ));
                        });
                        ui.horizontal(|ui| {
                            ui.add(egui_macroquad::egui::Label::new(
                                egui_macroquad::egui::RichText::new(format!("Czerwony pasek: "))
                                    .size(font_size),
                            ));
                            let honors_checked = ui.add_sized(
                                [widget_width, widget_height * 0.5],
                                egui_macroquad::egui::Checkbox::new(&mut hon_value, ""),
                            );
                        });
                    });

                    // List of secondary schools
                    ui.vertical(|ui| {
                        ui.horizontal(|ui| {
                            ui.add(egui_macroquad::egui::Label::new(
                                egui_macroquad::egui::RichText::new(format!("Osiągnięcia: "))
                                    .size(font_size),
                            ));
                            let achv_slider = ui.add_sized(
                                [widget_width, widget_height * 0.5],
                                egui_macroquad::egui::Slider::new(&mut achv_value, 0..=18)
                                    .step_by(1.0)
                                    .show_value(false),
                            );
                            ui.add(egui_macroquad::egui::Label::new(
                                egui_macroquad::egui::RichText::new(format!(
                                    "{} punkty",
                                    achv_value
                                ))
                                .size(font_size),
                            ));
                        });
                        ui.horizontal(|ui| {
                            ui.add(egui_macroquad::egui::Label::new(
                                egui_macroquad::egui::RichText::new(format!("Wolontariat: "))
                                    .size(font_size),
                            ));
                            let vol_checked = ui.add_sized(
                                [widget_width, widget_height * 0.5],
                                egui_macroquad::egui::Checkbox::new(&mut vol_value, ""),
                            );
                        });
                        // Punkty
                        ui.horizontal(|ui| {
                            let exam_points = ExamResults {
                                polish: (pol_value, "Polish"),
                                math: (mat_value, "Math"),
                                second_language: (ang_value, "English"),
                            }
                            .calculate_points()
                            .unwrap();

                            let certificate_points = CertificateResults {
                                polish: (cpol_value, "jezyk polski"),
                                math: (cmat_value, "matematyka"),
                                first_addtional_course: (cang_value, "jezyk angielski"),
                                second_addtional_course: (cinf_value, "informatyka"),
                                achievements: achv_value,
                                honors: hon_value,
                                volounteering: vol_value,
                            }
                            .calculate_points()
                            .unwrap();

                            total_points = certificate_points + exam_points;
                            ui.label(
                                egui_macroquad::egui::RichText::new(format!(
                                    "Punkty Do Szkoły średniej: "
                                ))
                                .size(font_size),
                            );
                            ui.label(
                                egui_macroquad::egui::RichText::new(format!("{}", total_points))
                                    .color(if total_points <= 100.0 {
                                        egui_macroquad::egui::Color32::RED
                                    } else if total_points <= 150.0 {
                                        egui_macroquad::egui::Color32::YELLOW
                                    } else {
                                        egui_macroquad::egui::Color32::GREEN
                                    })
                                    .size(font_size),
                            );
                        });

                        ui.horizontal(|ui| {
                            ui.add(egui_macroquad::egui::Label::new(
                                egui_macroquad::egui::RichText::new(format!("Wybór szkoły: "))
                                    .size(font_size),
                            ));
                        });

                        ui.horizontal(|ui| {
                            egui_macroquad::egui::ComboBox::from_label("")
                                .selected_text(schools[selected].get_full_name())
                                .show_ui(ui, |ui| {
                                    for (i, item) in schools.iter().enumerate() {
                                        ui.selectable_value(&mut selected, i, item.get_full_name());
                                    }
                                });
                        });
                        ui.horizontal(|ui| {
                            if ui
                                .add(egui_macroquad::egui::Button::new(
                                    egui_macroquad::egui::RichText::new(format!("Exit"))
                                        .size(font_size),
                                ))
                                .clicked()
                            {
                                should_exit = true;
                            };
                        });
                    });
                    ui.vertical(|ui| {
                        let points = PlotPoints::from_iter(vec![
                            [0.0, schools[selected].points as f64],
                            [4.0, schools[selected].points as f64],
                        ]);
                        let target = Line::new(schools[selected].get_full_name(), points);

                        let points_bar = Bar::new(2.0, total_points.into()).width(0.5);
                        let barchart = BarChart::new("Punkty", vec![points_bar]);

                        Plot::new("Punkty do szkoły średniej")
                            .legend(egui_plot::Legend::default())
                            .y_axis_label("Punkty")
                            .include_y(0.0)
                            .include_x(0.0)
                            .include_y(200.0)
                            .show(ui, |plot_ui| {
                                plot_ui.bar_chart(barchart);
                                plot_ui.line(target);
                            });
                    });
                });
            });
        });
        // Draw things before egui

        egui_macroquad::draw();

        // Draw things after egui

        next_frame().await;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_calculate_exam_points() -> Result<(), String> {
        assert_eq!(
            ExamResults {
                polish: (100, "Polish"),
                math: (100, "Math"),
                second_language: (100, "English")
            }
            .calculate_points(),
            Ok(100.0)
        );
        assert_eq!(
            ExamResults {
                polish: (110, "Polish"),
                math: (100, "Math"),
                second_language: (100, "English")
            }
            .calculate_points(),
            Err("Score cannot be greater than 100")
        );

        Ok(())
    }

    #[test]
    fn test_calculate_certificate_points() -> Result<(), String> {
        assert_eq!(
            CertificateResults {
                polish: (6, "jezyk polski"),
                math: (5, "matematyka"),
                first_addtional_course: (4, "jezyk angielski"),
                second_addtional_course: (3, "informatyka"),
                achievements: 10,
                honors: true,
                volounteering: false,
            }
            .calculate_points(),
            Ok(74.0)
        );

        assert_eq!(
            CertificateResults {
                polish: (2, "jezyk polski"),
                math: (2, "matematyka"),
                first_addtional_course: (2, "jezyk angielski"),
                second_addtional_course: (2, "informatyka"),
                achievements: 0,
                honors: false,
                volounteering: true,
            }
            .calculate_points(),
            Ok(11.0)
        );

        assert_eq!(CertificateResults{ polish : (2, "jezyk polski" ),
            math : (2, "matematyka"),
            first_addtional_course : (2, "jezyk angielski"),
            second_addtional_course : (2, "informatyka"),
            achievements : 100,
            honors : false,
            volounteering : true,

        }.calculate_points(), Err("Grade needs to be between 2 and 6 and achievemnts needs to be between 0 and 18 points"));

        Ok(())
    }
}
