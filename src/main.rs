#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release
#![allow(rustdoc::missing_crate_level_docs)] // it's an example

//use eframe::egui;
// Ref: https://www.otouczelnie.pl/kalkulator/osmoklasista

// TODO: zwolnienie z egzaminu

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

fn main() -> Result<(), String> {
    //env_logger::init(); // Log to stderr (if you run with `RUST_LOG=debug`).

    // Our application state:
    let mut name = "Arthur".to_owned();
    let mut age = 42;
    /*
    eframe::run_simple_native("My egui App", options, move |ctx, _frame| {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("My egui Application");
            ui.horizontal(|ui| {
                let name_label = ui.label("Your name: ");
                ui.text_edit_singleline(&mut name)
                    .labelled_by(name_label.id);
            });
            ui.add(egui::Slider::new(&mut age, 0..=120).text("age"));
            if ui.button("Increment").clicked() {
                age += 1;
            }
            ui.label(format!("Hello '{name}', age {age}"));
        });
    })*/
    Ok(())
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
