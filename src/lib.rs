use bdsp_ug_generator::personal_info_bdsp::PersonalInfoBDSP;
use bdsp_ug_generator::statues::{get_statue_data, Statue, StatueConfig};
use bdsp_ug_generator::xorshift::XorShift;
use bdsp_ug_generator::{
    available_pokemon, get_available_egg_moves, personal_table, run_results, Filter, RoomType,
    Version,
};
use eframe::egui::{Context, Visuals};
use eframe::{egui, CreationContext, Frame};
use egui_extras::{Size, TableBuilder};
use lazy_static::lazy_static;

const GENDER_SYMBOLS: [&str; 3] = ["♂", "♀", "-"];
const SPECIES_EN_RAW: &str = include_str!("../resources/text/other/en/species_en.txt");
const ABILITIES_EN_RAW: &str = include_str!("../resources/text/other/en/abilities_en.txt");
const NATURES_EN_RAW: &str = include_str!("../resources/text/other/en/natures_en.txt");
const MOVES_EN_RAW: &str = include_str!("../resources/text/other/en/moves_en.txt");
const ITEMS_EN_RAW: &str = include_str!("../resources/text/items/items_en.txt");

lazy_static! {
    pub static ref SPECIES_EN: Vec<&'static str> = load_string_list(SPECIES_EN_RAW);
    pub static ref ABILITIES_EN: Vec<&'static str> = load_string_list(ABILITIES_EN_RAW);
    pub static ref NATURES_EN: Vec<&'static str> = load_string_list(NATURES_EN_RAW);
    pub static ref MOVES_EN: Vec<&'static str> = load_string_list(MOVES_EN_RAW);
    pub static ref ITEMS_EN: Vec<&'static str> = load_string_list(ITEMS_EN_RAW);
}

fn load_string_list(list: &str) -> Vec<&str> {
    list.split('\n')
        .map(|s| {
            if s.is_empty() {
                s
            } else if s.as_bytes()[s.len() - 1] == b'\r' {
                &s[..(s.len() - 1)]
            } else {
                s
            }
        })
        .collect()
}

#[derive(PartialOrd, PartialEq, Copy, Clone, Default)]
#[repr(u8)]
enum StoryFlag {
    UndergroundUnlocked = 1,
    StrengthObtained,
    DefogObtained,
    SevenBadges,
    WaterfallObtained,
    #[default]
    NationalDex,
}

impl StoryFlag {
    fn get_str(&self) -> &'static str {
        match self {
            StoryFlag::UndergroundUnlocked => "Underground Unlocked",
            StoryFlag::StrengthObtained => "Strength Obtained",
            StoryFlag::DefogObtained => "Defog Obtained",
            StoryFlag::SevenBadges => "7 Badges",
            StoryFlag::WaterfallObtained => "Waterfall Obtained",
            StoryFlag::NationalDex => "National Dex",
        }
    }
}

pub struct BDSPUgGeneratorUI {
    s0: String,
    s1: String,
    s2: String,
    s3: String,
    min_advances: u32,
    max_advances: u32,
    delay: u32,
    min_ivs: [u8; 6],
    max_ivs: [u8; 6],
    version: Version,
    story_flag: StoryFlag,
    room: RoomType,
    diglett_mode: bool,
    shiny: bool,
    exclusive: bool,
    gender: Option<u8>,
    ability: Option<u8>,
    egg_move: Option<u16>,
    natures: [bool; 25],
    item: Option<u16>,
    personal_info: Option<&'static PersonalInfoBDSP>,
    available_pokemon: Vec<u16>,
    available_egg_moves: Vec<u16>,
    show_statues: bool,
    statue_data: Vec<(String, Statue)>,
    selected_statue: Option<usize>,
    statue_config: StatueConfig,
    results: Vec<(
        String,
        String,
        &'static str,
        &'static str,
        String,
        String,
        String,
        String,
        String,
        String,
        &'static str,
        String,
        &'static str,
        &'static str,
        &'static str,
        String,
    )>,
    error: &'static str,
}

