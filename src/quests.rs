use serde::Deserialize;

use crate::{
    game::{Building, Settlement},
    region::{DistrictNeed, Material, Region},
};

#[derive(Debug, Deserialize)]
pub struct CampaignQuests {
    pub preparation: Option<QuestStep>,
    pub finale: Option<QuestStep>,
    pub side: QuestStep,
    pub request: String,
    pub offering: (Material, u16),
    pub districts: Option<[(DistrictNeed, String); 2]>,
}

#[derive(Debug, Deserialize)]
pub struct QuestStep {
    pub title: String,
    pub hint: String,
    pub goals: Vec<Goal>,
}

impl QuestStep {
    pub fn pending(&self, city: &Settlement, region: &Region) -> Option<Goal> {
        self.goals
            .iter()
            .copied()
            .find(|goal| !goal.met(city, region))
    }

    pub fn progress(&self, city: &Settlement, region: &Region) -> String {
        self.goals
            .iter()
            .map(|goal| goal.progress(city, region))
            .collect::<Vec<_>>()
            .join(" · ")
    }
}

#[derive(Clone, Copy, Debug, Deserialize)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum Goal {
    Harvests {
        count: u16,
    },
    Reserve {
        material: Material,
        count: u16,
    },
    HippoDeliveries {
        count: u16,
    },
    Roads {
        count: u16,
    },
    Camps {
        count: u16,
    },
    OlivePressings {
        count: u16,
    },
    Building {
        kind: Building,
        count: u16,
        #[serde(default)]
        roadside: bool,
        #[serde(default)]
        upgraded: bool,
    },
}

impl Goal {
    fn amount(self, city: &Settlement, region: &Region) -> (u16, u16) {
        match self {
            Self::Harvests { count } => (city.harvests, count),
            Self::Reserve { material, count } => (city.stock[material.index()], count),
            Self::HippoDeliveries { count } => (city.hippo_deliveries, count),
            Self::Roads { count } => (city.roads.len() as u16, count),
            Self::Camps { count } => (
                city.camps.iter().filter(|&&open| open).count() as u16,
                count,
            ),
            Self::OlivePressings { count } => (city.olive_pressings, count),
            Self::Building {
                kind,
                count,
                roadside,
                upgraded,
            } => (
                city.buildings
                    .iter()
                    .filter(|site| {
                        site.kind == kind
                            && (!upgraded || site.upgraded)
                            && (!roadside || city.connected(region, site))
                    })
                    .count() as u16,
                count,
            ),
        }
    }

    pub fn met(self, city: &Settlement, region: &Region) -> bool {
        let (have, need) = self.amount(city, region);
        have >= need
    }

    pub fn prompt(self) -> &'static str {
        match self {
            Self::Harvests { .. } => "Ernte einbringen",
            Self::Reserve { .. } => "Vorrat anlegen",
            Self::HippoDeliveries { .. } => "Nilpferd sammeln lassen",
            Self::Roads { .. } => "Wege anlegen",
            Self::Camps { .. } => "Sammellager öffnen",
            Self::OlivePressings { .. } => "Oliven pressen",
            Self::Building { upgraded: true, .. } => "Gebäude ausbauen",
            Self::Building {
                kind: Building::Well,
                ..
            } => "Brunnen bauen",
            Self::Building {
                kind: Building::Market,
                ..
            } => "Markt bauen",
            Self::Building {
                kind: Building::Tower,
                ..
            } => "Wachturm bauen",
            Self::Building {
                kind: Building::Granary,
                ..
            } => "Speicher bauen",
            Self::Building {
                kind: Building::Civic(kind),
                ..
            } => kind.spec().name,
            Self::Building { .. } => "Dorf ausbauen",
        }
    }

    pub fn progress(self, city: &Settlement, region: &Region) -> String {
        let label = match self {
            Self::Harvests { .. } => "Ernten".into(),
            Self::Reserve { material, .. } => region.kind.material(material).into(),
            Self::HippoDeliveries { .. } => "Nilpferd-Lieferungen".into(),
            Self::Roads { .. } => "Neue Wegfelder".into(),
            Self::Camps { .. } => "Sammellager".into(),
            Self::OlivePressings { .. } => "Pressgänge".into(),
            Self::Building {
                kind,
                roadside,
                upgraded,
                ..
            } => {
                let road = if roadside { " am Weg" } else { "" };
                let upgrade = if upgraded { "Ausbau " } else { "" };
                format!("{upgrade}{}{road}", kind.name())
            }
        };
        let (have, need) = self.amount(city, region);
        format!("{label} {}/{need}", have.min(need))
    }
}
