use crate::{
    game::{Building, Crop, Game, Quest, Settlement},
    play::{Play, Tool, UiAction},
    region::{Region, Tile},
    wheel::Wheel,
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
    pub fn chapter(self) -> (u8, &'static str) {
        match self {
            Self::Grab => (1, "Holz für ein erstes Dach"),
            Self::Drop => (2, "Vom Wald ins Dorf"),
            Self::House => (3, "Ein Zuhause am Ufer"),
            Self::Field => (4, "Ein Beet für die Küche"),
            Self::Rain | Self::Grow => (5, "Wasser für die Saat"),
            Self::Harvest => (6, "Die erste Ernte"),
            Self::SendHippo | Self::Delivery => (7, "Ein Auftrag für zwei"),
            Self::Pet | Self::Complete => (8, "Danke, kleines Nilpferd"),
        }
    }

    pub fn wheel(self) -> Option<(Wheel, UiAction)> {
        match self {
            Self::House => Some((Wheel::Build, UiAction::SelectBuilding(Building::House))),
            Self::Field => Some((Wheel::Build, UiAction::SelectBuilding(Building::Field))),
            Self::Rain => Some((Wheel::Miracles, UiAction::Order(Tool::Miracles, 0))),
            Self::SendHippo => Some((Wheel::Hippo, UiAction::Order(Tool::Hippo, 0))),
            _ => None,
        }
    }

    pub fn next_action(self, play: &Play, game: &Game) -> UiAction {
        if let Some((wheel, choice)) = self.wheel() {
            let equipped = match choice {
                UiAction::SelectBuilding(kind) => {
                    play.tool == Tool::Build && play.building() == kind
                }
                UiAction::Order(tool, option) => play.tool == tool && play.option == option,
                _ => false,
            };
            if !equipped {
                return UiAction::QuickWheel(wheel);
            }
        } else if matches!(self, Self::Grow | Self::Delivery | Self::Complete) {
            return UiAction::Guide;
        } else if play.tool != Tool::Hand {
            return UiAction::Tool(Tool::Hand);
        }
        // A player may place the hut or field on any legal plot. Other lessons teach a
        // particular interaction, so recover the target if the pointer has wandered.
        if play.lesson_target(game) != Some(play.aim)
            && !(matches!(self, Self::House | Self::Field)
                && game.building_plan(play.building(), play.aim).is_ok())
        {
            return UiAction::Guide;
        }
        UiAction::Confirm
    }

    pub fn instruction(self, action: UiAction, pad: bool) -> &'static str {
        match action {
            UiAction::QuickWheel(Wheel::Build) if self == Self::Field => {
                "Öffne das Baurad und wähle das Feld."
            }
            UiAction::QuickWheel(Wheel::Build) => "Öffne das Baurad und wähle die Hütte.",
            UiAction::QuickWheel(Wheel::Miracles) => "Öffne das Wunderrad und wähle Regen.",
            UiAction::QuickWheel(Wheel::Hippo) => "Öffne das Nilpferdrad und wähle Sammeln.",
            UiAction::Tool(_) => "Nimm die Pfote, um das Ziel zu berühren.",
            UiAction::Guide if !matches!(self, Self::Grow | Self::Delivery | Self::Complete) => {
                "Mit „Ziel zeigen“ findest du zur Markierung zurück."
            }
            _ => match self {
                Self::Grab if pad => "Halte A am Holzstoß. Lass los, um das Holz zu nehmen.",
                Self::Grab => {
                    "Halte den markierten Holzstoß oder unten „Greifen“. Lass zum Aufheben los."
                }
                Self::Drop => "Der Sammelplatz ist markiert. Lege dort dein Bündel ab.",
                Self::House => "Der Umriss zeigt, wo die Hütte stehen wird. Bestätige den Bau.",
                Self::Field => "Bestätige das Beet auf der freien Wiese am Dorfweg.",
                Self::Rain => "Der Regenkreis liegt über dem Beet. Lass es regnen.",
                Self::Grow if pad => "Die Saat wächst. Schau dich mit dem rechten Stick um.",
                Self::Grow => "Die Saat wächst. Ziehe die Insel, um dich umzusehen.",
                Self::Harvest => "Das Beet ist reif. Sammle die Nahrung für dein Dorf ein.",
                Self::SendHippo => "Die Quelle ist markiert. Schicke dein Nilpferd zum Sammeln.",
                Self::Delivery => "Dein Nilpferd sammelt und trägt das Bündel selbst ins Dorf.",
                Self::Pet => "Dein Nilpferd hat geliefert. Streichle es zur Belohnung.",
                Self::Complete => "Deine erste Kampagne ist bereit.",
            },
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
