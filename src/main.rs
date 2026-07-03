#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release
#![allow(rustdoc::missing_crate_level_docs)] // it's an example

//use eframe::egui;
// Ref: https://www.otouczelnie.pl/kalkulator/osmoklasista

// I LO Szczecinek : https://cloud-d.edupage.org/cloud/Regulamin_i_harmonogram_rekrutacji_2025_2026_do_I_LO_Szczecinek.pdf?z%3A0mYjQe50qOoFEVT1pUcg5F%2BU1Qs%2BV%2FKMV5Rnq4AztxBI4PNFFctdFVyIc46bQWYEnD07Yx83qP7RLhSDLOMznQ%3D%3D
// XV LO Gdansk : https://lo15.edu.gdansk.pl/Content/pub/452/rekrutacja%202025-26/regulamin_rekrutacji_2025_26.pdf

// punkty https://www.vlo.gda.pl/zasady_przyznawania_punktow/
// https://isap.sejm.gov.pl/isap.nsf/download.xsp/WDU20190001737/O/D20191737.pdf
//
// TODO: android storage path does not work

use egui_plot::{Bar, BarChart, Line, Plot, PlotPoints};
use macroquad::prelude::*; // Import necessary components
use serde::{Deserialize, Serialize};
use toml;
use tracing::{instrument, span, Level};

//
#[derive(PartialEq, Debug, Clone)]
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
    Find,
    NotFound,
    Profil(u8),
    School(u8),
    Tutorial(u8),
}

#[derive(Deserialize)]
struct School {
    name: String,
    profiles: Vec<Threshold>,
    min_threashold: f32,
}

trait ReplaceEveryN {
    fn replace_every_char_n(&self, from: char, to: char, n: usize) -> String;
}

impl ReplaceEveryN for str {
    fn replace_every_char_n(&self, from: char, to: char, n: usize) -> String {
        let mut count = 0;
        self.chars()
            .map(|c| {
                if c == from {
                    count += 1;
                    if count % n == 0 {
                        to
                    } else {
                        from
                    }
                } else {
                    c
                }
            })
            .collect::<String>()
    }
}

impl School {
    pub fn new(name: &str, profiles: Vec<Threshold>) -> School {
        let min_threashold = profiles.iter().fold(200.0, |acc, profil| {
            if acc > profil.points {
                profil.points
            } else {
                acc
            }
        });

        School {
            name: name.to_string(),
            profiles,
            min_threashold,
        }
    }

    pub fn get_full_name(&self) -> String {
        format!("{} (minimalny próg: {})", self.name, self.min_threashold)
    }
}

impl std::fmt::Display for School {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // replace every third " " with "\n"
        let displayed_format = self.name.trim().replace_every_char_n(
            " ".chars().next().unwrap(),
            "\n".chars().next().unwrap(),
            3,
        );
        write!(f, "{}", displayed_format)
    }
}

impl PartialEq for School {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
    }
}

#[derive(Deserialize)]
struct Schools {
    schools: Vec<School>,
}

enum City<'a> {
    Gdansk(&'a [School]),
    Koszalin(&'a [School]),
    Poznan(&'a [School]),
    Warszawa(&'a [School]),
}

impl<'a> City<'a> {
    pub fn get_schools(&self) -> &'a [School] {
        match self {
            City::Gdansk(schools) => schools,
            City::Koszalin(schools) => schools,
            City::Poznan(schools) => schools,
            City::Warszawa(schools) => schools,
        }
    }
}

impl<'a> std::fmt::Display for City<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let as_str = match self {
            City::Gdansk(_) => "Gdańsk",
            City::Koszalin(_) => "Koszalin",
            City::Poznan(_) => "Poznań",
            City::Warszawa(_) => "Warszawa",
        };
        write!(f, "{}", as_str)
    }
}

impl<'a> PartialEq for City<'a> {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (City::Gdansk(_), City::Gdansk(_))
            | (City::Koszalin(_), City::Koszalin(_))
            | (City::Warszawa(_), City::Warszawa(_))
            | (City::Poznan(_), City::Poznan(_)) => true,
            _ => false,
        }
    }
}

#[derive(Debug, Deserialize)]
struct Threshold {
    base_name: String,
    points: f32,
    second_course: String,
}
impl Threshold {
    pub fn new(base_name: &str, points: f32, second_course: &str) -> Threshold {
        Threshold {
            base_name: base_name.to_string(),
            points,
            second_course: second_course.to_string(),
        }
    }
    pub fn get_full_name(&self) -> String {
        format!(
            "{} (przedmiot: {}) - {} pkt",
            self.base_name, self.second_course, self.points
        )
    }
}

