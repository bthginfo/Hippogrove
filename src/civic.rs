use serde::{Deserialize, Serialize};

use crate::{game::Stock, region::Biome};

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum Branch {
    #[default]
    Homes,
    Industry,
    Culture,
    Defense,
}

impl Branch {
    pub const ALL: [Self; 4] = [Self::Homes, Self::Industry, Self::Culture, Self::Defense];

    pub fn name(self) -> &'static str {
        match self {
            Self::Homes => "Wohnen",
            Self::Industry => "Handwerk",
            Self::Culture => "Dorfleben",
            Self::Defense => "Wache",
        }
    }

    pub fn topics(self) -> [Tech; 3] {
        match self {
            Self::Homes => [Tech::Foundations, Tech::Families, Tech::Waterworks],
            Self::Industry => [Tech::Workshops, Tech::Smithing, Tech::Navigation],
            Self::Culture => [Tech::Commons, Tech::Festivals, Tech::Lore],
            Self::Defense => [Tech::Palisades, Tech::Militia, Tech::Fortifications],
        }
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum Tech {
    #[default]
    Foundations,
    Families,
    Waterworks,
    Workshops,
    Smithing,
    Navigation,
    Commons,
    Festivals,
    Lore,
    Palisades,
    Militia,
    Fortifications,
}

impl Tech {
    pub fn branch(self) -> Branch {
        Branch::ALL
            .into_iter()
            .find(|branch| branch.topics().contains(&self))
            .unwrap()
    }

    pub fn name(self) -> &'static str {
        match self {
            Self::Foundations => "Neue Feuerstellen",
            Self::Families => "Familienhöfe",
            Self::Waterworks => "Wasserkunst",
            Self::Workshops => "Werkbänke",
            Self::Smithing => "Schmiedekunst",
            Self::Navigation => "Handelsrouten",
            Self::Commons => "Gemeinschaftsplätze",
            Self::Festivals => "Feste und Spiele",
            Self::Lore => "Lieder und Weissagung",
            Self::Palisades => "Grenzwacht",
            Self::Militia => "Dorfwache",
            Self::Fortifications => "Festungsbau",
        }
    }

    pub fn parents(self) -> &'static [Self] {
        match self {
            Self::Families => &[Self::Foundations],
            Self::Waterworks => &[Self::Families, Self::Workshops],
            Self::Smithing => &[Self::Workshops],
            Self::Navigation => &[Self::Smithing, Self::Commons],
            Self::Festivals => &[Self::Commons],
            Self::Lore => &[Self::Festivals, Self::Families],
            Self::Militia => &[Self::Palisades, Self::Workshops],
            Self::Fortifications => &[Self::Militia, Self::Smithing],
            _ => &[],
        }
    }

    pub fn price(self) -> u16 {
        match self {
            Self::Foundations | Self::Workshops | Self::Commons | Self::Palisades => 2,
            Self::Families | Self::Smithing | Self::Festivals | Self::Militia => 3,
            Self::Waterworks | Self::Navigation | Self::Lore | Self::Fortifications => 5,
        }
    }

    pub fn city_rank(self) -> u8 {
        match self.price() {
            2 => 1,
            3 => 2,
            _ => 3,
        }
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
#[serde(default)]
pub struct Research {
    pub insight: u16,
    learned: u16,
}

impl Default for Research {
    fn default() -> Self {
        Self {
            insight: 2,
            learned: 0,
        }
    }
}

impl Research {
    pub fn all_plans() -> Self {
        let mut research = Self::default();
        for tech in Branch::ALL.into_iter().flat_map(Branch::topics) {
            research.learned |= 1 << tech as u16;
        }
        research
    }
    pub fn knows(self, tech: Tech) -> bool {
        self.learned & (1 << tech as u8) != 0
    }

    pub fn count(self) -> u32 {
        self.learned.count_ones()
    }

    pub fn earn(&mut self, insight: u16) {
        self.insight = self.insight.saturating_add(insight);
    }

    pub fn available(self, tech: Tech, city_rank: u8) -> Result<(), String> {
        if self.knows(tech) {
            return Err("Diese Forschung ist abgeschlossen.".into());
        }
        let missing: Vec<_> = tech
            .parents()
            .iter()
            .filter(|&&parent| !self.knows(parent))
            .map(|parent| parent.name())
            .collect();
        if !missing.is_empty() {
            return Err(format!("Voraussetzung · {}", missing.join(" und ")));
        }
        if city_rank < tech.city_rank() {
            return Err(format!("Dorfstufe {} erforderlich", tech.city_rank()));
        }
        if self.insight < tech.price() {
            return Err(format!("{} Erkenntnis erforderlich", tech.price()));
        }
        Ok(())
    }

    pub fn learn(&mut self, tech: Tech, city_rank: u8) -> Result<(), String> {
        self.available(tech, city_rank)?;
        self.insight -= tech.price();
        self.learned |= 1 << tech as u8;
        Ok(())
    }

    pub fn permits(self, kind: Civic) -> Result<(), String> {
        let unlock = kind.spec().unlock;
        if self.knows(unlock) {
            Ok(())
        } else {
            Err(format!("Erforsche zuerst {}.", unlock.name()))
        }
    }
}

pub type Goods = [u16; 3];
pub const GOODS_LABELS: [&str; 3] = ["Bretter", "Quader", "Werkzeug"];

pub fn goods_text(goods: Goods) -> String {
    goods
        .into_iter()
        .zip(GOODS_LABELS)
        .filter(|&(quantity, _)| quantity > 0)
        .map(|(quantity, label)| format!("{quantity} {label}"))
        .collect::<Vec<_>>()
        .join(" · ")
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub enum Civic {
    Lodge,
    Courtyard,
    Nursery,
    Infirmary,
    Bath,
    Sawmill,
    Masonry,
    Bakery,
    Forge,
    Port,
    Garden,
    Tavern,
    Hall,
    Theater,
    Archive,
    Oracle,
    Temple,
    Palisade,
    Beacon,
    Barracks,
    Bastion,
    Gate,
    Forester,
    WinterHall,
    SeedPavilion,
    WaterTerrace,
    OlivePress,
}

pub struct Spec {
    pub name: &'static str,
    pub purpose: &'static str,
    pub branch: Branch,
    pub unlock: Tech,
    pub size: i16,
    pub stock: Stock,
    pub goods: Goods,
}

impl Civic {
    pub const ALL: [Self; 27] = [
        Self::Lodge,
        Self::Courtyard,
        Self::Nursery,
        Self::Infirmary,
        Self::Bath,
        Self::Sawmill,
        Self::Masonry,
        Self::Bakery,
        Self::Forge,
        Self::Port,
        Self::Garden,
        Self::Tavern,
        Self::Hall,
        Self::Theater,
        Self::Archive,
        Self::Oracle,
        Self::Temple,
        Self::Palisade,
        Self::Beacon,
        Self::Barracks,
        Self::Bastion,
        Self::Gate,
        Self::Forester,
        Self::WinterHall,
        Self::SeedPavilion,
        Self::WaterTerrace,
        Self::OlivePress,
    ];

    pub fn regional(biome: Biome) -> Self {
        [
            Self::Forester,
            Self::WinterHall,
            Self::SeedPavilion,
            Self::WaterTerrace,
            Self::OlivePress,
        ][biome as usize]
    }

    pub fn available_in(self, biome: Biome) -> bool {
        !matches!(
            self,
            Self::Forester
                | Self::WinterHall
                | Self::SeedPavilion
                | Self::WaterTerrace
                | Self::OlivePress
        ) || self == Self::regional(biome)
    }

    pub fn production(self) -> Option<&'static str> {
        match self {
            Self::Sawmill => Some("Bretter sägen"),
            Self::Masonry => Some("Quader hauen"),
            Self::Forge => Some("Werkzeug schmieden"),
            Self::OlivePress => Some("Oliven pressen"),
            _ => None,
        }
    }

    pub fn beds(self) -> usize {
        match self {
            Self::Lodge => 3,
            Self::Courtyard => 5,
            _ => 0,
        }
    }

    pub fn spec(self) -> Spec {
        use Branch::*;
        use Tech::*;
        let (name, purpose, branch, unlock, size, stock, goods) = match self {
            Self::Lodge => (
                "Familienhaus",
                "Drei Wohnplätze. Bei Nahrung und guter Stimmung werden Kinder geboren.",
                Homes,
                Foundations,
                2,
                [11, 4, 0],
                [0, 0, 0],
            ),
            Self::Courtyard => (
                "Hofhaus",
                "Fünf Wohnplätze und ein geschützter Innenhof für die Nachbarschaft.",
                Homes,
                Families,
                3,
                [13, 8, 0],
                [2, 0, 0],
            ),
            Self::Nursery => (
                "Kinderhaus",
                "Familien brauchen weniger Vorräte, und Kinder wachsen schneller auf.",
                Homes,
                Families,
                2,
                [7, 5, 2],
                [0, 0, 0],
            ),
            Self::Infirmary => (
                "Heilstube",
                "Stärkt die Stimmung und beschleunigt die Erholung des Nilpferds.",
                Homes,
                Foundations,
                2,
                [8, 6, 0],
                [0, 0, 0],
            ),
            Self::Bath => (
                "Badhaus",
                "Ein Brunnen in der Nähe versorgt das Bad und hebt die Stimmung.",
                Homes,
                Waterworks,
                3,
                [9, 12, 0],
                [1, 3, 0],
            ),
            Self::Sawmill => (
                "Sägewerk",
                "Verarbeitet Holz zu Brettern.",
                Industry,
                Workshops,
                2,
                [8, 4, 0],
                [0, 0, 0],
            ),
            Self::Masonry => (
                "Steinhof",
                "Verarbeitet Stein zu Quadern.",
                Industry,
                Workshops,
                2,
                [5, 7, 0],
                [0, 0, 0],
            ),
            Self::Bakery => (
                "Bäckerei",
                "Backt nach der ersten Ernte aus Holz und Getreide zusätzliche Nahrung.",
                Industry,
                Workshops,
                2,
                [7, 6, 0],
                [0, 0, 0],
            ),
            Self::Forge => (
                "Schmiede",
                "Formt Baustoffe zu Werkzeug für Wache und Ausbau.",
                Industry,
                Smithing,
                2,
                [6, 10, 0],
                [2, 1, 0],
            ),
            Self::Port => (
                "Handelshafen",
                "Handelt am Ufer mit Werkzeug und bringt Stein und Nahrung. Dient als Sammelplatz.",
                Industry,
                Navigation,
                3,
                [12, 8, 0],
                [4, 2, 0],
            ),
            Self::Garden => (
                "Gartenplatz",
                "Hebt die Stimmung ohne laufenden Vorratsbedarf.",
                Culture,
                Commons,
                2,
                [4, 3, 0],
                [0, 0, 0],
            ),
            Self::Tavern => (
                "Schankstube",
                "Gibt aus Vorräten Mahlzeiten aus und stärkt die Gemeinschaft.",
                Culture,
                Commons,
                2,
                [8, 5, 0],
                [0, 0, 0],
            ),
            Self::Hall => (
                "Festhalle",
                "Verwandelt Vorräte in Glauben und Feierlaune.",
                Culture,
                Festivals,
                3,
                [11, 8, 0],
                [2, 0, 0],
            ),
            Self::Theater => (
                "Theater",
                "Aufführungen geben Erkenntnis und heben die Stimmung.",
                Culture,
                Festivals,
                3,
                [8, 12, 0],
                [0, 2, 0],
            ),
            Self::Archive => (
                "Archiv",
                "Schreiber tauschen Holz gegen Erkenntnis und öffnen tiefere Forschung.",
                Culture,
                Lore,
                2,
                [9, 8, 0],
                [2, 0, 0],
            ),
            Self::Oracle => (
                "Orakel",
                "Beschleunigt das Aufladen der Wunderkraft.",
                Culture,
                Lore,
                1,
                [5, 10, 0],
                [0, 2, 0],
            ),
            Self::Temple => (
                "Tempel",
                "Zieht Glauben an und vergrößert den Vorrat an Wunderkraft.",
                Culture,
                Lore,
                3,
                [12, 15, 0],
                [2, 4, 0],
            ),
            Self::Palisade => (
                "Palisade",
                "Drei Palisaden am Übergang halten Überfälle auf Vorräte ab.",
                Defense,
                Palisades,
                1,
                [3, 1, 0],
                [0, 0, 0],
            ),
            Self::Gate => (
                "Palisadentor",
                "Schützt am Weg und öffnet sich selbstständig für Gläubige und Nilpferd.",
                Defense,
                Palisades,
                1,
                [5, 3, 0],
                [0, 0, 0],
            ),
            Self::Forester => (
                "Försterhof",
                "Pflegt Eichenholzvorkommen im Umkreis und trägt Holz ins Lager.",
                Industry,
                Workshops,
                2,
                [9, 5, 0],
                [0, 0, 0],
            ),
            Self::WinterHall => (
                "Winterhalle",
                "Wärmt das Dorf im Schneefall und spart Wintervorräte.",
                Culture,
                Commons,
                2,
                [11, 6, 0],
                [1, 0, 0],
            ),
            Self::SeedPavilion => (
                "Saatpavillon",
                "Felder in der Nähe keimen schneller. Das Saatgut profitiert vom Flussnebel.",
                Homes,
                Families,
                2,
                [8, 6, 0],
                [1, 0, 0],
            ),
            Self::WaterTerrace => (
                "Wasserterrasse",
                "Leitet Seewasser auf Felder im Umkreis. Schützt sie in der Trockenzeit.",
                Homes,
                Waterworks,
                2,
                [7, 10, 0],
                [0, 2, 0],
            ),
            Self::OlivePress => (
                "Olivenpresse",
                "Presst Oliven für die Feste. Bei Seebrise ist das Öl besonders gefragt und bringt Glauben.",
                Industry,
                Workshops,
                2,
                [7, 10, 0],
                [0, 1, 0],
            ),
            Self::Beacon => (
                "Signalfeuer",
                "Wachen bewegen sich im Licht des Feuers schneller.",
                Defense,
                Palisades,
                1,
                [4, 5, 0],
                [0, 0, 0],
            ),
            Self::Barracks => (
                "Wachhaus",
                "Bildet Wachen aus, die Vorräte schützen und an Nachbardörfern patrouillieren.",
                Defense,
                Militia,
                2,
                [10, 8, 0],
                [2, 0, 0],
            ),
            Self::Bastion => (
                "Bastion",
                "Schützt einen großen Umkreis und erhöht die Zahl ausbildbarer Wachen.",
                Defense,
                Fortifications,
                3,
                [8, 13, 0],
                [2, 3, 1],
            ),
        };
        Spec {
            name,
            purpose,
            branch,
            unlock,
            size,
            stock,
            goods,
        }
    }
}
