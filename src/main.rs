#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release
#![allow(rustdoc::missing_crate_level_docs)] // it's an example

//use eframe::egui;
// Ref: https://www.otouczelnie.pl/kalkulator/osmoklasista

// I LO Szczecinek : https://cloud-d.edupage.org/cloud/Regulamin_i_harmonogram_rekrutacji_2025_2026_do_I_LO_Szczecinek.pdf?z%3A0mYjQe50qOoFEVT1pUcg5F%2BU1Qs%2BV%2FKMV5Rnq4AztxBI4PNFFctdFVyIc46bQWYEnD07Yx83qP7RLhSDLOMznQ%3D%3D
// XV LO Gdansk : https://lo15.edu.gdansk.pl/Content/pub/452/rekrutacja%202025-26/regulamin_rekrutacji_2025_26.pdf

// TODO: zwolnienie z egzaminu
// TODO: koszalin . Zobacz jak na komorce to wyglada
// TODO: android TV ikonka (wlasny manifest)

use egui_plot::{Bar, BarChart, Line, Plot, PlotPoints};
use macroquad::prelude::*; // Import necessary components
                           //

enum SelectionState {
    None,
    City,
    School,
    Profil,
    Exit,
}

struct School<'a> {
    name: &'a str,
    profiles: &'a [Threshold<'a>],
}

impl<'a> School<'a> {
    pub fn new(name: &'a str, profiles: &'a [Threshold<'a>]) -> School<'a> {
        School { name, profiles }
    }

    pub fn get_full_name(&self) -> String {
        format!("{}", self.name)
    }
}

impl<'a> std::fmt::Display for School<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name)
    }
}

impl<'a> PartialEq for School<'a> {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
    }
}

enum City<'a> {
    Gdansk(&'a [School<'a>]),
    Koszalin(&'a [School<'a>]),
    Poznan(&'a [School<'a>]),
}

impl<'a> City<'a> {
    pub fn get_schools(&self) -> &'a [School<'a>] {
        match self {
            City::Gdansk(schools) => schools,
            City::Koszalin(schools) => schools,
            City::Poznan(schools) => schools,
        }
    }
}

impl<'a> std::fmt::Display for City<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let as_str = match self {
            // TODO: how to make it stringify in Rust
            City::Gdansk(_) => "Gdańsk",
            City::Koszalin(_) => "Koszalin",
            City::Poznan(_) => "Poznań",
        };
        write!(f, "{}", as_str)
    }
}

impl<'a> PartialEq for City<'a> {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (City::Gdansk(_), City::Gdansk(_))
            | (City::Koszalin(_), City::Koszalin(_))
            | (City::Poznan(_), City::Poznan(_)) => true,
            _ => false,
        }
    }
}

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

fn process_city(
    ui: &mut egui_macroquad::egui::Ui,
    font_size: f32,
    widget_width: f32,
    widget_height: f32,
    cities: &[City],
    selected_city: &mut usize,
    initialization: &mut bool,
) -> SelectionState {
    let mut state = SelectionState::City;

    ui.vertical(|ui| {
        (0..cities.len()).for_each(|c| {
            let alt_city = &cities[c];
            ui.radio_value(&mut *selected_city, c, format!("{alt_city}"));
        });
    });
    let ok_button = ui.add(egui_macroquad::egui::Button::new(
        egui_macroquad::egui::RichText::new(format!("OK")).size(font_size),
    ));
    if *initialization {
        ok_button.request_focus();
        *initialization = false;
    }
    if ok_button.clicked() {
        state = SelectionState::None;
        *initialization = true;
    };

    state
}

