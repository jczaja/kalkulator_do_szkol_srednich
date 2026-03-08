#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release
#![allow(rustdoc::missing_crate_level_docs)] // it's an example

//use eframe::egui;
// Ref: https://www.otouczelnie.pl/kalkulator/osmoklasista

// I LO Szczecinek : https://cloud-d.edupage.org/cloud/Regulamin_i_harmonogram_rekrutacji_2025_2026_do_I_LO_Szczecinek.pdf?z%3A0mYjQe50qOoFEVT1pUcg5F%2BU1Qs%2BV%2FKMV5Rnq4AztxBI4PNFFctdFVyIc46bQWYEnD07Yx83qP7RLhSDLOMznQ%3D%3D
// XV LO Gdansk : https://lo15.edu.gdansk.pl/Content/pub/452/rekrutacja%202025-26/regulamin_rekrutacji_2025_26.pdf

// punkty https://www.vlo.gda.pl/zasady_przyznawania_punktow/
// https://isap.sejm.gov.pl/isap.nsf/download.xsp/WDU20190001737/O/D20191737.pdf

use egui_plot::{Bar, BarChart, Line, Plot, PlotPoints};
use macroquad::prelude::*; // Import necessary components
                           //
#[derive(PartialEq)]
enum SelectionState {
    None,
    City,
    Contests,
    Contest1, // Konkursy przedmiotowe ponadwojewódzkie
    Contest2, // Konkursy tematyczne ponadwojewódzkie
    Contest3, // Konkursy przedmiotowe wojewódzkie
    Contest4, // Konkursy tematyczne wojewódzkie
    Contest5, // Konkursy artystyczne międzynarodowe/ogólnopolskie
    Contest6, // Konkursy artystyczne ponadwojewódzkie/wojewódzkie
    Contest7, // Konkursy niekuratoryjne
    Exit,
    Profil,
    School,
}

struct School<'a> {
    name: &'a str,
    profiles: &'a [Threshold<'a>],
    min_threashold: f32,
}

impl<'a> School<'a> {
    pub fn new(name: &'a str, profiles: &'a [Threshold<'a>]) -> School<'a> {
        let min_threashold = profiles.iter().fold(200.0, |acc, profil| {
            if acc > profil.points {
                profil.points
            } else {
                acc
            }
        });

        School {
            name,
            profiles,
            min_threashold,
        }
    }

    pub fn get_full_name(&self) -> String {
        format!("{} (minimalny próg: {})", self.name, self.min_threashold)
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

// Struktura przechowująca wszystkie osiągnięcia w konkursach
struct Contest {
    national_subject: ContestNationalSubject, // konkursy przedmiotowe ponadwojewódzkie
    national_thematic: ContestNationalThematic, // konkursy tematyczne ponadwojewódzkie
    regional_subject: ContestRegionalSubject, // konkursy przedmiotowe wojewódzkie
    regional_thematic: ContestRegionalThematic, // konkursy tematyczne wojewódzkie
    artistic_international: ContestArtisticInternational, // konkursy artystyczne międzynarodowe/ogólnopolskie
    artistic_regional: ContestArtisticRegional, // konkursy artystyczne ponadwojewódzkie/wojewódzkie
    noncuratorial: NoncuratorialContest,
}

impl Contest {
    pub fn calculate_points(&self) -> Result<f32, &str> {
        // Laureat konkursu przedmiotowego ogólnopolskiego = automatyczne przyjęcie (200 pkt)
        if self.national_subject == ContestNationalSubject::Laureate {
            return Ok(200.0);
        }

        // W pozostałych przypadkach maksymalnie 18 punktów z osiągnięć
        let total_points = std::cmp::min(
            18,
            self.national_subject.as_u32()
                + self.national_thematic.as_u32()
                + self.regional_subject.as_u32()
                + self.regional_thematic.as_u32()
                + self.artistic_international.as_u32()
                + self.artistic_regional.as_u32()
                + self.noncuratorial.as_u32(),
        );
        Ok(total_points as f32)
    }
}

// 1. Konkursy PRZEDMIOTOWE ponadwojewódzkie (punkt 1a rozporządzenia)
#[derive(PartialEq, Clone, Copy)]
enum ContestNationalSubject {
    None,
    Laureate, // Laureat konkursu przedmiotowego ogólnopolskiego - 200 pkt (automatyczne przyjęcie)
    Finalist, // Finalista konkursu przedmiotowego - 10 pkt
}

impl ContestNationalSubject {
    fn as_u32(&self) -> u32 {
        match self {
            ContestNationalSubject::None => 0,
            ContestNationalSubject::Laureate => 200,
            ContestNationalSubject::Finalist => 10,
        }
    }
}

impl std::fmt::Display for ContestNationalSubject {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ContestNationalSubject::None => write!(f, "Brak"),
            ContestNationalSubject::Laureate => {
                write!(f, "Laureat konkursu przedmiotowego ogólnopolskiego (200 pkt - automatyczne przyjęcie)")
            }
            ContestNationalSubject::Finalist => {
                write!(f, "Finalista konkursu przedmiotowego (10 pkt)")
            }
        }
    }
}

// 2. Konkursy TEMATYCZNE/INTERDYSCYPLINARNE ponadwojewódzkie (punkt 1b,c rozporządzenia)
#[derive(PartialEq, Clone, Copy)]
enum ContestNationalThematic {
    None,
    Laureate, // Laureat konkursu tematycznego/interdyscyplinarnego - 7 pkt
    Finalist, // Finalista konkursu tematycznego/interdyscyplinarnego - 5 pkt
}

impl ContestNationalThematic {
    fn as_u32(&self) -> u32 {
        match self {
            ContestNationalThematic::None => 0,
            ContestNationalThematic::Laureate => 7,
            ContestNationalThematic::Finalist => 5,
        }
    }
}

impl std::fmt::Display for ContestNationalThematic {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ContestNationalThematic::None => write!(f, "Brak"),
            ContestNationalThematic::Laureate => {
                write!(
                    f,
                    "Laureat konkursu tematycznego lub interdyscyplinarnego (7 pkt)"
                )
            }
            ContestNationalThematic::Finalist => {
                write!(
                    f,
                    "Finalista konkursu tematycznego lub interdyscyplinarnego (5 pkt)"
                )
            }
        }
    }
}