impl Default for BDSPUgGeneratorUI {
    fn default() -> Self {
        let statue_data_raw = get_statue_data();

        let mut statue_data = statue_data_raw
            .into_iter()
            .map(|s| {
                if s.rarity == 1 {
                    (SPECIES_EN[s.mons_id].to_string(), s)
                } else {
                    (format!("{} - Rare", SPECIES_EN[s.mons_id]), s)
                }
            })
            .collect::<Vec<(String, Statue)>>();

        statue_data.sort_by(|s1, s2| s1.0.cmp(&s2.0));

        Self {
            s0: "".to_string(),
            s1: "".to_string(),
            s2: "".to_string(),
            s3: "".to_string(),
            min_advances: 0,
            max_advances: 10000,
            delay: 0,
            min_ivs: [0, 0, 0, 0, 0, 0],
            max_ivs: [31, 31, 31, 31, 31, 31],
            version: Version::BD,
            story_flag: StoryFlag::default(),
            room: RoomType::SpaciousCave,
            diglett_mode: false,
            shiny: false,
            exclusive: false,
            gender: None,
            ability: None,
            egg_move: None,
            natures: [false; 25],
            item: None,
            personal_info: None,
            available_pokemon: available_pokemon(Version::BD, 6, RoomType::SpaciousCave),
            available_egg_moves: vec![],
            show_statues: false,
            statue_data,
            selected_statue: None,
            statue_config: StatueConfig::default(),
            results: vec![],
            error: "",
        }
    }
}

impl BDSPUgGeneratorUI {
    pub fn new(cc: &CreationContext<'_>) -> Self {
        cc.egui_ctx.set_visuals(Visuals::dark());
        Self::default()
    }
}