impl std::fmt::Display for Threshold {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // replace every third " " with "\n"
        let formatted_name = format!(
            "{} (przedmiot: {}) - {} pkt",
            self.base_name.trim(),
            self.second_course,
            self.points
        );
        let displayed_name = formatted_name.replace_every_char_n(
            " ".chars().next().unwrap(),
            "\n".chars().next().unwrap(),
            3,
        );
        write!(f, "{}", displayed_name)
    }
}

#[derive(Serialize, Deserialize)]
struct Config {
    exams: ExamResults,
    contests: Contest,
    certificate: CertificateResults,
    completed_tutorial: bool,
}

// Each tuple is representing score (in percentage) of given exam and name of topic
#[derive(Serialize, Deserialize)]
struct ExamResults {
    polish: (u8, String),
    math: (u8, String),
    second_language: (u8, String),
}

impl ExamResults {
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
#[derive(Serialize, Deserialize)]
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
#[derive(PartialEq, Clone, Copy, Deserialize, Serialize)]
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
#[derive(PartialEq, Clone, Copy, Deserialize, Serialize)]
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
#[derive(PartialEq, Clone, Copy, Deserialize, Serialize)]
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
#[derive(PartialEq, Clone, Copy, Deserialize, Serialize)]
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
#[derive(PartialEq, Clone, Copy, Deserialize, Serialize)]
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
#[derive(PartialEq, Clone, Copy, Deserialize, Serialize)]
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
#[derive(PartialEq, Clone, Copy, Deserialize, Serialize)]
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

#[derive(Clone, Deserialize, Serialize)]
struct CertificateResults {
    polish: (u8, String),
    math: (u8, String),
    first_addtional_course: (u8, String),
    second_addtional_course: (u8, String),
    honors: bool,
    volounteering: bool,
}

impl CertificateResults {
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

fn calculate_points(exams: &ExamResults, certs: &CertificateResults, contests: &Contest) -> f32 {
    let exam_points = exams.calculate_points().unwrap();

    let certificate_points = certs.calculate_points().unwrap();

    let contests_points = contests.calculate_points().unwrap();

    // No more than 200 points
    let mut total_points = certificate_points + exam_points + contests_points;
    if total_points > 200.0 {
        total_points = 200.0;
    }
    total_points
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

fn process_find(
    total_points: f32,
    schools: &[School],
    selected_school: &mut usize,
    selected_profil: &mut usize,
) -> SelectionState {
    let mut state = SelectionState::NotFound;

    let mut candidate_points = 0.0;
    // iterate through schools and profiles
    schools.iter().enumerate().for_each(|(n, s)| {
        s.profiles.iter().enumerate().for_each(|(np, p)| {
            if p.points <= total_points && p.points > candidate_points {
                candidate_points = p.points;
                *selected_school = n;
                *selected_profil = np;
                state = SelectionState::None;
            }
        });
    });

    state
}

fn process_notfound(
    ui: &mut egui_macroquad::egui::Ui,
    font_size: f32,
    widget_width: f32,
    widget_height: f32,
    flash_counter: &mut i32,
) -> SelectionState {
    // First time entering - initialize counter
    if *flash_counter == 0 {
        *flash_counter = 60; // Display message for 60 frames (~1 second)
    }

    // Display large message in the center
    ui.vertical_centered(|ui| {
        ui.add_space(widget_height * 2.0);

        ui.add(egui_macroquad::egui::Label::new(
            egui_macroquad::egui::RichText::new("Brak pasującego profilu i szkoły")
                .size(font_size * 3.0)
                .color(egui_macroquad::egui::Color32::RED),
        ));
    });

    // Decrease counter
    *flash_counter -= 1;

    // When counter reaches 0, return to None state
    if *flash_counter == 0 {
        SelectionState::None
    } else {
        SelectionState::NotFound // Stay in NotFound state
    }
}

fn process_school(
    ui: &mut egui_macroquad::egui::Ui,
    font_size: f32,
    widget_width: f32,
    widget_height: f32,
    schools: &[School],
    slide_num: u8,
    selected_school: &mut usize,
    initialization: &mut bool,
) -> SelectionState {
    let mut state = SelectionState::School(slide_num);

    // get number of schools and print 9 per page
    // calculate number of pages
    const NUM_SCHOOLS_PER_SLIDE: usize = 9;
    // 3  -> (3-1)/9 + 1 = 1
    // 9 -> (9-1)/9 + 1 = 1
    let num_slides: u8 = ((schools.len() - 1) / NUM_SCHOOLS_PER_SLIDE) as u8 + 1;

    let start_offset = NUM_SCHOOLS_PER_SLIDE as u8 * (slide_num - 1);
    let end_offset = std::cmp::min(
        NUM_SCHOOLS_PER_SLIDE as u8 * (slide_num),
        schools.len() as u8,
    );

    //    println!("schools.len(): {} , num_slides: {} start_offset: {} end_offset: {}",schools.len(),num_slides,start_offset,end_offset);

    ui.vertical(|ui| {
        //(NUM_SCHOOLS_PER_SLIDE as u8*num_slides*(slide_num-1)..schools.len() as u8).for_each(|c| {
        (start_offset..end_offset).for_each(|c| {
            let alt_school = &schools[c as usize];
            ui.radio_value(
                &mut *selected_school,
                c as usize,
                format!("{}", alt_school.get_full_name()),
            );
        });
    });
    ui.horizontal(|ui| {
        ui.add_space(ui.available_width() / 2.0 - 0.5 * widget_width);
        let back_button = if slide_num > 1 {
            ui.add(egui_macroquad::egui::Button::new(
                egui_macroquad::egui::RichText::new(format!("<<")).size(font_size),
            ))
        } else {
            ui.add_enabled(
                false,
                egui_macroquad::egui::Button::new(
                    egui_macroquad::egui::RichText::new(format!("<<")).size(font_size),
                ),
            )
        };
        if back_button.clicked() {
            state = SelectionState::School(if slide_num > 1 { slide_num - 1 } else { 1 });
        };

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
        let forward_button = if slide_num < num_slides {
            ui.add(egui_macroquad::egui::Button::new(
                egui_macroquad::egui::RichText::new(format!(">>")).size(font_size),
            ))
        } else {
            ui.add_enabled(
                false,
                egui_macroquad::egui::Button::new(
                    egui_macroquad::egui::RichText::new(format!(">>")).size(font_size),
                ),
            )
        };

        if forward_button.clicked() {
            state = SelectionState::School(if slide_num < num_slides {
                slide_num + 1
            } else {
                num_slides
            });
        };
    });

    state
}

//[[licea]]
//szkola = "I LO im. Mikołaja Kopernika"
//klasa = "Klasa akademicka / politechniczna"
//rozszerzenia = "mat-fiz"
//przedmioty_rekrutacja = ["język polski", "matematyka", "fizyka", "język obcy nowożytny"]
//prog_2025 = 170.75
//prog_2024 = 169.2
#[derive(Serialize, Deserialize, Debug)]
struct Liceum {
    szkola: String,
    klasa: String,
    rozszerzenia: String,
    przedmioty_rekrutacja: Vec<String>,
    prog_2025: f32,
    prog_2024: f32,
}

#[derive(Serialize, Deserialize, Debug)]
struct Data {
    licea: Vec<Liceum>,
}

fn get_profiles_data(profiles_str: &str) -> Vec<School> {
    // Fetch the data and deserialize
    let data: Data = toml::from_str(profiles_str).expect("Unable to parse config");
    // Transform data into structures
    // Threshold
    let mut profiles_map: std::collections::HashMap<String, Vec<Threshold>> = data
        .licea
        .into_iter()
        .map(|l| {
            let extra_subjects = l
                .przedmioty_rekrutacja
                .iter()
                .filter(|&p| {
                    p != "język polski" && p != "matematyka" && p != "język obcy nowożytny"
                })
                .collect::<Vec<&String>>();

            // If among extra subject we have Geo and Wos then pick WOS
            // if there is Inf and Fix then pick Inf
            // other wise pick first one

            let subject = extra_subjects
                .iter()
                .find(|s| **s == "wiedza o społeczeństwie" || **s == "informatyka")
                .unwrap_or(&extra_subjects[0]);
            let myth = Threshold::new(&l.szkola, l.prog_2025, subject);
            (l.szkola, myth)
        })
        .fold(
            std::collections::HashMap::new(),
            |mut acc, (szkola, myth)| {
                acc.entry(szkola).or_insert_with(Vec::new).push(myth);
                acc
            },
        );
    // used by this program
    // Make profiles gathered per school

    profiles_map
        .into_iter()
        .map(|(name, profiles)| School::new(&name, profiles))
        .collect::<Vec<School>>()
}

fn save_config(
    certs: CertificateResults,
    contests: Contest,
    exams: ExamResults,
    completed_tutorial: bool,
) -> Result<(), String> {
    let config = Config {
        certificate: certs,
        contests,
        exams,
        completed_tutorial,
    };
    let toml_string = toml::to_string(&config).unwrap();
    let storage_dir = get_storage_dir();
    let mut storage = storage_dir.clone();
    storage.push("config.toml");
    std::fs::create_dir_all(&storage_dir)
        .map_err(|e| format!("Unable to create storage dir : {storage_dir:?} Error: {e}"))?;
    std::fs::write(&storage, toml_string)
        .map_err(|e| format!("Unable to write config into : {storage:?} Error : {e}"))?;
    tracing::info!("Config Saved");
    Ok(())
}

fn get_config() -> Result<(CertificateResults, Contest, ExamResults, bool), String> {
    let storage_dir = get_storage_dir();
    let mut storage = storage_dir.clone();
    storage.push("config.toml");
    let maybe_content = std::fs::read_to_string(&storage).ok(); // Read config
    tracing::info!("After storage read");
    match maybe_content {
        Some(content) => {
            tracing::info!("Some(content)");
            let config: Config =
                toml::from_str(&content.clone()).map_err(|e| "Unable to parse config")?;
            tracing::info!("Config loaded and deserialized from storage.");
            Ok((
                config.certificate,
                config.contests,
                config.exams,
                config.completed_tutorial,
            ))
        }
        None => {
            tracing::info!("None");
            let exam_results = ExamResults {
                polish: (50, "Polish".to_string()),
                math: (50, "Math".to_owned()),
                second_language: (50, "English".to_owned()),
            };
            let certs = CertificateResults {
                polish: (3, "jezyk polski".to_owned()),
                math: (3, "matematyka".to_owned()),
                first_addtional_course: (3, "jezyk angielski".to_owned()),
                second_addtional_course: (3, "informatyka".to_owned()),
                honors: false,
                volounteering: false,
            };

            let contests = Contest {
                national_subject: ContestNationalSubject::None,
                national_thematic: ContestNationalThematic::None,
                regional_subject: ContestRegionalSubject::None,
                regional_thematic: ContestRegionalThematic::None,
                artistic_international: ContestArtisticInternational::None,
                artistic_regional: ContestArtisticRegional::None,
                noncuratorial: NoncuratorialContest::None,
            };
            let config = Config {
                exams: exam_results,
                contests,
                certificate: certs,
                completed_tutorial: false,
            };
            tracing::info!("Before serializing");
            let toml_string =
                toml::to_string(&config).map_err(|_| format!("Failed to serialize config"))?;
            tracing::info!("Before storage.set");

            std::fs::create_dir_all(&storage_dir).map_err(|e| {
                format!("Unable to create storage dir : {storage_dir:?} Error: {e}")
            })?;
            std::fs::write(&storage, toml_string)
                .map_err(|e| format!("Unable to write config into : {storage:?} Error : {e}"))?;

            tracing::info!("Config serialized and saved to storage.");
            Ok((
                config.certificate,
                config.contests,
                config.exams,
                config.completed_tutorial,
            ))
        }
    }
}

fn process_profil(
    ui: &mut egui_macroquad::egui::Ui,
    font_size: f32,
    widget_width: f32,
    widget_height: f32,
    profils: &[Threshold],
    slide_num: u8,
    selected_profil: &mut usize,
    initialization: &mut bool,
) -> SelectionState {
    let mut state = SelectionState::Profil(slide_num);

    // get number of schools and print 9 per page
    // calculate number of pages
    const NUM_PROFILS_PER_SLIDE: usize = 9;
    // 3  -> (3-1)/9 + 1 = 1
    // 9 -> (9-1)/9 + 1 = 1
    let num_slides: u8 = ((profils.len() - 1) / NUM_PROFILS_PER_SLIDE) as u8 + 1;

    let start_offset = NUM_PROFILS_PER_SLIDE as u8 * (slide_num - 1);
    let end_offset = std::cmp::min(
        NUM_PROFILS_PER_SLIDE as u8 * (slide_num),
        profils.len() as u8,
    );

    //    println!("profils.len(): {} , num_slides: {} start_offset: {} end_offset: {}",profils.len(),num_slides,start_offset,end_offset);
    //
    ui.vertical(|ui| {
        (start_offset..end_offset).for_each(|c| {
            let alt_profil: &Threshold = &profils[c as usize];
            ui.radio_value(
                &mut *selected_profil,
                c as usize,
                format!("{}", alt_profil.get_full_name()),
            );
        });
    });
    ui.horizontal(|ui| {
        //ui.add_space(20.0);
        ui.add_space(ui.available_width() / 2.0 - 0.5 * widget_width);

        let back_button = if slide_num > 1 {
            ui.add(egui_macroquad::egui::Button::new(
                egui_macroquad::egui::RichText::new(format!("<<")).size(font_size),
            ))
        } else {
            ui.add_enabled(
                false,
                egui_macroquad::egui::Button::new(
                    egui_macroquad::egui::RichText::new(format!("<<")).size(font_size),
                ),
            )
        };
        if back_button.clicked() {
            state = SelectionState::Profil(if slide_num > 1 { slide_num - 1 } else { 1 });
        };

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
        let forward_button = if slide_num < num_slides {
            ui.add(egui_macroquad::egui::Button::new(
                egui_macroquad::egui::RichText::new(format!(">>")).size(font_size),
            ))
        } else {
            ui.add_enabled(
                false,
                egui_macroquad::egui::Button::new(
                    egui_macroquad::egui::RichText::new(format!(">>")).size(font_size),
                ),
            )
        };

        if forward_button.clicked() {
            state = SelectionState::Profil(if slide_num < num_slides {
                slide_num + 1
            } else {
                num_slides
            });
        };
    });

    tracing::info!("Selected profil: {}", selected_profil);
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
                total_points = calculate_points(&exams, &certs, &contests);
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
                ui.add_space(20.0);
                let find_button = ui.add(egui_macroquad::egui::Button::new(
                    egui_macroquad::egui::RichText::new(format!("Zaproponuj\nprofil"))
                        .size(font_size),
                ));
                if let SelectionState::Find = prev_gamestate {
                    set_focus(&find_button, initialization);
                }
                if find_button.clicked() {
                    state = SelectionState::Find;
                    *initialization = true;
                };
            });