// 3. Konkursy PRZEDMIOTOWE wojewódzkie (punkt 3a,d rozporządzenia)
#[derive(PartialEq, Clone, Copy)]
enum ContestRegionalSubject {
    None,
    MultipleFinalist, // 2+ finalistów konkursu przedmiotowego - 10 pkt
    Finalist,         // Finalista konkursu przedmiotowego - 7 pkt
}

impl ContestRegionalSubject {
    fn as_u32(&self) -> u32 {
        match self {
            ContestRegionalSubject::None => 0,
            ContestRegionalSubject::MultipleFinalist => 10,
            ContestRegionalSubject::Finalist => 7,
        }
    }
}

impl std::fmt::Display for ContestRegionalSubject {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ContestRegionalSubject::None => write!(f, "Brak"),
            ContestRegionalSubject::MultipleFinalist => {
                write!(f, "Wielokrotny finalista konkursu przedmiotowego (10 pkt)")
            }
            ContestRegionalSubject::Finalist => {
                write!(f, "Finalista konkursu przedmiotowego (7 pkt)")
            }
        }
    }
}

// 4. Konkursy TEMATYCZNE/INTERDYSCYPLINARNE wojewódzkie (punkt 3b,c,e,f rozporządzenia)
#[derive(PartialEq, Clone, Copy)]
enum ContestRegionalThematic {
    None,
    MultipleLaureate, // 2+ laureatów konkursu tematycznego/interdyscyplinarnego - 7 pkt
    MultipleFinalist, // 2+ finalistów konkursu tematycznego/interdyscyplinarnego - 5 pkt
    Laureate,         // Laureat konkursu tematycznego/interdyscyplinarnego - 5 pkt
    Finalist,         // Finalista konkursu tematycznego/interdyscyplinarnego - 3 pkt
}

impl ContestRegionalThematic {
    fn as_u32(&self) -> u32 {
        match self {
            ContestRegionalThematic::None => 0,
            ContestRegionalThematic::MultipleLaureate => 7,
            ContestRegionalThematic::MultipleFinalist => 5,
            ContestRegionalThematic::Laureate => 5,
            ContestRegionalThematic::Finalist => 3,
        }
    }
}

impl std::fmt::Display for ContestRegionalThematic {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ContestRegionalThematic::None => write!(f, "Brak"),
            ContestRegionalThematic::MultipleLaureate => {
                write!(
                    f,
                    "Wielokrotny laureat konkursu tematycznego/interdyscyplinarnego (7 pkt)"
                )
            }
            ContestRegionalThematic::MultipleFinalist => {
                write!(
                    f,
                    "Wielokrotny finalista konkursu tematycznego/interdyscyplinarnego (5 pkt)"
                )
            }
            ContestRegionalThematic::Laureate => {
                write!(
                    f,
                    "Laureat konkursu tematycznego/interdyscyplinarnego (5 pkt)"
                )
            }
            ContestRegionalThematic::Finalist => {
                write!(
                    f,
                    "Finalista konkursu tematycznego/interdyscyplinarnego (3 pkt)"
                )
            }
        }
    }
}

// 5. Konkursy artystyczne MIĘDZYNARODOWE/OGÓLNOPOLSKIE (punkt 2 rozporządzenia)
#[derive(PartialEq, Clone, Copy)]
enum ContestArtisticInternational {
    None,
    // Objęte ramowym planem nauczania szkoły artystycznej:
    CompetitionFinalistInCurriculum, // Finalista konkursu objętego planem - 10 pkt
    // Nieobjęte ramowym planem nauczania:
    TournamentLaureateOutsideCurriculum, // Laureat turnieju nieobjętego planem - 4 pkt
    TournamentFinalistOutsideCurriculum, // Finalista turnieju nieobjętego planem - 3 pkt
}

impl ContestArtisticInternational {
    fn as_u32(&self) -> u32 {
        match self {
            ContestArtisticInternational::None => 0,
            ContestArtisticInternational::CompetitionFinalistInCurriculum => 10,
            ContestArtisticInternational::TournamentLaureateOutsideCurriculum => 4,
            ContestArtisticInternational::TournamentFinalistOutsideCurriculum => 3,
        }
    }
}

impl std::fmt::Display for ContestArtisticInternational {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ContestArtisticInternational::None => write!(f, "Brak"),
            ContestArtisticInternational::CompetitionFinalistInCurriculum => {
                write!(
                    f,
                    "Finalista konkursu artystycznego objętego planem nauczania (10 pkt)"
                )
            }
            ContestArtisticInternational::TournamentLaureateOutsideCurriculum => {
                write!(
                    f,
                    "Laureat turnieju artystycznego nieobjętego planem nauczania (4 pkt)"
                )
            }
            ContestArtisticInternational::TournamentFinalistOutsideCurriculum => {
                write!(
                    f,
                    "Finalista turnieju artystycznego nieobjętego planem nauczania (3 pkt)"
                )
            }
        }
    }
}

// 6. Konkursy artystyczne PONADWOJEWÓDZKIE/WOJEWÓDZKIE (punkt 4 rozporządzenia)
#[derive(PartialEq, Clone, Copy)]
enum ContestArtisticRegional {
    None,
    // Wielokrotne - objęte planem:
    MultipleCompetitionFinalistInCurriculum, // 2+ finalistów konkursu objętego planem - 10 pkt
    // Wielokrotne - nieobjęte planem:
    MultipleTournamentLaureateOutsideCurriculum, // 2+ laureatów turnieju nieobjętego planem - 7 pkt
    MultipleTournamentFinalistOutsideCurriculum, // 2+ finalistów turnieju nieobjętego planem - 5 pkt
    // Pojedyncze - objęte planem:
    CompetitionFinalistInCurriculum, // Finalista konkursu objętego planem - 7 pkt
    // Pojedyncze - nieobjęte planem:
    TournamentLaureateOutsideCurriculum, // Laureat turnieju nieobjętego planem - 3 pkt
    TournamentFinalistOutsideCurriculum, // Finalista turnieju nieobjętego planem - 2 pkt
}