fn process_school(
    ui: &mut egui_macroquad::egui::Ui,
    font_size: f32,
    widget_width: f32,
    widget_height: f32,
    schools: &[School],
    selected_school: &mut usize,
    initialization: &mut bool,
) -> SelectionState {
    let mut state = SelectionState::School;

    ui.vertical(|ui| {
        (0..schools.len()).for_each(|c| {
            let alt_school = &schools[c];
            ui.radio_value(&mut *selected_school, c, format!("{alt_school}"));
        });
    });
    let ok_button = ui.add(egui_macroquad::egui::Button::new(
        egui_macroquad::egui::RichText::new(format!("OK")).size(font_size),
    ));
    if *initialization {
        ok_button.request_focus();
        *initialization = false;
    }
    if ok_button.clicked() {
        state = SelectionState::None;
        *initialization = true;
    };

    state
}

//cities[selected_city].get_schools().first().unwrap()
fn process_profil(
    ui: &mut egui_macroquad::egui::Ui,
    font_size: f32,
    widget_width: f32,
    widget_height: f32,
    profils: &[Threshold],
    selected_profil: &mut usize,
    initialization: &mut bool,
) -> SelectionState {
    let mut state = SelectionState::Profil;

    ui.vertical(|ui| {
        (0..profils.len()).for_each(|c| {
            let alt_profil = &profils[c];
            ui.radio_value(
                &mut *selected_profil,
                c,
                format!("{}", alt_profil.get_full_name()),
            );
        });
    });
    let ok_button = ui.add(egui_macroquad::egui::Button::new(
        egui_macroquad::egui::RichText::new(format!("OK")).size(font_size),
    ));
    if *initialization {
        ok_button.request_focus();
        *initialization = false;
    }
    if ok_button.clicked() {
        state = SelectionState::None;
        *initialization = true;
    };

    state
}

