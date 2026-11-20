use crate::{
    game::{Building, Crop, Game, Quest, Settlement},
    region::{Region, Tile},
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Step {
    Grab,
    Drop,
    House,
    Field,
    Rain,
    Grow,
    Harvest,
    SendHippo,
    Delivery,
    Pet,
    Complete,
}

impl Step {
    pub fn hint(self) -> &'static str {
        match self {
            Self::Grab => "Halten greift bis zu 18 Holz.",
            Self::Drop => "Zum markierten Sammelplatz.",
            Self::House => "Bestätige den freien Bauplatz.",
            Self::Field => "Baue am markierten Platz.",
            Self::Rain => "Der Regenkreis trifft das Feld.",
            Self::Grow => "Die Saat reift von allein.",
            Self::Harvest => "Wähle die reife Saat mit der Pfote.",
            Self::SendHippo => "Bestätige Sammeln an der Quelle.",
            Self::Delivery => "Es bringt das Bündel zum Lager.",
            Self::Pet => "Wähle das Nilpferd mit der Pfote.",
            Self::Complete => "Jetzt ist die Kampagne offen.",
        }
    }

    pub fn current(city: &Settlement) -> Self {
        if city.quest == Quest::Complete {
            return Self::Complete;
        }
        if city.hand_deliveries == 0 {
            return if city.held.is_some() {
                Self::Drop
            } else {
                Self::Grab
            };
        }
        if city.count(Building::House) == 0 {
            return Self::House;
        }
        if city.count(Building::Field) == 0 {
            return Self::Field;
        }
        if city.harvests == 0 {
            let mut growing = false;
            for field in city
                .buildings
                .iter()
                .filter(|site| site.kind == Building::Field)
            {
                match field.crop {
                    Crop::Ripe => return Self::Harvest,
                    Crop::Growing(_) => growing = true,
                    Crop::Dry => {}
                }
            }
            return if growing { Self::Grow } else { Self::Rain };
        }
        if city.hippo_deliveries > 0 {
            if city.care_interactions > 0 {
                Self::Complete
            } else {
                Self::Pet
            }
        } else if city.walkers[0]
            .job
            .is_some_and(|job| matches!(job, crate::game::Job::Gather(_)))
        {
            Self::Delivery
        } else {
            Self::SendHippo
        }
    }

    pub fn text(self) -> (&'static str, &'static str) {
        match self {
            Self::Grab => (
                "Holz greifen",
                "Wähle den markierten Holzstoß. Tippe auf Greifen oder halte den Knopf für ein größeres Bündel.",
            ),
            Self::Drop => (
                "Bündel ablegen",
                "Wähle den markierten Sammelplatz und tippe auf Ablegen.",
            ),
            Self::House => (
                "Eine Hütte bauen",
                "Der Grundriss zeigt den Bauplatz. Bestätige die Hütte auf der freien Wiese.",
            ),
            Self::Field => (
                "Ein Feld anlegen",
                "Platziere ein Feld auf freiem Boden neben dem Dorfweg.",
            ),
            Self::Rain => (
                "Regen wirken",
                "Halte den Regenkreis über das Feld und bestätige den Regenschauer.",
            ),
            Self::Grow => (
                "Die Saat wächst",
                "Warte, bis das Feld reif ist. Du kannst dabei die Kamera bewegen.",
            ),
            Self::Harvest => (
                "Das Feld ernten",
                "Wähle das reife Feld mit der Pfote. Tippe auf Ernten.",
            ),
            Self::SendHippo => (
                "Das Nilpferd beauftragen",
                "Bestätige Sammeln an der markierten Quelle. Das Nilpferd trägt das Bündel zum Lager.",
            ),
            Self::Delivery => (
                "Das Nilpferd trägt",
                "Schau zu, wie das Nilpferd sammelt und seine erste Ladung ablegt.",
            ),
            Self::Pet => (
                "Das Nilpferd streicheln",
                "Wähle dein Nilpferd mit der Pfote und streichle es für seine erste Lieferung.",
            ),
            Self::Complete => (
                "Übung abgeschlossen",
                "Wähle eine Kampagne, um deine erste Insel zu besiedeln.",
            ),
        }
    }

    pub fn tile(self, game: &Game) -> Tile {
        let region = game.region();
        let field = |ripe| {
            game.city()
                .buildings
                .iter()
                .filter(|site| site.kind == Building::Field)
                .min_by_key(|site| ripe && !matches!(site.crop, Crop::Ripe))
                .map_or(region.start, |site| site.tile)
        };
        match self {
            Self::Grab => region.sources[0].tile,
            Self::Drop => region.depot,
            Self::House => plot(game, Building::House, Tile(19, 20)),
            Self::Field => plot(game, Building::Field, Tile(16, 23)),
            Self::Rain | Self::Grow => field(false),
            Self::Harvest => field(true),
            Self::SendHippo => region.sources[1].tile,
            Self::Delivery | Self::Pet => game.city().walkers[0].tile(),
            Self::Complete => region.start,
        }
    }
}

fn plot(game: &Game, kind: Building, preferred: Tile) -> Tile {
    let region: &Region = game.region();
    region
        .bounds
        .tiles()
        .filter(|tile| tile.within(region.start, 7))
        .filter(|&tile| game.building_plan(kind, tile).is_ok())
        .min_by_key(|&tile| tile.distance(preferred))
        .unwrap_or(preferred)
}