impl ContestArtisticRegional {
    fn as_u32(&self) -> u32 {
        match self {
            ContestArtisticRegional::None => 0,
            ContestArtisticRegional::MultipleCompetitionFinalistInCurriculum => 10,
            ContestArtisticRegional::MultipleTournamentLaureateOutsideCurriculum => 7,
            ContestArtisticRegional::MultipleTournamentFinalistOutsideCurriculum => 5,
            ContestArtisticRegional::CompetitionFinalistInCurriculum => 7,
            ContestArtisticRegional::TournamentLaureateOutsideCurriculum => 3,
            ContestArtisticRegional::TournamentFinalistOutsideCurriculum => 2,
        }
    }
}

impl std::fmt::Display for ContestArtisticRegional {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ContestArtisticRegional::None => write!(f, "Brak"),
            ContestArtisticRegional::MultipleCompetitionFinalistInCurriculum => {
                write!(f, "Wielokrotny finalista konkursu artystycznego objętego planem nauczania (10 pkt)")
            }
            ContestArtisticRegional::MultipleTournamentLaureateOutsideCurriculum => {
                write!(f, "Wielokrotny laureat turnieju artystycznego nieobjętego planem nauczania (7 pkt)")
            }
            ContestArtisticRegional::MultipleTournamentFinalistOutsideCurriculum => {
                write!(f, "Wielokrotny finalista turnieju artystycznego nieobjętego planem nauczania (5 pkt)")
            }
            ContestArtisticRegional::CompetitionFinalistInCurriculum => {
                write!(
                    f,
                    "Finalista konkursu artystycznego objętego planem nauczania (7 pkt)"
                )
            }
            ContestArtisticRegional::TournamentLaureateOutsideCurriculum => {
                write!(
                    f,
                    "Laureat turnieju artystycznego nieobjętego planem nauczania (3 pkt)"
                )
            }
            ContestArtisticRegional::TournamentFinalistOutsideCurriculum => {
                write!(
                    f,
                    "Finalista turnieju artystycznego nieobjętego planem nauczania (2 pkt)"
                )
            }
        }
    }
}

// Miejsca od 1 do 3 na szczeblu konkursów ogólnopolskich albo międzynarodowych
#[derive(PartialEq, Clone, Copy)]
enum NoncuratorialContest {
    None,
    International,
    National,
    Voidship,
    District,
}

impl NoncuratorialContest {
    fn as_u32(&self) -> u32 {
        match self {
            NoncuratorialContest::None => 0,
            NoncuratorialContest::International => 4,
            NoncuratorialContest::National => 3,
            NoncuratorialContest::Voidship => 2,
            NoncuratorialContest::District => 1,
        }
    }
}

impl std::fmt::Display for NoncuratorialContest {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            NoncuratorialContest::None => write!(f, "Brak"),
            NoncuratorialContest::International => {
                write!(f, "Wysokie miejsce w konkursie międzynarodowym (4 pkt)")
            }
            NoncuratorialContest::National => {
                write!(f, "Wysokie miejsce w konkursie ogólnopolskim (3 pkt)")
            }
            NoncuratorialContest::Voidship => {
                write!(f, "Wysokie miejsce w konkursie wojewódzkim (2 pkt)")
            }
            NoncuratorialContest::District => {
                write!(f, "Wysokie miejsce w konkursie powiatowym (1 pkt)")
            }
        }
    }
}
// Konkursy wojewódzkie kuratoryjne (punkt 3 rozporządzenia)

#[derive(PartialEq, Clone, Copy)]
enum ContestRegionalCuratorial {
    None,
    // Konkursy przedmiotowe wojewódzkie
    MultipleSubjectFinalist, // 2+ finalistów konkursu przedmiotowego - 10 pkt
    SubjectFinalist,         // Finalista konkursu przedmiotowego - 7 pkt
    // Konkursy tematyczne/interdyscyplinarne wojewódzkie
    MultipleThematicLaureate, // 2+ laureatów konkursu tematycznego/interdyscyplinarnego - 7 pkt
    MultipleThematicFinalist, // 2+ finalistów konkursu tematycznego/interdyscyplinarnego - 5 pkt
    ThematicLaureate,         // Laureat konkursu tematycznego/interdyscyplinarnego - 5 pkt
    ThematicFinalist,         // Finalista konkursu tematycznego/interdyscyplinarnego - 3 pkt
}

impl ContestRegionalCuratorial {
    fn as_u32(&self) -> u32 {
        match self {
            ContestRegionalCuratorial::None => 0,
            ContestRegionalCuratorial::MultipleSubjectFinalist => 10,
            ContestRegionalCuratorial::SubjectFinalist => 7,
            ContestRegionalCuratorial::MultipleThematicLaureate => 7,
            ContestRegionalCuratorial::MultipleThematicFinalist => 5,
            ContestRegionalCuratorial::ThematicLaureate => 5,
            ContestRegionalCuratorial::ThematicFinalist => 3,
        }
    }
}

impl std::fmt::Display for ContestRegionalCuratorial {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ContestRegionalCuratorial::None => write!(f, "Brak"),
            ContestRegionalCuratorial::MultipleSubjectFinalist => {
                write!(f, "2+ finalistów konkursu przedmiotowego (10 pkt)")
            }
            ContestRegionalCuratorial::SubjectFinalist => {
                write!(f, "Finalista konkursu przedmiotowego (7 pkt)")
            }
            ContestRegionalCuratorial::MultipleThematicLaureate => {
                write!(
                    f,
                    "2+ laureatów konkursu tematycznego/interdyscyplinarnego (7 pkt)"
                )
            }
            ContestRegionalCuratorial::MultipleThematicFinalist => {
                write!(
                    f,
                    "2+ finalistów konkursu tematycznego/interdyscyplinarnego (5 pkt)"
                )
            }
            ContestRegionalCuratorial::ThematicLaureate => {
                write!(
                    f,
                    "Laureat konkursu tematycznego/interdyscyplinarnego (5 pkt)"
                )
            }
            ContestRegionalCuratorial::ThematicFinalist => {
                write!(
                    f,
                    "Finalista konkursu tematycznego/interdyscyplinarnego (3 pkt)"
                )
            }
        }
    }
}

