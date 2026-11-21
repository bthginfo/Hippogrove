use std::sync::OnceLock;

use serde::{Deserialize, Serialize};

use crate::{
    quests::CampaignQuests,
    region::{Biome, DistrictNeed, Region, Tile, WorldLayout},
};

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
pub enum Mode {
    #[default]
    Chronicle,
    Campaign(Biome),
    Sandbox(Biome),
    Tutorial,
}

impl Mode {
    pub fn name(self) -> &'static str {
        match self {
            Self::Chronicle => "Die Schreine am Wasser",
            Self::Tutorial => "Übungsinsel",
            Self::Campaign(Biome::Valley) => "Das Weidenland",
            Self::Campaign(Biome::Fjord) => "Die Lichter des Fjords",
            Self::Campaign(Biome::Terraces) => "Die Kanäle der Jade",
            Self::Campaign(Biome::Lake) => "Die Glocken der Oase",
            Self::Campaign(Biome::Aegean) => "Die Winde der Ägäis",
            Self::Sandbox(biome) => biome.theme(),
        }
    }

    pub fn slot_name(self) -> String {
        match self {
            Self::Campaign(_) => format!("Kampagne · {}", self.name()),
            Self::Sandbox(_) => format!("Freies Bauen · {}", self.name()),
            Self::Chronicle => format!("Chronik · {}", self.name()),
            Self::Tutorial => "Tutorial · Übungsinsel".into(),
        }
    }

    pub fn sandbox(self) -> bool {
        matches!(self, Self::Sandbox(_))
    }

    pub fn regions(self, world_layout: impl Fn(usize) -> WorldLayout) -> Vec<Region> {
        let biome = match self {
            Self::Chronicle => return Region::campaign(),
            Self::Tutorial => return vec![Region::practice()],
            Self::Campaign(biome) | Self::Sandbox(biome) => biome,
        };
        static THEMES: OnceLock<[Theme; 5]> = OnceLock::new();
        let themes = THEMES.get_or_init(|| {
            serde_json::from_str(include_str!("../assets/world/themes.json"))
                .expect("The embedded campaign descriptions must be valid")
        });
        let theme = themes
            .iter()
            .find(|theme| theme.biome == biome)
            .expect("Each biome has a campaign");
        theme
            .chapters
            .iter()
            .enumerate()
            .take(if self.sandbox() { 1 } else { 3 })
            .map(|(index, chapter)| {
                let layout = world_layout(index);
                let mut region = Region::themed(biome, index, layout);
                region.name.clone_from(&chapter.name);
                region.subtitle.clone_from(&chapter.subtitle);
                region.lore.clone_from(&chapter.lore);
                if let Some(rival) = &chapter.rival {
                    region.rival.clone_from(rival);
                }
                region.renown += 3 * index as u16;
                if self.sandbox() {
                    region.guards.clear();
                    region.sandbox = true;
                    region.rival = "Ufergärten".into();
                    region.lore = concat!(
                        "Baue im ganzen Gebiet. Die Brücken sind offen, ",
                        "die Siedlungen sind friedlich und die Baupläne liegen bereit."
                    )
                    .into();
                } else if layout != WorldLayout::Classic {
                    let quests = &chapter.quests;
                    region.request.clone_from(&quests.request);
                    region.gift = quests.offering;
                    if let Some(districts) = &quests.districts {
                        for (district, (need, story)) in region.districts.iter_mut().zip(districts)
                        {
                            district.need = *need;
                            district.story.clone_from(story);
                        }
                    }
                    region.quests = Some(quests);
                    if layout == WorldLayout::Islands {
                        region.rival = crate::rivals::Rival::for_biome(biome).name.into();
                    }
                } else if index > 0 {
                    region.gift.1 = region.gift.1.saturating_add(index as u16 + 2);
                    let needs = [
                        DistrictNeed::Water,
                        DistrictNeed::Trade,
                        DistrictNeed::Patrol,
                        DistrictNeed::Shelter,
                    ];
                    for (i, district) in region.districts.iter_mut().enumerate() {
                        district.need = needs[(biome as usize + index + i) % needs.len()];
                    }
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
    quests: CampaignQuests,
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

    pub fn plot(self, tile: Tile) -> Tile {
        self.tile(tile).offset(-i16::from(self.mirror), 0)
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