            ui.horizontal(|ui| {
                ui.add(egui_macroquad::egui::Label::new(
                    egui_macroquad::egui::RichText::new("Szkoła: ".to_string()).size(font_size),
                ));

                let school_button = ui.add(
                    egui_macroquad::egui::Button::new(
                        egui_macroquad::egui::RichText::new(format!("{school}")).size(font_size),
                    )
                    .wrap(),
                );

                if let SelectionState::School(_) = prev_gamestate {
                    set_focus(&school_button, initialization);
                }
                // If we clicked school then we
                // show list of schools from first slide
                if school_button.clicked() {
                    state = SelectionState::School(1);
                    *initialization = true;
                };
            });

            ui.horizontal(|ui| {
                ui.add(egui_macroquad::egui::Label::new(
                    egui_macroquad::egui::RichText::new(format!("Profil: ")).size(font_size),
                ));
                let profil_button = ui.add(egui_macroquad::egui::Button::new(
                    egui_macroquad::egui::RichText::new(format!("{}", profil)).size(font_size),
                ));

                if let SelectionState::Profil(_) = prev_gamestate {
                    set_focus(&profil_button, initialization);
                }
                if profil_button.clicked() {
                    state = SelectionState::Profil(1);
                    *initialization = true;
                };
            });

            ui.horizontal(|ui| {
                if ui
                    .add(egui_macroquad::egui::Button::new(
                        egui_macroquad::egui::RichText::new(format!("Poradnik")).size(font_size),
                    ))
                    .clicked()
                {
                    state = SelectionState::Tutorial(1);
                };

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

fn process_tutorial(
    ui: &mut egui_macroquad::egui::Ui,
    font_size: f32,
    widget_width: f32,
    widget_height: f32,
    initialization: &mut bool,
    slide_num: u8,
) -> SelectionState {
    let mut gamestate = SelectionState::Tutorial(slide_num);
    const NUM_SLIDES: u8 = 5;

    let slide = match slide_num {
        1 => egui_macroquad::egui::include_image!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/assets/egzaminy.png"
        )),
        2 => egui_macroquad::egui::include_image!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/assets/swiadectwo.png"
        )),
        3 => egui_macroquad::egui::include_image!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/assets/konkursy.png"
        )),
        4 => egui_macroquad::egui::include_image!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/assets/szkoly.png"
        )),
        5 => egui_macroquad::egui::include_image!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/assets/wyniki.png"
        )),
        _ => panic!("Error: request for index of not existing picture"),
    };

    ui.vertical_centered(|ui| {
        ui.add(
            egui_macroquad::egui::Image::new(slide)
                .max_size(egui_macroquad::egui::vec2(
                    ui.available_width(),
                    ui.available_height() - 2.0 * widget_height,
                ))
                .corner_radius(5),
        );

        ui.add(egui_macroquad::egui::Label::new(
            egui_macroquad::egui::RichText::new(format!("Slajd:: {slide_num}/{NUM_SLIDES}"))
                .size(font_size * 1.2)
                .strong(),
        ));

        ui.horizontal(|ui| {
            ui.add_space(ui.available_width() / 2.0 - 0.5 * widget_width);
            let back_button = if slide_num > 1 {
                ui.add(egui_macroquad::egui::Button::new(
                    egui_macroquad::egui::RichText::new(format!("<<")).size(font_size),
                ))
            } else {
                ui.add_enabled(
                    false,
                    egui_macroquad::egui::Button::new(
                        egui_macroquad::egui::RichText::new(format!("<<")).size(font_size),
                    ),
                )
            };
            if back_button.clicked() {
                gamestate = SelectionState::Tutorial(if slide_num > 1 { slide_num - 1 } else { 1 });
            };

            let rozpocznij_button = ui.add(egui_macroquad::egui::Button::new(
                egui_macroquad::egui::RichText::new(format!("Rozpocznij")).size(font_size),
            ));
            if *initialization {
                rozpocznij_button.request_focus();
                *initialization = false;
            }

            if rozpocznij_button.clicked() {
                gamestate = SelectionState::None;
                *initialization = true;
            };

            let forward_button = if slide_num < 5 {
                ui.add(egui_macroquad::egui::Button::new(
                    egui_macroquad::egui::RichText::new(format!(">>")).size(font_size),
                ))
            } else {
                ui.add_enabled(
                    false,
                    egui_macroquad::egui::Button::new(
                        egui_macroquad::egui::RichText::new(format!(">>")).size(font_size),
                    ),
                )
            };

            if forward_button.clicked() {
                gamestate = SelectionState::Tutorial(if slide_num < NUM_SLIDES {
                    slide_num + 1
                } else {
                    NUM_SLIDES
                });
            };
        });
    });
    gamestate
}