impl eframe::App for BDSPUgGeneratorUI {
    fn update(&mut self, ctx: &Context, _frame: &mut Frame) {
        if self.show_statues {
            egui::Window::new("Statue Config").show(ctx, |ui| {
                egui::ComboBox::new("statues", "")
                    .selected_text(if let Some(index) = self.selected_statue.as_ref() {
                        &self.statue_data[*index].0
                    } else {
                        "None"
                    })
                    .width(155.0)
                    .show_ui(ui, |ui| {
                        ui.selectable_value(&mut self.selected_statue, None, "None");
                        self.statue_data
                            .iter()
                            .enumerate()
                            .for_each(|(index, (name, _))| {
                                ui.selectable_value(
                                    &mut self.selected_statue,
                                    Some(index),
                                    name.as_str(),
                                );
                            });
                    });
                ui.horizontal(|ui| {
                    if ui.button("Add").clicked() {
                        if let Some(index) = self.selected_statue.as_ref() {
                            self.statue_config.add_statue(self.statue_data[*index].1);
                        }
                    }

                    if ui.button("Remove Last").clicked() {
                        self.statue_config.statues.pop();
                    }

                    if ui.button("Close").clicked() {
                        self.show_statues = false;
                    }
                });

                ui.add_space(10.0);

                ui.push_id("statue_table", |ui| {
                    TableBuilder::new(ui)
                        .striped(false)
                        .clip(false)
                        .column(Size::exact(150.0))
                        .resizable(false)
                        .header(25.0, |mut header| {
                            header.col(|ui| {
                                ui.heading("Statues");
                            });
                        })
                        .body(|b| {
                            b.rows(15.0, self.statue_config.statues.len(), |index, mut r| {
                                let statue = &self.statue_config.statues[index];
                                r.col(|ui| {
                                    if statue.rarity == 1 {
                                        ui.label(SPECIES_EN[statue.mons_id]);
                                    } else {
                                        ui.label(format!("{} - Rare", SPECIES_EN[statue.mons_id]));
                                    }
                                });
                            });
                        });
                });
            });
        }

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.vertical(|ui| {
                    egui::Grid::new("input_grid")
                        .num_columns(2)
                        .spacing([5.0, 5.0])
                        .show(ui, |ui| {
                            ui.label("s0");
                            let output = egui::TextEdit::singleline(&mut self.s0)
                                .desired_width(150.0)
                                .show(ui);
                            if output.response.changed() {
                                if self.s0.chars().count() > 8 {
                                    self.s0 = self.s0.chars().take(8).collect();
                                }
                            }
                            ui.end_row();
                            ui.label("s1");
                            let output = egui::TextEdit::singleline(&mut self.s1)
                                .desired_width(150.0)
                                .show(ui);
                            if output.response.changed() {
                                if self.s1.chars().count() > 8 {
                                    self.s1 = self.s1.chars().take(8).collect();
                                }
                            }
                            ui.end_row();
                            ui.label("s2");
                            let output = egui::TextEdit::singleline(&mut self.s2)
                                .desired_width(150.0)
                                .show(ui);
                            if output.response.changed() {
                                if self.s2.chars().count() > 8 {
                                    self.s2 = self.s2.chars().take(8).collect();
                                }
                            }
                            ui.end_row();
                            ui.label("s3");
                            let output = egui::TextEdit::singleline(&mut self.s3)
                                .desired_width(150.0)
                                .show(ui);
                            if output.response.changed() {
                                if self.s3.chars().count() > 8 {
                                    self.s3 = self.s3.chars().take(8).collect();
                                }
                            }
                            ui.end_row();
                            ui.label("Min Advances");
                            ui.add(egui::DragValue::new(&mut self.min_advances));
                            ui.end_row();
                            ui.label("Max Advances");
                            ui.add(egui::DragValue::new(&mut self.max_advances));
                            ui.end_row();
                            ui.label("Delay");
                            ui.add(egui::DragValue::new(&mut self.delay));
                            ui.end_row();

                            ui.label("Version");
                            egui::ComboBox::from_id_source("cmb_version")
                                .width(150.0)
                                .selected_text(match self.version {
                                    Version::BD => "Brilliant Diamond",
                                    Version::SP => "Shining Pearl",
                                })
                                .show_ui(ui, |ui| {
                                    if ui
                                        .selectable_value(
                                            &mut self.version,
                                            Version::BD,
                                            "Brilliant Diamond",
                                        )
                                        .clicked()
                                        || ui
                                            .selectable_value(
                                                &mut self.version,
                                                Version::SP,
                                                "Shining Pearl",
                                            )
                                            .clicked()
                                    {
                                        self.available_pokemon = available_pokemon(
                                            self.version,
                                            self.story_flag as u8,
                                            self.room,
                                        );
                                    }
                                });
                            ui.end_row();

                            ui.label("Story Flag");
                            egui::ComboBox::from_id_source("cmb_story_flag")
                                .width(150.0)
                                .selected_text(self.story_flag.get_str())
                                .show_ui(ui, |ui| {
                                    if ui
                                        .selectable_value(
                                            &mut self.story_flag,
                                            StoryFlag::UndergroundUnlocked,
                                            StoryFlag::UndergroundUnlocked.get_str(),
                                        )
                                        .clicked()
                                        || ui
                                            .selectable_value(
                                                &mut self.story_flag,
                                                StoryFlag::StrengthObtained,
                                                StoryFlag::StrengthObtained.get_str(),
                                            )
                                            .clicked()
                                        || ui
                                            .selectable_value(
                                                &mut self.story_flag,
                                                StoryFlag::DefogObtained,
                                                StoryFlag::DefogObtained.get_str(),
                                            )
                                            .clicked()
                                        || ui
                                            .selectable_value(
                                                &mut self.story_flag,
                                                StoryFlag::SevenBadges,
                                                StoryFlag::SevenBadges.get_str(),
                                            )
                                            .clicked()
                                        || ui
                                            .selectable_value(
                                                &mut self.story_flag,
                                                StoryFlag::WaterfallObtained,
                                                StoryFlag::WaterfallObtained.get_str(),
                                            )
                                            .clicked()
                                        || ui
                                            .selectable_value(
                                                &mut self.story_flag,
                                                StoryFlag::NationalDex,
                                                StoryFlag::NationalDex.get_str(),
                                            )
                                            .clicked()
                                    {
                                        self.available_pokemon = available_pokemon(
                                            self.version,
                                            self.story_flag as u8,
                                            self.room,
                                        );
                                    }
                                });
                            ui.end_row();

                            ui.label("Room");
                            egui::ComboBox::from_id_source("cmb_room")
                                .width(150.0)
                                .selected_text(match self.room {
                                    RoomType::SpaciousCave => "Spacious Cave",
                                    RoomType::GrasslandCave => "Grassland Cave",
                                    RoomType::FountainspringCave => "Fountainspring Cave",
                                    RoomType::RockyCave => "Rocky Cave",
                                    RoomType::VolcanicCave => "Volcanic Cave",
                                    RoomType::SwampyCave => "Swampy Cave",
                                    RoomType::DazzlingCave => "Dazzling Cave",
                                    RoomType::WhiteoutCave => "Whiteout Cave",
                                    RoomType::IcyCave => "Icy Cave",
                                    RoomType::RiverbankCave => "Riverbank Cave",
                                    RoomType::SandsearCave => "Sandsear Cave",
                                    RoomType::StillWaterCavern => "Still Water Cavern",
                                    RoomType::SunlitCavern => "Sunlit Cavern",
                                    RoomType::BigBluffCavern => "Big Bluff Cavern",
                                    RoomType::StargleamCavern => "Stargleam Cavern",
                                    RoomType::GlacialCavern => "Glacial Cavern",
                                    RoomType::BogsunkCavern => "Bogsunk Cavern",
                                    RoomType::TyphloCavern => "Typhlo Cavern",
                                })
                                .show_ui(ui, |ui| {
                                    if ui
                                        .selectable_value(
                                            &mut self.room,
                                            RoomType::SpaciousCave,
                                            "Spacious Cave",
                                        )
                                        .clicked()
                                        || ui
                                            .selectable_value(
                                                &mut self.room,
                                                RoomType::GrasslandCave,
                                                "Grassland Cave",
                                            )
                                            .clicked()
                                        || ui
                                            .selectable_value(
                                                &mut self.room,
                                                RoomType::FountainspringCave,
                                                "Fountainspring Cave",
                                            )
                                            .clicked()
                                        || ui
                                            .selectable_value(
                                                &mut self.room,
                                                RoomType::RockyCave,
                                                "Rocky Cave",
                                            )
                                            .clicked()
                                        || ui
                                            .selectable_value(
                                                &mut self.room,
                                                RoomType::VolcanicCave,
                                                "Volcanic Cave",
                                            )
                                            .clicked()
                                        || ui
                                            .selectable_value(
                                                &mut self.room,
                                                RoomType::SwampyCave,
                                                "Swampy Cave",
                                            )
                                            .clicked()
                                        || ui
                                            .selectable_value(
                                                &mut self.room,
                                                RoomType::DazzlingCave,
                                                "Dazzling Cave",
                                            )
                                            .clicked()
                                        || ui
                                            .selectable_value(
                                                &mut self.room,
                                                RoomType::WhiteoutCave,
                                                "Whiteout Cave",
                                            )
                                            .clicked()
                                        || ui
                                            .selectable_value(
                                                &mut self.room,
                                                RoomType::IcyCave,
                                                "Icy Cave",
                                            )
                                            .clicked()
                                        || ui
                                            .selectable_value(
                                                &mut self.room,
                                                RoomType::RiverbankCave,
                                                "Riverbank Cave",
                                            )
                                            .clicked()
                                        || ui
                                            .selectable_value(
                                                &mut self.room,
                                                RoomType::SandsearCave,
                                                "Sandsear Cave",
                                            )
                                            .clicked()
                                        || ui
                                            .selectable_value(
                                                &mut self.room,
                                                RoomType::StillWaterCavern,
                                                "Still Water Cavern",
                                            )
                                            .clicked()
                                        || ui
                                            .selectable_value(
                                                &mut self.room,
                                                RoomType::SunlitCavern,
                                                "Sunlit Cavern",
                                            )
                                            .clicked()
                                        || ui
                                            .selectable_value(
                                                &mut self.room,
                                                RoomType::BigBluffCavern,
                                                "Big Bluff Cavern",
                                            )
                                            .clicked()
                                        || ui
                                            .selectable_value(
                                                &mut self.room,
                                                RoomType::StargleamCavern,
                                                "Stargleam Cavern",
                                            )
                                            .clicked()
                                        || ui
                                            .selectable_value(
                                                &mut self.room,
                                                RoomType::GlacialCavern,
                                                "Glacial Cavern",
                                            )
                                            .clicked()
                                        || ui
                                            .selectable_value(
                                                &mut self.room,
                                                RoomType::BogsunkCavern,
                                                "Bogsunk Cavern",
                                            )
                                            .clicked()
                                        || ui
                                            .selectable_value(
                                                &mut self.room,
                                                RoomType::TyphloCavern,
                                                "Typhlo Cavern",
                                            )
                                            .clicked()
                                    {
                                        self.available_pokemon = available_pokemon(
                                            self.version,
                                            self.story_flag as u8,
                                            self.room,
                                        );
                                    }
                                });
                            ui.end_row();

                            ui.label("Diglett Mode");
                            ui.checkbox(&mut self.diglett_mode, "");
                            ui.end_row();

                            ui.label("Species");
                            egui::ComboBox::from_id_source("cmb_species")
                                .width(150.0)
                                .selected_text(if let Some(personal_info) = self.personal_info {
                                    SPECIES_EN[personal_info.get_species()]
                                } else {
                                    "None"
                                })
                                .show_ui(ui, |ui| {
                                    if ui.selectable_label(false, "Any").clicked() {
                                        self.personal_info = None;
                                    }
                                    for &p in &self.available_pokemon {
                                        if ui
                                            .selectable_label(false, SPECIES_EN[p as usize])
                                            .clicked()
                                        {
                                            self.personal_info = Some(
                                                personal_table::BDSP.get_form_entry(p as usize, 0),
                                            );
                                            self.available_egg_moves = get_available_egg_moves(p);
                                        }
                                    }
                                });
                            ui.end_row();

                            ui.label("Shiny");
                            ui.checkbox(&mut self.shiny, "");
                            ui.end_row();

                            ui.label("Gender");
                            egui::ComboBox::from_id_source("cmb_gender")
                                .width(150.0)
                                .selected_text(if let Some(i) = self.gender {
                                    GENDER_SYMBOLS[i as usize]
                                } else {
                                    "Any"
                                })
                                .show_ui(ui, |ui| {
                                    ui.selectable_value(&mut self.gender, None, "Any");
                                    ui.selectable_value(
                                        &mut self.gender,
                                        Some(0),
                                        GENDER_SYMBOLS[0],
                                    );
                                    ui.selectable_value(
                                        &mut self.gender,
                                        Some(1),
                                        GENDER_SYMBOLS[1],
                                    );
                                    ui.selectable_value(
                                        &mut self.gender,
                                        Some(2),
                                        GENDER_SYMBOLS[2],
                                    );
                                });
                            ui.end_row();

                            ui.label("Nature");
                            egui::ComboBox::from_id_source("cmb_nature")
                                .width(150.0)
                                .selected_text(if self.natures.iter().all(|&n| n == false) {
                                    "Any".to_string()
                                } else {
                                    let mut s = String::new();
                                    for (i, nature) in self.natures.iter().enumerate() {
                                        if *nature {
                                            if s.is_empty() {
                                                s = NATURES_EN[i].to_string();
                                            } else {
                                                s = format!("{},{}", s, NATURES_EN[i]);
                                            }
                                        }
                                    }
                                    s
                                })
                                .show_ui(ui, |ui| {
                                    if ui.selectable_label(false, "Clear").clicked() {
                                        for nature in self.natures.iter_mut() {
                                            *nature = false;
                                        }
                                    }
                                    for i in 0..25 {
                                        ui.checkbox(&mut self.natures[i], NATURES_EN[i]);
                                    }
                                });
                            ui.end_row();

                            ui.label("Ability");
                            egui::ComboBox::from_id_source("cmb_ability")
                                .selected_text(if let Some(personal_info) = &self.personal_info {
                                    if let Some(ability) = &self.ability {
                                        if *ability == 0 {
                                            ABILITIES_EN[personal_info.get_ability_1()]
                                        } else {
                                            ABILITIES_EN[personal_info.get_ability_2()]
                                        }
                                    } else {
                                        "Any"
                                    }
                                } else {
                                    if let Some(ability) = &self.ability {
                                        if *ability == 0 {
                                            "1"
                                        } else {
                                            "2"
                                        }
                                    } else {
                                        "Any"
                                    }
                                })
                                .show_ui(ui, |ui| {
                                    ui.selectable_value(&mut self.ability, None, "Any");
                                    if let Some(personal_info) = &self.personal_info {
                                        ui.selectable_value(
                                            &mut self.ability,
                                            Some(0),
                                            ABILITIES_EN[personal_info.get_ability_1()],
                                        );
                                        ui.selectable_value(
                                            &mut self.ability,
                                            Some(1),
                                            ABILITIES_EN[personal_info.get_ability_2()],
                                        );
                                    } else {
                                        ui.selectable_value(&mut self.ability, Some(0), "1");
                                        ui.selectable_value(&mut self.ability, Some(1), "2");
                                    }
                                });
                            ui.end_row();

                            ui.label("Egg Move");
                            egui::ComboBox::from_id_source("cmb_egg_move")
                                .width(150.0)
                                .selected_text(if self.personal_info.is_some() {
                                    if let Some(egg_move) = &self.egg_move {
                                        MOVES_EN[*egg_move as usize]
                                    } else {
                                        "Any"
                                    }
                                } else {
                                    "Any"
                                })
                                .show_ui(ui, |ui| {
                                    ui.selectable_value(&mut self.egg_move, None, "Any");
                                    if self.personal_info.is_some() {
                                        for &available_egg_move in &self.available_egg_moves {
                                            ui.selectable_value(
                                                &mut self.egg_move,
                                                Some(available_egg_move),
                                                MOVES_EN[available_egg_move as usize],
                                            );
                                        }
                                    }
                                });
                            ui.end_row();

                            ui.label("Item");
                            egui::ComboBox::from_id_source("cmb_item")
                                .width(150.0)
                                .selected_text(if self.personal_info.is_some() {
                                    if let Some(item) = &self.item {
                                        ITEMS_EN[*item as usize]
                                    } else {
                                        "Any"
                                    }
                                } else {
                                    "Any"
                                })
                                .show_ui(ui, |ui| {
                                    ui.selectable_value(&mut self.item, None, "Any");
                                    if let Some(personal_info) = &self.personal_info {
                                        ui.selectable_value(
                                            &mut self.item,
                                            Some(personal_info.get_item_1() as u16),
                                            ITEMS_EN[personal_info.get_item_1()],
                                        );
                                        ui.selectable_value(
                                            &mut self.item,
                                            Some(personal_info.get_item_2() as u16),
                                            ITEMS_EN[personal_info.get_item_2()],
                                        );
                                        ui.selectable_value(
                                            &mut self.item,
                                            Some(personal_info.get_item_3() as u16),
                                            ITEMS_EN[personal_info.get_item_3()],
                                        );
                                    }
                                });
                            ui.end_row();
                            ui.label("Exclusive Search");
                            ui.checkbox(&mut self.exclusive, "");
                            ui.end_row();
                        });
                    if ui.button("Statues").clicked() {
                        self.show_statues = true;
                    }
                    ui.add_space(5.0);
                    if ui.button("Search").clicked() {
                        if let Ok(s0) = u32::from_str_radix(&self.s0, 16) {
                            if let Ok(s1) = u32::from_str_radix(&self.s1, 16) {
                                if let Ok(s2) = u32::from_str_radix(&self.s2, 16) {
                                    if let Ok(s3) = u32::from_str_radix(&self.s3, 16) {
                                        let filter = Filter {
                                            shiny: self.shiny,
                                            species: if let Some(personal_info) =
                                                &self.personal_info
                                            {
                                                Some(personal_info.get_species() as u16)
                                            } else {
                                                None
                                            },
                                            min_ivs: self.min_ivs,
                                            max_ivs: self.max_ivs,
                                            ability: self.ability,
                                            nature: {
                                                let natures = self
                                                    .natures
                                                    .iter()
                                                    .enumerate()
                                                    .filter_map(|(i, &n)| {
                                                        if n == true {
                                                            Some(i as u8)
                                                        } else {
                                                            None
                                                        }
                                                    })
                                                    .collect::<Vec<u8>>();
                                                if natures.is_empty() {
                                                    None
                                                } else {
                                                    Some(natures)
                                                }
                                            },
                                            item: self.item,
                                            egg_move: self.egg_move,
                                            gender: self.gender,
                                            exclusive: self.exclusive,
                                        };

                                        let mut rng = XorShift::from_state([s0, s1, s2, s3]);
                                        if self.min_advances < 4096 {
                                            rng.advance(
                                                self.min_advances as usize + self.delay as usize,
                                            );
                                        } else {
                                            rng.jump(
                                                self.min_advances as usize + self.delay as usize,
                                            );
                                        }

                                        let results = run_results(
                                            self.max_advances,
                                            rng,
                                            self.version,
                                            self.story_flag as u8,
                                            self.room,
                                            filter,
                                            self.diglett_mode,
                                            &self.statue_config,
                                        );
                                        let mut count = 0;
                                        for result in results.iter() {
                                            count += result.regular_pokemon.len();
                                            if result.rare_pokemon.is_some() {
                                                count += 1;
                                            }
                                        }
                                        self.results = Vec::with_capacity(count);
                                        for result in results {
                                            for pokemon in result.regular_pokemon {
                                                let personal_info = if let Some(personal_info) =
                                                    &self.personal_info
                                                {
                                                    *personal_info
                                                } else {
                                                    personal_table::BDSP
                                                        .get_form_entry(pokemon.species as usize, 0)
                                                };
                                                let ability = if pokemon.ability == 0 {
                                                    personal_info.get_ability_1()
                                                } else {
                                                    personal_info.get_ability_2()
                                                };

                                                let egg_move =
                                                    if let Some(egg_move) = &pokemon.egg_move {
                                                        *egg_move
                                                    } else {
                                                        0
                                                    };

                                                self.results.push((
                                                    (result.advance + self.min_advances)
                                                        .to_string(),
                                                    format!("{:X}", pokemon.pid),
                                                    SPECIES_EN[pokemon.species as usize],
                                                    if pokemon.shiny { "!!!" } else { "X" },
                                                    pokemon.ivs[0].to_string(),
                                                    pokemon.ivs[1].to_string(),
                                                    pokemon.ivs[2].to_string(),
                                                    pokemon.ivs[3].to_string(),
                                                    pokemon.ivs[4].to_string(),
                                                    pokemon.ivs[5].to_string(),
                                                    ABILITIES_EN[ability],
                                                    GENDER_SYMBOLS[pokemon.gender as usize]
                                                        .to_string(),
                                                    NATURES_EN[pokemon.nature as usize],
                                                    ITEMS_EN[pokemon.item as usize],
                                                    MOVES_EN[egg_move as usize],
                                                    format!("{:X}", pokemon.ec),
                                                ));
                                            }

                                            if let Some(pokemon) = result.rare_pokemon {
                                                let personal_info = if let Some(personal_info) =
                                                    &self.personal_info
                                                {
                                                    *personal_info
                                                } else {
                                                    personal_table::BDSP
                                                        .get_form_entry(pokemon.species as usize, 0)
                                                };
                                                let ability = if pokemon.ability == 0 {
                                                    personal_info.get_ability_1()
                                                } else {
                                                    personal_info.get_ability_2()
                                                };

                                                let egg_move =
                                                    if let Some(egg_move) = &pokemon.egg_move {
                                                        *egg_move
                                                    } else {
                                                        0
                                                    };

                                                self.results.push((
                                                    (result.advance + self.min_advances)
                                                        .to_string(),
                                                    format!("{:X}", pokemon.pid),
                                                    SPECIES_EN[pokemon.species as usize],
                                                    if pokemon.shiny { "!!!" } else { "X" },
                                                    pokemon.ivs[0].to_string(),
                                                    pokemon.ivs[1].to_string(),
                                                    pokemon.ivs[2].to_string(),
                                                    pokemon.ivs[3].to_string(),
                                                    pokemon.ivs[4].to_string(),
                                                    pokemon.ivs[5].to_string(),
                                                    ABILITIES_EN[ability],
                                                    GENDER_SYMBOLS[pokemon.gender as usize]
                                                        .to_string(),
                                                    NATURES_EN[pokemon.nature as usize],
                                                    ITEMS_EN[pokemon.item as usize],
                                                    MOVES_EN[egg_move as usize],
                                                    format!("{:X}", pokemon.ec),
                                                ));
                                            }
                                        }
                                    } else {
                                        self.error = "Failed to parse s3";
                                    }
                                } else {
                                    self.error = "Failed to parse s2";
                                }
                            } else {
                                self.error = "Failed to parse s1";
                            }
                        } else {
                            self.error = "Failed to parse s0";
                        }
                    }
                    ui.label(self.error);
                });
                ui.add_space(10.0);
                ui.vertical(|ui| {
                    egui::Grid::new("iv_filter")
                        .num_columns(3)
                        .spacing([5.0, 5.0])
                        .show(ui, |ui| {
                            ui.label("HP");
                            ui.add(egui::DragValue::new(&mut self.min_ivs[0]).clamp_range(0..=31));
                            ui.add(egui::DragValue::new(&mut self.max_ivs[0]).clamp_range(0..=31));
                            ui.end_row();
                            ui.label("ATK");
                            ui.add(egui::DragValue::new(&mut self.min_ivs[1]).clamp_range(0..=31));
                            ui.add(egui::DragValue::new(&mut self.max_ivs[1]).clamp_range(0..=31));
                            ui.end_row();
                            ui.label("DEF");
                            ui.add(egui::DragValue::new(&mut self.min_ivs[2]).clamp_range(0..=31));
                            ui.add(egui::DragValue::new(&mut self.max_ivs[2]).clamp_range(0..=31));
                            ui.end_row();
                            ui.label("SPA");
                            ui.add(egui::DragValue::new(&mut self.min_ivs[3]).clamp_range(0..=31));
                            ui.add(egui::DragValue::new(&mut self.max_ivs[3]).clamp_range(0..=31));
                            ui.end_row();
                            ui.label("SPD");
                            ui.add(egui::DragValue::new(&mut self.min_ivs[4]).clamp_range(0..=31));
                            ui.add(egui::DragValue::new(&mut self.max_ivs[4]).clamp_range(0..=31));
                            ui.end_row();
                            ui.label("SPE");
                            ui.add(egui::DragValue::new(&mut self.min_ivs[5]).clamp_range(0..=31));
                            ui.add(egui::DragValue::new(&mut self.max_ivs[5]).clamp_range(0..=31));
                            ui.end_row();
                        });
                });
                ui.add_space(10.0);
                ui.vertical(|ui| {
                    TableBuilder::new(ui)
                        .striped(true)
                        .cell_layout(egui::Layout::centered_and_justified(
                            egui::Direction::LeftToRight,
                        ))
                        .column(Size::initial(80.0).at_least(80.0))
                        .column(Size::initial(80.0).at_least(80.0))
                        .column(Size::initial(100.0).at_least(100.0))
                        .column(Size::initial(60.0).at_least(60.0))
                        .column(Size::initial(40.0).at_least(40.0))
                        .column(Size::initial(40.0).at_least(40.0))
                        .column(Size::initial(40.0).at_least(40.0))
                        .column(Size::initial(40.0).at_least(40.0))
                        .column(Size::initial(40.0).at_least(40.0))
                        .column(Size::initial(40.0).at_least(40.0))
                        .column(Size::initial(100.0).at_least(100.0))
                        .column(Size::initial(70.0).at_least(70.0))
                        .column(Size::initial(70.0).at_least(70.0))
                        .column(Size::initial(70.0).at_least(70.0))
                        .column(Size::initial(100.0).at_least(100.0))
                        .column(Size::initial(80.0).at_least(80.0))
                        .resizable(true)
                        .header(20.0, |mut header| {
                            header.col(|ui| {
                                ui.heading("Advance");
                            });
                            header.col(|ui| {
                                ui.heading("PID");
                            });
                            header.col(|ui| {
                                ui.heading("Species");
                            });
                            header.col(|ui| {
                                ui.heading("Shiny");
                            });
                            header.col(|ui| {
                                ui.heading("HP");
                            });
                            header.col(|ui| {
                                ui.heading("ATK");
                            });
                            header.col(|ui| {
                                ui.heading("DEF");
                            });
                            header.col(|ui| {
                                ui.heading("SPA");
                            });
                            header.col(|ui| {
                                ui.heading("SPD");
                            });
                            header.col(|ui| {
                                ui.heading("SPD");
                            });
                            header.col(|ui| {
                                ui.heading("Ability");
                            });
                            header.col(|ui| {
                                ui.heading("Gender");
                            });
                            header.col(|ui| {
                                ui.heading("Nature");
                            });
                            header.col(|ui| {
                                ui.heading("Item");
                            });
                            header.col(|ui| {
                                ui.heading("Egg Move");
                            });
                            header.col(|ui| {
                                ui.heading("EC");
                            });
                        })
                        .body(|body| {
                            body.rows(18.0, self.results.len(), |index, mut row| {
                                let result = self.results.get(index).unwrap();
                                row.col(|ui| {
                                    ui.label(&result.0);
                                });
                                row.col(|ui| {
                                    ui.label(&result.1);
                                });
                                row.col(|ui| {
                                    ui.label(result.2);
                                });
                                row.col(|ui| {
                                    ui.label(result.3);
                                });
                                row.col(|ui| {
                                    ui.label(&result.4);
                                });
                                row.col(|ui| {
                                    ui.label(&result.5);
                                });
                                row.col(|ui| {
                                    ui.label(&result.6);
                                });
                                row.col(|ui| {
                                    ui.label(&result.7);
                                });
                                row.col(|ui| {
                                    ui.label(&result.8);
                                });
                                row.col(|ui| {
                                    ui.label(&result.9);
                                });
                                row.col(|ui| {
                                    ui.label(result.10);
                                });
                                row.col(|ui| {
                                    ui.label(&result.11);
                                });
                                row.col(|ui| {
                                    ui.label(result.12);
                                });
                                row.col(|ui| {
                                    ui.label(result.13);
                                });
                                row.col(|ui| {
                                    ui.label(result.14);
                                });
                                row.col(|ui| {
                                    ui.label(&result.15);
                                });
                            });
                        });
                });
            });
        });
    }
}