fn process_none(
    ui: &mut egui_macroquad::egui::Ui,
    font_size: f32,
    widget_width: f32,
    widget_height: f32,
    exams: &mut ExamResults,
    certs: &mut CertificateResults,
    initialization: &mut bool,
    prev_gamestate: &SelectionState,
    city: &City,
    school: &School,
    profil: &Threshold,
) -> SelectionState {
    let mut state = SelectionState::None;
    let mut total_points = 0.0;

    let set_focus = |widget: &egui_macroquad::egui::Response, initialization: &mut bool| {
        if *initialization {
            // Make focus depending on previous selection state
            widget.request_focus();
            *initialization = false;
        }
    };

    // język polski
    ui.horizontal(|ui| {
        ui.vertical(|ui| {
            ui.horizontal(|ui| {
                ui.add(egui_macroquad::egui::Label::new(
                    egui_macroquad::egui::RichText::new(format!("Egzamin Język polski:      "))
                        .size(font_size),
                ));
                let pol_slider = ui.add_sized(
                    [widget_width, widget_height * 0.5],
                    egui_macroquad::egui::Slider::new(&mut exams.polish.0, 0..=100)
                        .step_by(1.0)
                        .show_value(false),
                );
                ui.add(egui_macroquad::egui::Label::new(
                    egui_macroquad::egui::RichText::new(format!("{} %", exams.polish.0))
                        .size(font_size),
                ));

                if let SelectionState::None = prev_gamestate {
                    set_focus(&pol_slider, initialization);
                }
            });
            ui.horizontal(|ui| {
                ui.add(egui_macroquad::egui::Label::new(
                    egui_macroquad::egui::RichText::new(format!("Egzamin Matematyka:     "))
                        .size(font_size),
                ));
                let mat_slider = ui.add_sized(
                    [widget_width, widget_height * 0.5],
                    egui_macroquad::egui::Slider::new(&mut exams.math.0, 0..=100)
                        .step_by(1.0)
                        .show_value(false),
                );
                ui.add(egui_macroquad::egui::Label::new(
                    egui_macroquad::egui::RichText::new(format!("{} %", exams.math.0))
                        .size(font_size),
                ));
            });
            ui.horizontal(|ui| {
                ui.add(egui_macroquad::egui::Label::new(
                    egui_macroquad::egui::RichText::new(format!("Egzamin Język angielski: "))
                        .size(font_size),
                ));
                let ang_slider = ui.add_sized(
                    [widget_width, widget_height * 0.5],
                    egui_macroquad::egui::Slider::new(&mut exams.second_language.0, 0..=100)
                        .step_by(1.0)
                        .show_value(false),
                );
                ui.add(egui_macroquad::egui::Label::new(
                    egui_macroquad::egui::RichText::new(format!("{} %", exams.second_language.0))
                        .size(font_size),
                ));
            });
            // Świadectwo
            ui.horizontal(|ui| {
                ui.add(egui_macroquad::egui::Label::new(
                    egui_macroquad::egui::RichText::new(format!("Świadectwo Język polski:   "))
                        .size(font_size),
                ));
                let cpol_slider = ui.add_sized(
                    [widget_width, widget_height * 0.5],
                    egui_macroquad::egui::Slider::new(&mut certs.polish.0, 2..=6)
                        .step_by(1.0)
                        .show_value(false),
                );
                ui.add(egui_macroquad::egui::Label::new(
                    egui_macroquad::egui::RichText::new(format!("{} ", certs.polish.0))
                        .size(font_size),
                ));
            });
            ui.horizontal(|ui| {
                ui.add(egui_macroquad::egui::Label::new(
                    egui_macroquad::egui::RichText::new(format!("Świadectwo Matematyka:   "))
                        .size(font_size),
                ));
                let cmat_slider = ui.add_sized(
                    [widget_width, widget_height * 0.5],
                    egui_macroquad::egui::Slider::new(&mut certs.math.0, 2..=6)
                        .step_by(1.0)
                        .show_value(false),
                );
                ui.add(egui_macroquad::egui::Label::new(
                    egui_macroquad::egui::RichText::new(format!("{} ", certs.math.0))
                        .size(font_size),
                ));
            });
            ui.horizontal(|ui| {
                ui.add(egui_macroquad::egui::Label::new(
                    egui_macroquad::egui::RichText::new(format!("Świadectwo Język angielski: "))
                        .size(font_size),
                ));
                let cang_slider = ui.add_sized(
                    [widget_width, widget_height * 0.5],
                    egui_macroquad::egui::Slider::new(&mut certs.first_addtional_course.0, 2..=6)
                        .step_by(1.0)
                        .show_value(false),
                );
                ui.add(egui_macroquad::egui::Label::new(
                    egui_macroquad::egui::RichText::new(format!(
                        "{} ",
                        certs.first_addtional_course.0
                    ))
                    .size(font_size),
                ));
            });
            ui.horizontal(|ui| {
                ui.add(egui_macroquad::egui::Label::new(
                    egui_macroquad::egui::RichText::new(format!(
                        "Świadectwo {}:              ",
                        profil.second_course
                    ))
                    .size(font_size),
                ));
                let cinf_slider = ui.add_sized(
                    [widget_width, widget_height * 0.5],
                    egui_macroquad::egui::Slider::new(&mut certs.second_addtional_course.0, 2..=6)
                        .step_by(1.0)
                        .show_value(false),
                );
                ui.add(egui_macroquad::egui::Label::new(
                    egui_macroquad::egui::RichText::new(format!(
                        "{} ",
                        certs.second_addtional_course.0
                    ))
                    .size(font_size),
                ));
            });
            ui.horizontal(|ui| {
                ui.add(egui_macroquad::egui::Label::new(
                    egui_macroquad::egui::RichText::new(format!("Czerwony pasek: "))
                        .size(font_size),
                ));
                ui.scope(|ui| {
                    ui.style_mut().spacing.icon_width = font_size;
                    let honors_checked = ui.add_sized(
                        [widget_width, widget_height * 0.5],
                        egui_macroquad::egui::Checkbox::new(&mut certs.honors, ""),
                    );
                });
            });
        });

        // List of secondary schools
        ui.vertical(|ui| {
            ui.horizontal(|ui| {
                ui.add(egui_macroquad::egui::Label::new(
                    egui_macroquad::egui::RichText::new(format!("Osiągnięcia: ")).size(font_size),
                ));
                let achv_slider = ui.add_sized(
                    [widget_width, widget_height * 0.5],
                    egui_macroquad::egui::Slider::new(&mut certs.achievements, 0..=18)
                        .step_by(1.0)
                        .show_value(false),
                );
                ui.add(egui_macroquad::egui::Label::new(
                    egui_macroquad::egui::RichText::new(format!("{} punkty", certs.achievements))
                        .size(font_size),
                ));
            });
            ui.horizontal(|ui| {
                ui.add(egui_macroquad::egui::Label::new(
                    egui_macroquad::egui::RichText::new(format!("Wolontariat: ")).size(font_size),
                ));
                ui.scope(|ui| {
                    ui.style_mut().spacing.icon_width = font_size;
                    let vol_checked = ui.add_sized(
                        [widget_width, widget_height * 0.5],
                        egui_macroquad::egui::Checkbox::new(&mut certs.volounteering, ""),
                    );
                });
            });
            // Punkty
            ui.horizontal(|ui| {
                let exam_points = exams.calculate_points().unwrap();

                let certificate_points = certs.calculate_points().unwrap();

                total_points = certificate_points + exam_points;
                ui.label(
                    egui_macroquad::egui::RichText::new(format!("Punkty Do Szkoły średniej: "))
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

            // by default there is a button "Wybierz miasto" and label next to it:
            // "<miasto>"

            ui.horizontal(|ui| {
                ui.add(egui_macroquad::egui::Label::new(
                    egui_macroquad::egui::RichText::new("Miasto: ".to_string()).size(font_size),
                ));

                let city_button = ui.add(egui_macroquad::egui::Button::new(
                    egui_macroquad::egui::RichText::new(format!("{city}")).size(font_size),
                ));

                if let SelectionState::City = prev_gamestate {
                    set_focus(&city_button, initialization);
                }
                if city_button.clicked() {
                    state = SelectionState::City;
                    *initialization = true;
                };
            });

            ui.horizontal(|ui| {
                ui.add(egui_macroquad::egui::Label::new(
                    egui_macroquad::egui::RichText::new("Szkoła: ".to_string()).size(font_size),
                ));

                let school_button = ui.add(egui_macroquad::egui::Button::new(
                    egui_macroquad::egui::RichText::new(format!("{school}")).size(font_size),
                ));

                if let SelectionState::School = prev_gamestate {
                    set_focus(&school_button, initialization);
                }
                if school_button.clicked() {
                    state = SelectionState::School;
                    *initialization = true;
                };
            });

            ui.horizontal(|ui| {
                ui.add(egui_macroquad::egui::Label::new(
                    egui_macroquad::egui::RichText::new(format!("Profil: ")).size(font_size),
                ));
                let profil_button = ui.add(egui_macroquad::egui::Button::new(
                    egui_macroquad::egui::RichText::new(format!("{}", profil.get_full_name()))
                        .size(font_size),
                ));

                if let SelectionState::Profil = prev_gamestate {
                    set_focus(&profil_button, initialization);
                }
                if profil_button.clicked() {
                    state = SelectionState::Profil;
                    *initialization = true;
                };
            });

            ui.horizontal(|ui| {
                if ui
                    .add(egui_macroquad::egui::Button::new(
                        egui_macroquad::egui::RichText::new(format!("Exit")).size(font_size),
                    ))
                    .clicked()
                {
                    state = SelectionState::Exit;
                };
            });
        });
        ui.vertical(|ui| {
            let points = PlotPoints::from_iter(vec![
                [0.0, profil.points as f64],
                [4.0, profil.points as f64],
            ]);
            let target = Line::new(profil.get_full_name(), points);
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
    state
}

#[macroquad::main("kalkulator punktów do szkoły średniej")]
async fn main() {
    let mut pol_value: u8 = 69;
    let mut ang_value: u8 = 100;
    let mut mat_value: u8 = 60;
    let mut initialization = true;

    let mut cpol_value: u8 = 4;
    let mut cang_value: u8 = 6;
    let mut cmat_value: u8 = 5;
    let mut cinf_value: u8 = 6;
    let mut achv_value: u8 = 0;
    let mut vol_value: bool = true;
    let mut hon_value: bool = true;

    let mut exam_points = ExamResults {
        polish: (pol_value, "Polish"),
        math: (mat_value, "Math"),
        second_language: (ang_value, "English"),
    };

    let mut certs = CertificateResults {
        polish: (cpol_value, "jezyk polski"),
        math: (cmat_value, "matematyka"),
        first_addtional_course: (cang_value, "jezyk angielski"),
        second_addtional_course: (cinf_value, "informatyka"),
        achievements: achv_value,
        honors: hon_value,
        volounteering: vol_value,
    };

    let g1 = &[Threshold::new(
        "Klasa 1A (politechniczna)\n",
        168.75,
        "Fizyka",
    )];
    let g2 = &[
        Threshold::new("LO III - Klasa 1A (ekonomiczna)\n", 166.74, "Geografia"),
        Threshold::new(
            "LO III - Klasa 1D-1(politechniczna)\n",
            171.64,
            "Informatyka",
        ),
        Threshold::new(
            "LO III - Klasa 1D-2 (politechniczna)\n",
            167.17,
            "Informatyka",
        ),
    ];

    let g3 = &[
        Threshold::new("LO VIII - Klasa 1BD-1 (mat-geo-ang)\n", 170.6, "Geografia"),
        Threshold::new(
            "LO VIII - Klasa 1BD-2 (mat-inf-ang)\n",
            170.3,
            "Informatyka",
        ),
    ];

    let g4 = &[
        Threshold::new("LO IX - Klasa 1A (mat-fiz)\n", 161.90, "Fizyka"),
        Threshold::new("LO IX - Klasa 1C (mat-fiz-inf)", 144.90, "Informatyka"),
        Threshold::new("LO IX - Klasa 1E (mat-fiz-inf)\n", 157.80, "WOS"),
    ];

    let g5 = &[Threshold::new(
        "LO X - Klasa 1A (dwujęzyczna politechniczna)\n",
        160.87,
        "Fizyka",
    )];

    let g6 = &[
        Threshold::new("LO XV - Klasa 1A (mat-inf-ang)\n", 147.65, "Fizyka"),
        Threshold::new("LO XV - Klasa 1D (mat-geo-ang)\n", 155.25, "Geografia"),
    ];

    let g7 = &[
        Threshold::new("LO XIX - Klasa 1A (ekonomiczna)\n", 162.70, "Geografia"),
        Threshold::new(
            "LO XIX - Klasa 1B (artystyczna-muzyczna)\n",
            136.20,
            "Historia",
        ),
        Threshold::new("LO XIX - Klasa 1E (mat-fiz-ang)\n", 167.70, "Fizyka"),
    ];

    let k1 = &[Threshold::new(
        "LO I - Klasa 1A (ABAKUS)\n",
        163.85,
        "Fizyka",
    )];

    let k2 = &[Threshold::new(
        "LO II - Klasa 1A (mat-fiz)\n",
        161.90,
        "Fizyka",
    )];
    let k3 = &[Threshold::new(
        "LO III - Klasa 1A (ekonomiczna)\n",
        162.70,
        "Geografia",
    )];

    let p1 = &[
        Threshold::new("LO I - Klasa 1 (ABAKUS)\n", 163.85, "Fizyka"),
        Threshold::new("LO I - Klasa 1 (COLUMBUS)\n", 172.90, "Geografia"),
        Threshold::new("LO I - Klasa 1 (SIGMA)\n", 165.00, "Chemia"),
    ];

    let p2 = &[
        Threshold::new("LO VIII - Klasa 1A (informatyczna)\n", 172.20, "Fizyka"),
        Threshold::new("LO VIII - Klasa 1C (fizyczna)\n", 163.95, "Fizyka"),
        Threshold::new(
            "LO VIII - Klasa 1D grupa 1 (ekonomiczna)\n",
            173.30,
            "Geografia",
        ),
        Threshold::new(
            "LO VIII - Klasa 1D grupa 2 (ekonomiczna)\n",
            163.85,
            "Geografia",
        ),
    ];

    let p3 = &[
        Threshold::new(
            "LO XVI - Klasa 1C (ekonomiczno-informatyczna)\n",
            90.85,
            "Geografia",
        ),
        Threshold::new("LO XVI - Klasa 1D (politechniczna)\n", 126.60, "Fizyka"),
    ];
    let schools_gdansk = vec![
        School::new("LO II Gdańsk", g1),
        School::new("LO III Gdańsk", g2),
        School::new("LO VIII Gdańsk", g3),
        School::new("LO IX Gdańsk", g4),
        School::new("LO X Gdańsk", g5),
        School::new("LO XV Gdańsk", g6),
        School::new("LO XIX Gdańsk", g7),
    ];

    let schools_koszalin = vec![
        School::new("LO I Koszalin", k1),
        School::new("LO II Koszalin", k2),
        School::new("LO III Koszalin", k3),
    ];

    let schools_poznan = vec![
        School::new("LO I im. Karola Marcinkowskiego Poznań", p1),
        School::new("LO VIII Liceum im. Adama Mickiewicza Poznań", p2),
        School::new("LO XVI im. Charlesa de Gaulle Poznań", p3),
    ];
    let cities = [
        City::Gdansk(&schools_gdansk),
        //  City::Koszalin(&schools_koszalin),
        City::Poznan(&schools_poznan),
    ];
    let mut gamestate = SelectionState::None;
    let mut prev_gamestate = SelectionState::None;

    let mut selected_city = 0;
    let mut selected_school = 0;
    let mut selected = 0;
    let mut total_points: f32 = 0.0;

    loop {
        match gamestate {
            SelectionState::Exit => break,
            _ => (),
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
                ui.style_mut().text_styles.insert(
                    egui_macroquad::egui::TextStyle::Button,
                    egui_macroquad::egui::FontId::new(
                        font_size,
                        egui_macroquad::egui::FontFamily::Proportional,
                    ),
                );
                ui.style_mut().text_styles.insert(
                    egui_macroquad::egui::TextStyle::Small,
                    egui_macroquad::egui::FontId::new(
                        font_size,
                        egui_macroquad::egui::FontFamily::Proportional,
                    ),
                );
                match gamestate {
                    SelectionState::None => {
                        let city = &cities[selected_city];
                        gamestate = process_none(
                            ui,
                            font_size,
                            widget_width,
                            widget_height,
                            &mut exam_points,
                            &mut certs,
                            &mut initialization,
                            &prev_gamestate,
                            city,
                            &city.get_schools()[selected_school],
                            &city.get_schools()[selected_school].profiles[selected],
                        );
                        prev_gamestate = SelectionState::None;
                    }
                    SelectionState::City => {
                        gamestate = process_city(
                            ui,
                            font_size,
                            widget_width,
                            widget_height,
                            &cities,
                            &mut selected_city,
                            &mut initialization,
                        );
                        prev_gamestate = SelectionState::City;
                    }
                    SelectionState::School => {
                        gamestate = process_school(
                            ui,
                            font_size,
                            widget_width,
                            widget_height,
                            cities[selected_city].get_schools(),
                            &mut selected_school,
                            &mut initialization,
                        );
                        prev_gamestate = SelectionState::School;
                    }
                    SelectionState::Profil => {
                        gamestate = process_profil(
                            ui,
                            font_size,
                            widget_width,
                            widget_height,
                            cities[selected_city].get_schools()[selected_school].profiles,
                            &mut selected,
                            &mut initialization,
                        );
                        prev_gamestate = SelectionState::Profil;
                    }

                    SelectionState::Exit => (),
                }
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
