use serde::{Deserialize, Serialize};

use crate::region::{Biome, DistrictNeed, Region, Tile};

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
pub enum Mode {
    #[default]
    Chronicle,
    Campaign(Biome),
    Sandbox(Biome),
}

impl Mode {
    pub fn name(self) -> &'static str {
        match self {
            Self::Chronicle => "Die Schreine am Wasser",
            Self::Campaign(Biome::Valley) => "Das Weidenland",
            Self::Campaign(Biome::Fjord) => "Die Lichter des Fjords",
            Self::Campaign(Biome::Terraces) => "Die Kanäle der Jade",
            Self::Campaign(Biome::Lake) => "Die Glocken der Oase",
            Self::Campaign(Biome::Aegean) => "Die Winde der Ägäis",
            Self::Sandbox(biome) => biome.theme(),
        }
    }

    pub fn sandbox(self) -> bool {
        matches!(self, Self::Sandbox(_))
    }

    pub fn regions(self) -> Vec<Region> {
        let biome = match self {
            Self::Chronicle => return Region::campaign(),
            Self::Campaign(biome) | Self::Sandbox(biome) => biome,
        };
        let themes: Vec<Theme> = serde_json::from_str(include_str!("../assets/world/themes.json"))
            .expect("The embedded campaign descriptions must be valid");
        let theme = themes
            .into_iter()
            .find(|theme| theme.biome == biome)
            .unwrap();
        theme.chapters.into_iter().enumerate().take(if self.sandbox() { 1 } else { 3 })
            .map(|(index, chapter)| {
                let mut region = Region::themed(biome, index);
                region.name = chapter.name;
                region.subtitle = chapter.subtitle;
                region.lore = chapter.lore;
                if let Some(rival) = chapter.rival {
                    region.rival = rival;
                }
                if index > 0 {
                    region.gift.1 = region.gift.1.saturating_add(index as u16 + 2);
                    region.renown += 3 * index as u16;
                    let needs = [DistrictNeed::Water, DistrictNeed::Trade, DistrictNeed::Patrol, DistrictNeed::Shelter];
                    for (i, district) in region.districts.iter_mut().enumerate() {
                        district.need = needs[(biome as usize + index + i) % needs.len()];
                    }
                } else if self.sandbox() {
                    region.guards.clear();
                    region.sandbox = true;
                    region.rival = "Ufergärten".into();
                    region.lore = "Baue im ganzen Gebiet. Die Brücken sind offen, die Siedlungen sind friedlich und die Baupläne liegen bereit.".into();
                }
                region
            })
            .collect()
    }
}

#[derive(Deserialize)]
struct Theme {
    biome: Biome,
    chapters: [Chapter; 3],
}

#[derive(Deserialize)]
struct Chapter {
    name: String,
    subtitle: String,
    lore: String,
    rival: Option<String>,
}

#[derive(Clone, Copy, Debug, Default)]
pub struct Layout {
    pub mirror: bool,
    pub shift: Tile,
    pub stage: usize,
}

impl Layout {
    pub fn for_stage(stage: usize, biome: Biome) -> Self {
        let drift = biome as i16 * 2;
        match stage {
            1 => Self {
                mirror: true,
                shift: Tile(31 + drift, 7),
                stage,
            },
            2 => Self {
                mirror: false,
                shift: Tile(13 - drift, -8),
                stage,
            },
            _ => Self::default(),
        }
    }

    pub fn tile(self, tile: Tile) -> Tile {
        Tile(if self.mirror { -tile.0 } else { tile.0 }, tile.1).offset(self.shift.0, self.shift.1)
    }

    pub fn original_x(self, x: i16) -> i16 {
        let x = x - self.shift.0;
        if self.mirror { -x } else { x }
    }

    pub fn original_point(self, point: bevy::math::Vec2) -> bevy::math::Vec2 {
        let point = point - self.shift.local();
        point.with_x(if self.mirror { -point.x } else { point.x })
    }
}