struct CertificateResults<'a> {
    polish: (u8, &'a str),
    math: (u8, &'a str),
    first_addtional_course: (u8, &'a str),
    second_addtional_course: (u8, &'a str),
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
        {
            Ok(get_course_points(self.polish.0)
                + get_course_points(self.math.0)
                + get_course_points(self.first_addtional_course.0)
                + get_course_points(self.second_addtional_course.0)
                + get_honors_points(self.honors)
                + get_volunteering_points(self.volounteering))
        } else {
            Err("Grades must be between 2 and 6.")
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
    ui.horizontal(|ui| {
        ui.add_space(20.0);
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
    });

    state
}

fn process_contest(
    ui: &mut egui_macroquad::egui::Ui,
    font_size: f32,
    widget_width: f32,
    widget_height: f32,
    schools: &[School],
    contests: &mut Contest,
    initialization: &mut bool,
    prev_gamestate: &SelectionState,
) -> SelectionState {
    let mut state = SelectionState::Contests;

    let set_focus = |widget: &egui_macroquad::egui::Response, initialization: &mut bool| {
        if *initialization {
            // Make focus depending on previous selection state
            widget.request_focus();
            *initialization = false;
        }
    };

    ui.vertical(|ui| {
        // === KONKURSY PRZEDMIOTOWE KURATORYJNE ===
        ui.add(egui_macroquad::egui::Label::new(
            egui_macroquad::egui::RichText::new("Konkursy przedmiotowe kuratoryjne:")
                .size(font_size * 1.2)
                .strong(),
        ));

        ui.horizontal(|ui| {
            // Ponadwojewódzkie
            let contest1_button = ui.add(egui_macroquad::egui::Button::new(
                egui_macroquad::egui::RichText::new(format!(
                    "Ponadwojewódzkie: {}",
                    contests.national_subject.as_u32()
                ))
                .size(font_size),
            ));

            if let SelectionState::Contest1 = prev_gamestate {
                set_focus(&contest1_button, initialization);
            }
            if contest1_button.clicked() {
                state = SelectionState::Contest1;
                *initialization = true;
            };

            // Wojewódzkie
            let contest3_button = ui.add(egui_macroquad::egui::Button::new(
                egui_macroquad::egui::RichText::new(format!(
                    "Wojewódzkie: {}",
                    contests.regional_subject.as_u32()
                ))
                .size(font_size),
            ));

            if let SelectionState::Contest3 = prev_gamestate {
                set_focus(&contest3_button, initialization);
            }
            if contest3_button.clicked() {
                state = SelectionState::Contest3;
                *initialization = true;
            };
        });

        ui.add_space(10.0);

        // === KONKURSY TEMATYCZNE/INTERDYSCYPLINARNE KURATORYJNE ===
        ui.add(egui_macroquad::egui::Label::new(
            egui_macroquad::egui::RichText::new(
                "Konkursy tematyczne/interdyscyplinarne kuratoryjne:",
            )
            .size(font_size * 1.2)
            .strong(),
        ));

        ui.horizontal(|ui| {
            // Ponadwojewódzkie
            let contest2_button = ui.add(egui_macroquad::egui::Button::new(
                egui_macroquad::egui::RichText::new(format!(
                    "Ponadwojewódzkie: {}",
                    contests.national_thematic.as_u32()
                ))
                .size(font_size),
            ));

            if let SelectionState::Contest2 = prev_gamestate {
                set_focus(&contest2_button, initialization);
            }
            if contest2_button.clicked() {
                state = SelectionState::Contest2;
                *initialization = true;
            };

            // Wojewódzkie
            let contest4_button = ui.add(egui_macroquad::egui::Button::new(
                egui_macroquad::egui::RichText::new(format!(
                    "Wojewódzkie: {}",
                    contests.regional_thematic.as_u32()
                ))
                .size(font_size),
            ));

            if let SelectionState::Contest4 = prev_gamestate {
                set_focus(&contest4_button, initialization);
            }
            if contest4_button.clicked() {
                state = SelectionState::Contest4;
                *initialization = true;
            };
        });

        ui.add_space(10.0);

        // === KONKURSY ARTYSTYCZNE ===
        ui.add(egui_macroquad::egui::Label::new(
            egui_macroquad::egui::RichText::new("Konkursy artystyczne:")
                .size(font_size * 1.2)
                .strong(),
        ));

        ui.horizontal(|ui| {
            // Międzynarodowe/ogólnopolskie
            let contest5_button = ui.add(egui_macroquad::egui::Button::new(
                egui_macroquad::egui::RichText::new(format!(
                    "Międzynarodowe/ogólnopolskie: {}",
                    contests.artistic_international.as_u32()
                ))
                .size(font_size),
            ));

            if let SelectionState::Contest5 = prev_gamestate {
                set_focus(&contest5_button, initialization);
            }
            if contest5_button.clicked() {
                state = SelectionState::Contest5;
                *initialization = true;
            };

            // Ponadwojewódzkie/wojewódzkie
            let contest6_button = ui.add(egui_macroquad::egui::Button::new(
                egui_macroquad::egui::RichText::new(format!(
                    "Ponadwojewódzkie/wojewódzkie: {}",
                    contests.artistic_regional.as_u32()
                ))
                .size(font_size),
            ));

            if let SelectionState::Contest6 = prev_gamestate {
                set_focus(&contest6_button, initialization);
            }
            if contest6_button.clicked() {
                state = SelectionState::Contest6;
                *initialization = true;
            };
        });

        ui.add_space(10.0);

        // === KONKURSY NIEKURATORYJNE ===
        ui.add(egui_macroquad::egui::Label::new(
            egui_macroquad::egui::RichText::new("Konkursy niekuratoryjne:")
                .size(font_size * 1.2)
                .strong(),
        ));

        let contest7_button = ui.add(egui_macroquad::egui::Button::new(
            egui_macroquad::egui::RichText::new(format!(
                "Miejsca 1-3 na różnych szczeblach: {}",
                contests.noncuratorial.as_u32()
            ))
            .size(font_size),
        ));

        if let SelectionState::Contest7 = prev_gamestate {
            set_focus(&contest7_button, initialization);
        }
        if contest7_button.clicked() {
            state = SelectionState::Contest7;
            *initialization = true;
        };

        ui.add_space(15.0);

        ui.horizontal(|ui| {
            ui.add_space(20.0);
            let ok_button = ui.add(egui_macroquad::egui::Button::new(
                egui_macroquad::egui::RichText::new(format!("OK")).size(font_size),
            ));
            if *initialization && state == SelectionState::Contests {
                ok_button.request_focus();
                *initialization = false;
            }
            if ok_button.clicked() {
                state = SelectionState::None;
                *initialization = true;
            };
            ui.add_space(20.0);
        });
    });

    state
}

// Contest 1: Konkursy przedmiotowe ponadwojewódzkie
fn process_contest1(
    ui: &mut egui_macroquad::egui::Ui,
    font_size: f32,
    _widget_width: f32,
    _widget_height: f32,
    _schools: &[School],
    contests: &mut Contest,
    initialization: &mut bool,
    _prev_gamestate: &SelectionState,
) -> SelectionState {
    let mut state = SelectionState::Contest1;

    println!("====> contest 1 initialization : {}", initialization);
    ui.vertical(|ui| {
        ui.add(egui_macroquad::egui::Label::new(
            egui_macroquad::egui::RichText::new(format!(
                "Konkursy przedmiotowe ponadwojewódzkie organizowane przez kuratorów oświaty:"
            ))
            .size(font_size),
        ));

        let options = [
            ContestNationalSubject::None,
            ContestNationalSubject::Finalist,
            ContestNationalSubject::Laureate,
        ];

        ui.vertical(|ui| {
            options.into_iter().for_each(|c| {
                ui.radio_value(&mut contests.national_subject, c, format!("{c}"));
            })
        });

        ui.add_space(10.0);

        // Przycisk OK z marginesem po prawej (żeby nie był zakryty przez kamerę)
        ui.horizontal(|ui| {
            ui.add_space(20.0); // margines z lewej
            let ok_button = ui.add(egui_macroquad::egui::Button::new(
                egui_macroquad::egui::RichText::new(format!("OK")).size(font_size),
            ));
            if *initialization {
                println!("====> contest 1 initialization");
                ok_button.request_focus();
                *initialization = false;
            }
            if ok_button.clicked() {
                state = SelectionState::Contests;
                *initialization = true;
            };
            ui.add_space(20.0); // margines z prawej - ważne dla kamery!
        });
    });

    state
}

// Contest 2: Konkursy tematyczne ponadwojewódzkie
fn process_contest2(
    ui: &mut egui_macroquad::egui::Ui,
    font_size: f32,
    _widget_width: f32,
    _widget_height: f32,
    _schools: &[School],
    contests: &mut Contest,
    initialization: &mut bool,
    _prev_gamestate: &SelectionState,
) -> SelectionState {
    let mut state = SelectionState::Contest2;

    ui.vertical(|ui| {
        ui.add(egui_macroquad::egui::Label::new(
            egui_macroquad::egui::RichText::new(format!("Konkursy tematyczne/interdyscyplinarne ponadwojewódzkie organizowane przez kuratorów oświaty:")).size(font_size),
        ));

        let options = [
            ContestNationalThematic::None,
            ContestNationalThematic::Laureate,
            ContestNationalThematic::Finalist,
        ];

        ui.vertical(|ui| {
            options.into_iter().for_each(|c| {
                ui.radio_value(&mut contests.national_thematic, c, format!("{c}"));
            })
        });

        ui.add_space(10.0);

        // Przycisk OK z marginesem po prawej (żeby nie był zakryty przez kamerę)
        ui.horizontal(|ui| {
            ui.add_space(20.0);
            let ok_button = ui.add(egui_macroquad::egui::Button::new(
                egui_macroquad::egui::RichText::new(format!("OK")).size(font_size),
            ));
            if *initialization {
                ok_button.request_focus();
                *initialization = false;
            }
            if ok_button.clicked() {
                state = SelectionState::Contests;
                *initialization = true;
            };
            ui.add_space(20.0);
        });
    });

    state
}

// Contest 3: Konkursy przedmiotowe wojewódzkie
fn process_contest3(
    ui: &mut egui_macroquad::egui::Ui,
    font_size: f32,
    _widget_width: f32,
    _widget_height: f32,
    _schools: &[School],
    contests: &mut Contest,
    initialization: &mut bool,
    _prev_gamestate: &SelectionState,
) -> SelectionState {
    let mut state = SelectionState::Contest3;

    ui.vertical(|ui| {
        ui.add(egui_macroquad::egui::Label::new(
            egui_macroquad::egui::RichText::new(format!(
                "Konkursy przedmiotowe wojewódzkie organizowane przez kuratorów oświaty:"
            ))
            .size(font_size),
        ));

        let options = [
            ContestRegionalSubject::None,
            ContestRegionalSubject::MultipleFinalist,
            ContestRegionalSubject::Finalist,
        ];

        ui.vertical(|ui| {
            options.into_iter().for_each(|c| {
                ui.radio_value(&mut contests.regional_subject, c, format!("{c}"));
            })
        });

        ui.add_space(10.0);

        ui.horizontal(|ui| {
            ui.add_space(20.0);
            let ok_button = ui.add(egui_macroquad::egui::Button::new(
                egui_macroquad::egui::RichText::new(format!("OK")).size(font_size),
            ));
            if *initialization {
                ok_button.request_focus();
                *initialization = false;
            }
            if ok_button.clicked() {
                state = SelectionState::Contests;
                *initialization = true;
            };
            ui.add_space(20.0);
        });
    });

    state
}

// Contest 4: Konkursy tematyczne wojewódzkie
fn process_contest4(
    ui: &mut egui_macroquad::egui::Ui,
    font_size: f32,
    _widget_width: f32,
    _widget_height: f32,
    _schools: &[School],
    contests: &mut Contest,
    initialization: &mut bool,
    _prev_gamestate: &SelectionState,
) -> SelectionState {
    let mut state = SelectionState::Contest4;

    ui.vertical(|ui| {
        ui.add(egui_macroquad::egui::Label::new(
            egui_macroquad::egui::RichText::new(format!("Konkursy tematyczne/interdyscyplinarne wojewódzkie organizowane przez kuratorów oświaty:")).size(font_size),
        ));

        let options = [
            ContestRegionalThematic::None,
            ContestRegionalThematic::MultipleLaureate,
            ContestRegionalThematic::MultipleFinalist,
            ContestRegionalThematic::Laureate,
            ContestRegionalThematic::Finalist,
        ];

        ui.vertical(|ui| {
            options.into_iter().for_each(|c| {
                ui.radio_value(&mut contests.regional_thematic, c, format!("{c}"));
            })
        });

        ui.add_space(10.0);

        ui.horizontal(|ui| {
            ui.add_space(20.0);
            let ok_button = ui.add(egui_macroquad::egui::Button::new(
                egui_macroquad::egui::RichText::new(format!("OK")).size(font_size),
            ));
            if *initialization {
                ok_button.request_focus();
                *initialization = false;
            }
            if ok_button.clicked() {
                state = SelectionState::Contests;
                *initialization = true;
            };
            ui.add_space(20.0);
        });
    });

    state
}

// Contest 5: Konkursy artystyczne międzynarodowe/ogólnopolskie
fn process_contest5(
    ui: &mut egui_macroquad::egui::Ui,
    font_size: f32,
    _widget_width: f32,
    _widget_height: f32,
    _schools: &[School],
    contests: &mut Contest,
    initialization: &mut bool,
    _prev_gamestate: &SelectionState,
) -> SelectionState {
    let mut state = SelectionState::Contest5;

    ui.vertical(|ui| {
        ui.add(egui_macroquad::egui::Label::new(
            egui_macroquad::egui::RichText::new(format!(
                "Konkursy artystyczne międzynarodowe/ogólnopolskie:"
            ))
            .size(font_size),
        ));

        let options = [
            ContestArtisticInternational::None,
            ContestArtisticInternational::CompetitionFinalistInCurriculum,
            ContestArtisticInternational::TournamentLaureateOutsideCurriculum,
            ContestArtisticInternational::TournamentFinalistOutsideCurriculum,
        ];

        ui.vertical(|ui| {
            options.into_iter().for_each(|c| {
                ui.radio_value(&mut contests.artistic_international, c, format!("{c}"));
            })
        });

        ui.add_space(10.0);

        ui.horizontal(|ui| {
            ui.add_space(20.0);
            let ok_button = ui.add(egui_macroquad::egui::Button::new(
                egui_macroquad::egui::RichText::new(format!("OK")).size(font_size),
            ));
            if *initialization {
                ok_button.request_focus();
                *initialization = false;
            }
            if ok_button.clicked() {
                state = SelectionState::Contests;
                *initialization = true;
            };
            ui.add_space(20.0);
        });
    });

    state
}

// Contest 6: Konkursy artystyczne ponadwojewódzkie/wojewódzkie
fn process_contest6(
    ui: &mut egui_macroquad::egui::Ui,
    font_size: f32,
    _widget_width: f32,
    _widget_height: f32,
    _schools: &[School],
    contests: &mut Contest,
    initialization: &mut bool,
    _prev_gamestate: &SelectionState,
) -> SelectionState {
    let mut state = SelectionState::Contest6;

    ui.vertical(|ui| {
        ui.add(egui_macroquad::egui::Label::new(
            egui_macroquad::egui::RichText::new(format!(
                "Konkursy artystyczne ponadwojewódzkie/wojewódzkie:"
            ))
            .size(font_size),
        ));

        let options = [
            ContestArtisticRegional::None,
            ContestArtisticRegional::MultipleCompetitionFinalistInCurriculum,
            ContestArtisticRegional::MultipleTournamentLaureateOutsideCurriculum,
            ContestArtisticRegional::MultipleTournamentFinalistOutsideCurriculum,
            ContestArtisticRegional::CompetitionFinalistInCurriculum,
            ContestArtisticRegional::TournamentLaureateOutsideCurriculum,
            ContestArtisticRegional::TournamentFinalistOutsideCurriculum,
        ];

        ui.vertical(|ui| {
            options.into_iter().for_each(|c| {
                ui.radio_value(&mut contests.artistic_regional, c, format!("{c}"));
            })
        });

        ui.add_space(10.0);

        ui.horizontal(|ui| {
            ui.add_space(20.0);
            let ok_button = ui.add(egui_macroquad::egui::Button::new(
                egui_macroquad::egui::RichText::new(format!("OK")).size(font_size),
            ));
            if *initialization {
                ok_button.request_focus();
                *initialization = false;
            }
            if ok_button.clicked() {
                state = SelectionState::Contests;
                *initialization = true;
            };
            ui.add_space(20.0);
        });
    });

    state
}

// Contest 7: Konkursy niekuratoryjne
fn process_contest7(
    ui: &mut egui_macroquad::egui::Ui,
    font_size: f32,
    _widget_width: f32,
    _widget_height: f32,
    _schools: &[School],
    contests: &mut Contest,
    initialization: &mut bool,
    _prev_gamestate: &SelectionState,
) -> SelectionState {
    let mut state = SelectionState::Contest7;

    ui.vertical(|ui| {
        ui.add(egui_macroquad::egui::Label::new(
            egui_macroquad::egui::RichText::new(format!(
                "Konkursy organizowane przez inne podmioty działające na terenie szkoły: "
            ))
            .size(font_size),
        ));
        let noncuratorials = [
            NoncuratorialContest::None,
            NoncuratorialContest::International,
            NoncuratorialContest::National,
            NoncuratorialContest::Voidship,
            NoncuratorialContest::District,
        ];
        ui.vertical(|ui| {
            noncuratorials.into_iter().for_each(|c| {
                ui.radio_value(&mut contests.noncuratorial, c, format!("{c}"));
            })
        });

        ui.add_space(10.0);

        ui.horizontal(|ui| {
            ui.add_space(20.0);
            let ok_button = ui.add(egui_macroquad::egui::Button::new(
                egui_macroquad::egui::RichText::new(format!("OK")).size(font_size),
            ));
            if *initialization {
                ok_button.request_focus();
                *initialization = false;
            }
            if ok_button.clicked() {
                state = SelectionState::Contests;
                *initialization = true;
            };
            ui.add_space(20.0);
        });
    });

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
            ui.radio_value(
                &mut *selected_school,
                c,
                format!("{}", alt_school.get_full_name()),
            );
        });
    });
    ui.horizontal(|ui| {
        ui.add_space(20.0);
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
    });

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
    ui.horizontal(|ui| {
        ui.add_space(20.0);
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
    });

    state
}