fn get_screenshot() {
    if is_key_pressed(KeyCode::S) {
        println!("====> Saving screenshot...");
        let image = macroquad::prelude::get_screen_data();
        image.export_png("screenshot.png");
    }
}

fn tablet7_window_conf() -> Conf {
    Conf {
        window_title: "kalkulator punktów do szkoły średniej".to_owned(),
        window_height: 800,
        window_width: 1280,
        window_resizable: false,
        ..Default::default()
    }
}
fn tablet10_window_conf() -> Conf {
    Conf {
        window_title: "kalkulator punktów do szkoły średniej".to_owned(),
        window_height: 1200,
        window_width: 1920,
        window_resizable: false,
        ..Default::default()
    }
}

fn init_tracing() {
    #[cfg(feature = "tracy")]
    {
        use tracing_subscriber::{layer::SubscriberExt, Registry};
        use tracing_tracy::TracyLayer;
        // Tracy subscriber
        tracing::subscriber::set_global_default(
            tracing_subscriber::registry().with(tracing_tracy::TracyLayer::default()),
        )
        .expect("setup tracy layer");
    }
    #[cfg(all(feature = "subscriber", not(feature = "tracy")))]
    {
        use tracing_subscriber::{filter::EnvFilter, fmt};
        fmt().with_env_filter(EnvFilter::from_default_env()).init();
    }
    tracing::info!("Tracing started!");
}

fn get_storage_dir() -> std::path::PathBuf {
    let mut storage_dir = std::path::PathBuf::default();
    // For android let's get internal cache of app
    #[cfg(target_os = "android")]
    {
        storage_dir = std::env::temp_dir()
    }
    // For other platforms (linux) XDG config path fallbacking to home is our option
    #[cfg(not(target_os = "android"))]
    {
        // 1. First check XDG_CONFIG_HOME
        if let Some(xdg_config) = std::env::var_os("XDG_CONFIG_HOME").map(std::path::PathBuf::from)
        {
            storage_dir = xdg_config;
            storage_dir.push("kalkulator_do_szkol_srednich");
        }
        // 2. If no XDG_CONFIG_HOME then lets build our own
        else if let Some(mut home) = std::env::var_os("HOME").map(std::path::PathBuf::from) {
            storage_dir.push(home);
            storage_dir.push(".config");
            storage_dir.push("kalkulator_do_szkol_srednich");
        }
        // 3. No HOME? then use current working directory
        else {
            storage_dir = std::path::PathBuf::from(".")
        }
    }
    tracing::info!("Storage Dir : {storage_dir:?}");
    storage_dir
}

//#[macroquad::main(tablet10_window_conf)]
#[macroquad::main("kalkulator punktów do szkoły średniej")]
async fn main() {
    let _guard = init_tracing();
    let mut initialization = true;

    let maybe_config = get_config();
    let (mut certs, mut contests, mut exam_points, mut completed_tutorial) = match maybe_config {
        Ok((certs, contests, exam_points, completed_tutorial)) => {
            (certs, contests, exam_points, completed_tutorial)
        }
        Err(e) => {
            tracing::error!("Error: {e}");
            println!("Error: {e}");
            return;
        }
    };
    tracing::info!("Config Loaded");

    let load_data = || -> Result<(Schools, Schools, Schools), &str> {
        // TODO: how to make this error reported via tracing in elegant way
        let schools_gdansk: Schools = toml::from_str(include_str!("../assets/gdansk.toml"))
            .map_err(|_| "Unable to load gdansk.toml")?;
        let schools_poznan: Schools = toml::from_str(include_str!("../assets/poznan.toml"))
            .map_err(|_| "Unable to load poznan.toml")?;
        let schools_warszawa: Schools = toml::from_str(include_str!("../assets/warszawa.toml"))
            .map_err(|_| "Unable to load warszawa.toml")?;
        Ok((schools_gdansk, schools_poznan, schools_warszawa))
    };

    let (schools_gdansk, schools_poznan, schools_warszawa) = match load_data() {
        Ok((schools_gdansk, schools_poznan, schools_warszawa)) => {
            (schools_gdansk, schools_poznan, schools_warszawa)
        }
        Err(e) => {
            tracing::error!("Error: {e}");
            panic!("{e}");
        }
    };

    let k1 = vec![Threshold::new("Klasa 1A (mat-fiz-inf)", 145.0, "Fizyka")];
    let schools_koszalin = vec![School::new("LO I im. St. Dubois Koszalin", k1)];

    let cities = [
        City::Gdansk(&schools_gdansk.schools),
        City::Koszalin(&schools_koszalin),
        City::Poznan(&schools_poznan.schools),
        City::Warszawa(&schools_warszawa.schools),
    ];
    tracing::info!("Profiles data loaded");

    let mut gamestate = if completed_tutorial {
        SelectionState::None
    } else {
        SelectionState::Tutorial(1)
    };
    let mut prev_gamestate = SelectionState::None;

    let mut selected_city = 0;
    let mut selected_school = 0;
    let mut selected = 0;

    // Counter for red flash effect
    let mut red_flash_counter = 0;

    loop {
        match gamestate {
            SelectionState::Exit => {
                match save_config(certs, contests, exam_points, completed_tutorial) {
                    Ok(_) => (),
                    Err(e) => tracing::error!({ e }),
                }

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
                egui_extras::install_image_loaders(egui_ctx);
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
                        completed_tutorial = true;
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
                    SelectionState::Find => {
                        gamestate = process_find(
                            calculate_points(&exam_points, &certs, &contests),
                            cities[selected_city].get_schools(),
                            &mut selected_school,
                            &mut selected,
                        );
                        prev_gamestate = SelectionState::Find;
                    }
                    SelectionState::NotFound => {
                        gamestate = process_notfound(
                            ui,
                            font_size,
                            widget_width,
                            widget_height,
                            &mut red_flash_counter,
                        );
                        // Focus should be on propose profil
                        if gamestate == SelectionState::None {
                            prev_gamestate = SelectionState::Find;
                        }
                    }
                    SelectionState::School(part) => {
                        gamestate = process_school(
                            ui,
                            font_size,
                            widget_width,
                            widget_height,
                            cities[selected_city].get_schools(),
                            part,
                            &mut selected_school,
                            &mut initialization,
                        );
                        if let SelectionState::School(_) = gamestate {
                            prev_gamestate = gamestate.clone();
                        } else if let SelectionState::None = gamestate {
                            prev_gamestate = SelectionState::School(1);
                            selected = 0;
                        }
                    }
                    SelectionState::Profil(part) => {
                        gamestate = process_profil(
                            ui,
                            font_size,
                            widget_width,
                            widget_height,
                            &cities[selected_city].get_schools()[selected_school].profiles,
                            part,
                            &mut selected,
                            &mut initialization,
                        );
                        if let SelectionState::Profil(_) = gamestate {
                            prev_gamestate = gamestate.clone();
                        } else if let SelectionState::None = gamestate {
                            prev_gamestate = SelectionState::Profil(1);
                        }
                    }
                    SelectionState::Tutorial(slide_num) => {
                        gamestate = process_tutorial(
                            ui,
                            font_size,
                            widget_width,
                            widget_height,
                            &mut initialization,
                            slide_num,
                        );
                        prev_gamestate = SelectionState::Tutorial(slide_num);
                    }

                    SelectionState::Exit => (),
                }
            });
        });

        // Draw things before egui

        egui_macroquad::draw();

        // Draw things after egui
        get_screenshot();

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
                polish: (100, "Polish".to_owned()),
                math: (100, "Math".to_owned()),
                second_language: (100, "English".to_owned())
            }
            .calculate_points(),
            Ok(100.0)
        );
        assert_eq!(
            ExamResults {
                polish: (110, "Polish".to_owned()),
                math: (100, "Math".to_owned()),
                second_language: (100, "English".to_owned())
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
                polish: (6, "jezyk polski".to_owned()),
                math: (5, "matematyka".to_owned()),
                first_addtional_course: (4, "jezyk angielski".to_owned()),
                second_addtional_course: (3, "informatyka".to_owned()),
                honors: true,
                volounteering: false,
            }
            .calculate_points(),
            Ok(64.0)
        );

        assert_eq!(
            CertificateResults {
                polish: (2, "jezyk polski".to_owned()),
                math: (2, "matematyka".to_owned()),
                first_addtional_course: (2, "jezyk angielski".to_owned()),
                second_addtional_course: (2, "informatyka".to_owned()),
                honors: false,
                volounteering: true,
            }
            .calculate_points(),
            Ok(11.0)
        );

        assert_eq!(
            CertificateResults {
                polish: (2, "jezyk polski".to_owned()),
                math: (1, "matematyka".to_owned()),
                first_addtional_course: (2, "jezyk angielski".to_owned()),
                second_addtional_course: (2, "informatyka".to_owned()),
                honors: false,
                volounteering: true,
            }
            .calculate_points(),
            Err("Grades must be between 2 and 6.")
        );

        Ok(())
    }

    #[test]
    fn test_process_find() -> Result<(), String> {
        let g1 = vec![Threshold::new(
            "Klasa 1A (politechniczna)\n",
            168.75,
            "Fizyka",
        )];
        let g2 = vec![
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

        let g3 = vec![
            Threshold::new("LO VIII - Klasa 1BD-1 (mat-geo-ang)\n", 170.6, "Geografia"),
            Threshold::new(
                "LO VIII - Klasa 1BD-2 (mat-inf-ang)\n",
                170.3,
                "Informatyka",
            ),
        ];

        let g4 = vec![
            Threshold::new("LO IX - Klasa 1A (mat-fiz)\n", 161.90, "Fizyka"),
            Threshold::new("LO IX - Klasa 1C (mat-fiz-inf)", 144.90, "Informatyka"),
            Threshold::new("LO IX - Klasa 1E (mat-fiz-inf)\n", 157.80, "WOS"),
        ];

        let g5 = vec![Threshold::new(
            "LO X - Klasa 1A (dwujęzyczna politechniczna)\n",
            160.87,
            "Fizyka",
        )];

        let g6 = vec![
            Threshold::new("LO XV - Klasa 1A (mat-inf-ang)\n", 147.65, "Fizyka"),
            Threshold::new("LO XV - Klasa 1D (mat-geo-ang)\n", 155.25, "Geografia"),
        ];

        let g7 = vec![
            Threshold::new("LO XIX - Klasa 1A (ekonomiczna)\n", 162.70, "Geografia"),
            Threshold::new(
                "LO XIX - Klasa 1B (artystyczna-muzyczna)\n",
                136.20,
                "Historia",
            ),
            Threshold::new("LO XIX - Klasa 1E (mat-fiz-ang)\n", 167.70, "Fizyka"),
        ];
        // Full
        let schools_gdansk = vec![
            School::new("LO II Gdańsk", g1),
            School::new("LO III Gdańsk", g2),
            School::new("LO VIII Gdańsk", g3),
            School::new("LO IX Gdańsk", g4),
            School::new("LO X Gdańsk", g5),
            School::new("LO XV Gdańsk", g6),
            School::new("LO XIX Gdańsk", g7),
        ];

        let mut selected_school = 0;
        let mut selected_profil = 0;

        let gamestate = process_find(
            150.0,
            &schools_gdansk,
            &mut selected_school,
            &mut selected_profil,
        );

        println!(
            "Winner: {}  : {:?}",
            schools_gdansk[selected_school],
            schools_gdansk[selected_school].profiles[selected_profil]
        );

        assert_eq!(gamestate, SelectionState::None);

        // With 150 points we should get g6 (X LO , mat-inf-ang)
        assert_eq!(selected_school, 5);
        assert_eq!(selected_profil, 0);

        Ok(())
    }

    #[test]
    fn test_calculate_points() -> Result<(), String> {
        let contest = Contest {
            national_subject: ContestNationalSubject::Laureate,
            national_thematic: ContestNationalThematic::None,
            regional_subject: ContestRegionalSubject::None,
            regional_thematic: ContestRegionalThematic::None,
            artistic_international: ContestArtisticInternational::None,
            artistic_regional: ContestArtisticRegional::None,
            noncuratorial: NoncuratorialContest::None,
        };
        let certs = CertificateResults {
            polish: (6, "jezyk polski".to_owned()),
            math: (5, "matematyka".to_owned()),
            first_addtional_course: (4, "jezyk angielski".to_owned()),
            second_addtional_course: (3, "informatyka".to_owned()),
            honors: true,
            volounteering: false,
        };
        let exams = ExamResults {
            polish: (100, "Polish".to_owned()),
            math: (100, "Math".to_owned()),
            second_language: (100, "English".to_owned()),
        };
        assert_eq!(calculate_points(&exams, &certs, &contest), 200.0);
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