fn process_none(
    ui: &mut egui_macroquad::egui::Ui,
    font_size: f32,
    widget_width: f32,
    widget_height: f32,
    exams: &mut ExamResults,
    certs: &mut CertificateResults,
    contests: &Contest,
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
            let contest_button = ui.add(egui_macroquad::egui::Button::new(
                egui_macroquad::egui::RichText::new(format!(
                    "Konkursy: {}",
                    contests.calculate_points().unwrap()
                ))
                .size(font_size),
            ));

            if let SelectionState::Contests = prev_gamestate {
                set_focus(&contest_button, initialization);
            }
            if contest_button.clicked() {
                state = SelectionState::Contests;
                *initialization = true;
            };

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

                let contests_points = contests.calculate_points().unwrap();

                // No more than 200 points
                total_points = certificate_points + exam_points + contests_points;
                if total_points > 200.0 {
                    total_points = 200.0;
                }
                ui.label(
                    egui_macroquad::egui::RichText::new(format!("Punkty Do Szkoły średniej: "))
                        .size(font_size),
                );
                ui.label(
                    egui_macroquad::egui::RichText::new(format!("{}", total_points))
                        .color(egui_macroquad::egui::Color32::YELLOW)
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
            let target = Line::new(profil.get_full_name(), points)
                .color(egui_macroquad::egui::Color32::BLUE);
            let points_bar = Bar::new(2.0, total_points.into()).width(0.5);
            let barchart =
                BarChart::new("Punkty", vec![points_bar]).color(if total_points >= profil.points {
                    egui_macroquad::egui::Color32::GREEN
                } else {
                    egui_macroquad::egui::Color32::RED
                });

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
    let pol_value: u8 = 69;
    let ang_value: u8 = 100;
    let mat_value: u8 = 60;
    let mut initialization = true;

    let cpol_value: u8 = 4;
    let cang_value: u8 = 6;
    let cmat_value: u8 = 5;
    let cinf_value: u8 = 6;
    let achv_value: u8 = 0;
    let vol_value: bool = true;
    let hon_value: bool = true;

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

    let k1 = &[Threshold::new("Klasa 1A (mat-fiz-inf)", 145.0, "Fizyka")];

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

    let schools_koszalin = vec![School::new("LO I im. St. Dubois Koszalin", k1)];

    let schools_poznan = vec![
        School::new("LO I im. Karola Marcinkowskiego Poznań", p1),
        School::new("LO VIII Liceum im. Adama Mickiewicza Poznań", p2),
        School::new("LO XVI im. Charlesa de Gaulle Poznań", p3),
    ];
    let cities = [
        City::Gdansk(&schools_gdansk),
        City::Koszalin(&schools_koszalin),
        City::Poznan(&schools_poznan),
    ];
    let mut gamestate = SelectionState::None;
    let mut prev_gamestate = SelectionState::None;

    let mut selected_city = 0;
    let mut selected_school = 0;
    let mut selected = 0;
    let mut contests = Contest {
        national_subject: ContestNationalSubject::None,
        national_thematic: ContestNationalThematic::None,
        regional_subject: ContestRegionalSubject::None,
        regional_thematic: ContestRegionalThematic::None,
        artistic_international: ContestArtisticInternational::None,
        artistic_regional: ContestArtisticRegional::None,
        noncuratorial: NoncuratorialContest::None,
    };

    loop {
        match gamestate {
            SelectionState::Exit => {
                #[cfg(target_os = "android")]
                {
                    // This is needed for android to exit without leaving black screen
                    std::process::exit(0);
                }
                #[cfg(not(target_os = "android"))]
                {
                    break;
                }
            }
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
                            &contests,
                            &mut initialization,
                            &prev_gamestate,
                            city,
                            &city.get_schools()[selected_school], // BABOL
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
                        // After reetting city we need to set indices
                        // to school and profil to safe 0 values
                        // otherwise there maybe out of bound error
                        if let SelectionState::None = gamestate {
                            selected_school = 0;
                            selected = 0;
                        }
                    }
                    SelectionState::Contests => {
                        // TODO:
                        gamestate = process_contest(
                            ui,
                            font_size,
                            widget_width,
                            widget_height,
                            cities[selected_city].get_schools(),
                            &mut contests,
                            &mut initialization,
                            &prev_gamestate,
                        );
                        prev_gamestate = SelectionState::Contests;
                    }
                    SelectionState::Contest1 => {
                        // TODO:
                        gamestate = process_contest1(
                            ui,
                            font_size,
                            widget_width,
                            widget_height,
                            cities[selected_city].get_schools(),
                            &mut contests,
                            &mut initialization,
                            &prev_gamestate,
                        );
                        prev_gamestate = SelectionState::Contest1;
                    }
                    SelectionState::Contest2 => {
                        // TODO:
                        gamestate = process_contest2(
                            ui,
                            font_size,
                            widget_width,
                            widget_height,
                            cities[selected_city].get_schools(),
                            &mut contests,
                            &mut initialization,
                            &prev_gamestate,
                        );
                        prev_gamestate = SelectionState::Contest2;
                    }
                    SelectionState::Contest3 => {
                        gamestate = process_contest3(
                            ui,
                            font_size,
                            widget_width,
                            widget_height,
                            cities[selected_city].get_schools(),
                            &mut contests,
                            &mut initialization,
                            &prev_gamestate,
                        );
                        prev_gamestate = SelectionState::Contest3;
                    }
                    SelectionState::Contest4 => {
                        gamestate = process_contest4(
                            ui,
                            font_size,
                            widget_width,
                            widget_height,
                            cities[selected_city].get_schools(),
                            &mut contests,
                            &mut initialization,
                            &prev_gamestate,
                        );
                        prev_gamestate = SelectionState::Contest4;
                    }
                    SelectionState::Contest5 => {
                        gamestate = process_contest5(
                            ui,
                            font_size,
                            widget_width,
                            widget_height,
                            cities[selected_city].get_schools(),
                            &mut contests,
                            &mut initialization,
                            &prev_gamestate,
                        );
                        prev_gamestate = SelectionState::Contest5;
                    }
                    SelectionState::Contest6 => {
                        gamestate = process_contest6(
                            ui,
                            font_size,
                            widget_width,
                            widget_height,
                            cities[selected_city].get_schools(),
                            &mut contests,
                            &mut initialization,
                            &prev_gamestate,
                        );
                        prev_gamestate = SelectionState::Contest6;
                    }
                    SelectionState::Contest7 => {
                        gamestate = process_contest7(
                            ui,
                            font_size,
                            widget_width,
                            widget_height,
                            cities[selected_city].get_schools(),
                            &mut contests,
                            &mut initialization,
                            &prev_gamestate,
                        );
                        prev_gamestate = SelectionState::Contest7;
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
                        if let SelectionState::None = gamestate {
                            selected = 0;
                        }
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
                honors: true,
                volounteering: false,
            }
            .calculate_points(),
            Ok(64.0)
        );

        assert_eq!(
            CertificateResults {
                polish: (2, "jezyk polski"),
                math: (2, "matematyka"),
                first_addtional_course: (2, "jezyk angielski"),
                second_addtional_course: (2, "informatyka"),
                honors: false,
                volounteering: true,
            }
            .calculate_points(),
            Ok(11.0)
        );

        assert_eq!(
            CertificateResults {
                polish: (2, "jezyk polski"),
                math: (1, "matematyka"),
                first_addtional_course: (2, "jezyk angielski"),
                second_addtional_course: (2, "informatyka"),
                honors: false,
                volounteering: true,
            }
            .calculate_points(),
            Err("Grades must be between 2 and 6.")
        );

        Ok(())
    }

    #[test]
    fn test_contest_points() -> Result<(), String> {
        // Test: laureat przedmiotowego ogólnopolskiego = 200 pkt
        let contest = Contest {
            national_subject: ContestNationalSubject::Laureate,
            national_thematic: ContestNationalThematic::None,
            regional_subject: ContestRegionalSubject::None,
            regional_thematic: ContestRegionalThematic::None,
            artistic_international: ContestArtisticInternational::None,
            artistic_regional: ContestArtisticRegional::None,
            noncuratorial: NoncuratorialContest::None,
        };
        assert_eq!(contest.calculate_points(), Ok(200.0f32));

        // Test: brak osiągnięć = 0 pkt
        let contest = Contest {
            national_subject: ContestNationalSubject::None,
            national_thematic: ContestNationalThematic::None,
            regional_subject: ContestRegionalSubject::None,
            regional_thematic: ContestRegionalThematic::None,
            artistic_international: ContestArtisticInternational::None,
            artistic_regional: ContestArtisticRegional::None,
            noncuratorial: NoncuratorialContest::None,
        };
        assert_eq!(contest.calculate_points(), Ok(0.0f32));

        // Test: finalista przedmiotowego ponadwojewódzkiego = 10 pkt
        let contest = Contest {
            national_subject: ContestNationalSubject::Finalist,
            national_thematic: ContestNationalThematic::None,
            regional_subject: ContestRegionalSubject::None,
            regional_thematic: ContestRegionalThematic::None,
            artistic_international: ContestArtisticInternational::None,
            artistic_regional: ContestArtisticRegional::None,
            noncuratorial: NoncuratorialContest::None,
        };
        assert_eq!(contest.calculate_points(), Ok(10.0f32));

        // Test: wielokrotny finalista przedmiotowego wojewódzkiego = 10 pkt
        let contest = Contest {
            national_subject: ContestNationalSubject::None,
            national_thematic: ContestNationalThematic::None,
            regional_subject: ContestRegionalSubject::MultipleFinalist,
            regional_thematic: ContestRegionalThematic::None,
            artistic_international: ContestArtisticInternational::None,
            artistic_regional: ContestArtisticRegional::None,
            noncuratorial: NoncuratorialContest::None,
        };
        assert_eq!(contest.calculate_points(), Ok(10.0f32));

        // Test: finalista przedmiotowego wojewódzkiego = 7 pkt
        let contest = Contest {
            national_subject: ContestNationalSubject::None,
            national_thematic: ContestNationalThematic::None,
            regional_subject: ContestRegionalSubject::Finalist,
            regional_thematic: ContestRegionalThematic::None,
            artistic_international: ContestArtisticInternational::None,
            artistic_regional: ContestArtisticRegional::None,
            noncuratorial: NoncuratorialContest::None,
        };
        assert_eq!(contest.calculate_points(), Ok(7.0f32));

        // Test: konkursy artystyczne międzynarodowe = 10 pkt
        let contest = Contest {
            national_subject: ContestNationalSubject::None,
            national_thematic: ContestNationalThematic::None,
            regional_subject: ContestRegionalSubject::None,
            regional_thematic: ContestRegionalThematic::None,
            artistic_international: ContestArtisticInternational::CompetitionFinalistInCurriculum,
            artistic_regional: ContestArtisticRegional::None,
            noncuratorial: NoncuratorialContest::None,
        };
        assert_eq!(contest.calculate_points(), Ok(10.0f32));

        // Test: kombinacja osiągnięć (max 18 pkt)
        let contest = Contest {
            national_subject: ContestNationalSubject::Finalist, // 10 pkt
            national_thematic: ContestNationalThematic::Laureate, // 7 pkt
            regional_subject: ContestRegionalSubject::None,
            regional_thematic: ContestRegionalThematic::Finalist, // 3 pkt
            artistic_international: ContestArtisticInternational::None,
            artistic_regional: ContestArtisticRegional::None,
            noncuratorial: NoncuratorialContest::None,
        };
        assert_eq!(contest.calculate_points(), Ok(18.0f32)); // 10+7+3 = 20, ale max 18

        Ok(())
    }
}
