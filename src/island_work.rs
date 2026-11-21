use super::*;
use crate::region::Bounds;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum WorkStage {
    #[default]
    Offered,
    Supplies,
    Raising,
    Opening,
    Running,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Dedication {
    Companion,
    Talent,
    Town,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct IslandWork {
    pub stage: WorkStage,
    pub delivered: u16,
    pub pinned: bool,
    pub dedication: Option<Dedication>,
    pub rounds: u32,
}

pub struct Blueprint {
    pub title: &'static str,
    pub request: &'static str,
    pub raise: &'static str,
    pub material: Material,
    pub amount: u16,
    pub partner: Building,
}

impl Blueprint {
    pub fn on(r: &Region) -> Self {
        let (title, request, raise, material, partners) = match r.kind {
            Biome::Valley => (
                "Die Bachmühle",
                "Mira mahlt das Korn von Hand. Mit einem Wasserrad kann sie wieder das ganze Dorf versorgen.",
                "Mühlrad einhängen",
                Material::Wood,
                [Building::Field, Building::Store, Building::Granary],
            ),
            Biome::Fjord => (
                "Das Wachtfeuer",
                "Im Nebel verlieren die Fischer die Küste. Ein Feuer über den Dächern soll die Heimfahrt und das Dorf sichern.",
                "Feuerkorb aufrichten",
                Material::Stone,
                [Building::House, Building::Workshop, Building::Tower],
            ),
            Biome::Terraces => (
                "Der Wasserpavillon",
                "Am oberen Hang bleiben die Reispflanzen trocken. Ein Schöpfrad hebt Wasser in die Rinnen der Terrassen.",
                "Schöpfrad einsetzen",
                Material::Wood,
                [Building::Field, Building::Well, Building::Granary],
            ),
            Biome::Lake => (
                "Der Hebebrunnen",
                "Die Wasserträgerinnen ziehen jeden Krug einzeln hoch. Ein langer Schwingbalken erschließt die Quelle für den Schrein.",
                "Brunnenhebel heben",
                Material::Wood,
                [Building::Store, Building::Well, Building::Shrine],
            ),
            Biome::Aegean => (
                "Das Hafenlicht",
                "Steinschiffe meiden die Riffe bei Dämmerung. Ein Lichtturm soll ihnen den Weg zum Kai zeigen.",
                "Laterne aufsetzen",
                Material::Stone,
                [Building::Store, Building::Workshop, Building::Market],
            ),
        };
        let n = r.chapter().min(2);
        Self {
            title,
            request,
            raise,
            material,
            amount: 6 + 3 * n as u16,
            partner: partners[n],
        }
    }

    pub fn effect(r: &Region, double: bool) -> String {
        let n = if double { 2 } else { 1 };
        match r.kind {
            Biome::Valley => format!("+{} Nahrung", 2 * n),
            Biome::Fjord => format!("{} s Schutz vor Überfällen", 8 * n),
            Biome::Terraces => format!("{} s Regen für wachsende Felder", 6 * n),
            Biome::Lake => format!("+{} Wunderkraft", 4 * n),
            Biome::Aegean => format!("+{} Stein", 2 * n),
        }
    }
}

impl IslandWork {
    pub(super) fn valid(&self, r: &Region) -> bool {
        let amount = Blueprint::on(r).amount;
        (match self.stage {
            WorkStage::Offered => self.delivered == 0 && !self.pinned,
            WorkStage::Supplies => !r.tutorial && self.delivered < amount,
            WorkStage::Raising | WorkStage::Opening => !r.tutorial && self.delivered == amount,
            WorkStage::Running => !r.tutorial && self.delivered == amount && !self.pinned,
        }) && (self.dedication.is_some() == (self.stage == WorkStage::Running))
            && (self.rounds == 0 || self.stage == WorkStage::Running)
    }
}

impl Settlement {
    pub fn work_partner(&self, r: &Region) -> bool {
        let kind = Blueprint::on(r).partner;
        self.buildings
            .iter()
            .any(|s| s.kind == kind && self.connected(r, s))
    }
    pub fn work_connection(&self, r: &Region, partner: Building) -> Option<(Tile, Tile, Stock)> {
        let mut best = None;
        for end in self
            .buildings
            .iter()
            .filter(|s| s.kind == partner)
            .flat_map(|s| s.kind.tiles(s.tile))
            .flat_map(Tile::neighbors)
            .filter(|&t| self.walkable(r, t))
        {
            for start in Bounds(end.offset(-12, -12), end.offset(12, 12))
                .tiles()
                .filter(|&t| {
                    t.distance(end) <= MAX_ROAD_RUN && r.bounds.contains(t) && self.road_at(r, t)
                })
            {
                if let Ok((cells, price)) = self.road_plan(r, start, end) {
                    let key = (cells.len(), start.distance(end));
                    if best.is_none_or(|(old, _, _, _)| key < old) {
                        best = Some((key, start, end, price));
                    }
                }
            }
        }
        best.map(|(_, start, end, price)| (start, end, price))
    }
    pub fn builder_present(&self, r: &Region) -> bool {
        let h = &self.walkers[0];
        h.tile().distance(r.depot) <= 1 && h.bundle.is_none() && h.route.is_empty()
    }
    pub fn builder_en_route(&self, r: &Region) -> bool {
        !self.builder_present(r)
            && matches!(self.walkers[0].job, Some(Job::Move(t)) if t.distance(r.depot) == 1)
    }
    pub fn work_hint(&self, r: &Region) -> String {
        let b = Blueprint::on(r);
        match self.work.stage {
            WorkStage::Offered => b.request.into(),
            WorkStage::Supplies => format!(
                "{} liefern · {}/{}",
                r.kind.material(b.material),
                self.work.delivered,
                b.amount
            ),
            WorkStage::Raising if self.builder_en_route(r) => {
                "Nilpferd unterwegs zum Bauplatz".into()
            }
            WorkStage::Raising if !self.builder_present(r) => "Nilpferd zum Bauplatz führen".into(),
            WorkStage::Raising if self.energy < 12. => "Nilpferd füttern · 12 Kraft nötig".into(),
            WorkStage::Raising => b.raise.into(),
            WorkStage::Opening if self.work_partner(r) => "Widmung wählen und Werk eröffnen".into(),
            WorkStage::Opening => format!("{} am Dorfweg anschließen", b.partner.name()),
            WorkStage::Running if !self.work_partner(r) => {
                format!("Betrieb ruht · Weg zum Gebäude {} fehlt", b.partner.name())
            }
            WorkStage::Running => format!(
                "Betrieb aktiv · {} je Versorgungsrunde",
                Blueprint::effect(r, self.work.dedication == Some(Dedication::Town))
            ),
        }
    }

    pub(super) fn accept_work(&mut self, r: &Region) -> GameResult {
        if r.tutorial || self.work.stage != WorkStage::Offered {
            return Err("Dieser Auftrag ist hier nicht verfügbar.".into());
        }
        self.work.stage = WorkStage::Supplies;
        self.work.pinned = true;
        Ok(format!(
            "{} angenommen · {}",
            Blueprint::on(r).title,
            self.work_hint(r)
        ))
    }
    pub(super) fn supply_work(&mut self, r: &Region) -> GameResult {
        if self.work.stage != WorkStage::Supplies {
            return Err("Hier ist keine Lieferung offen.".into());
        }
        let b = Blueprint::on(r);
        let i = b.material.index();
        let take = (b.amount - self.work.delivered).min(self.stock[i]);
        if take == 0 {
            return Err(format!(
                "Sammle {} und lege es im Lager ab.",
                r.kind.material(b.material)
            ));
        }
        self.stock[i] -= take;
        self.work.delivered += take;
        if self.work.delivered == b.amount {
            self.work.stage = WorkStage::Raising;
        }
        Ok(format!(
            "{take} {} verbaut · {}",
            r.kind.material(b.material),
            self.work_hint(r)
        ))
    }
    pub(super) fn summon_builder(&mut self, r: &Region) -> GameResult {
        if self.work.stage != WorkStage::Raising {
            return Err("Der Einsatz am Bauplatz ist noch nicht offen.".into());
        }
        let tile = r
            .depot
            .neighbors()
            .into_iter()
            .filter(|&t| self.walkable(r, t))
            .filter_map(|t| {
                self.travel_route(r, self.walkers[0].tile(), t, false, true)
                    .map(|path| (path.len(), t))
            })
            .min_by_key(|(dist, _)| *dist)
            .map(|(_, t)| t)
            .ok_or("Öffne einen Weg zum Sammelplatz.")?;
        self.move_hippo(r, tile)?;
        Ok("Dein Nilpferd kommt zum Sammelplatz. Dort kannst du mit ihm heben.".into())
    }
    pub(super) fn raise_work(&mut self, r: &Region, gifts: &mut Gifts) -> GameResult {
        if self.work.stage != WorkStage::Raising || !self.builder_present(r) {
            return Err("Das Nilpferd muss mit freien Händen am Sammelplatz stehen.".into());
        }
        if self.energy < 12. {
            return Err("Zum Heben braucht dein Nilpferd 12 Kraft.".into());
        }
        self.energy -= 12.;
        gifts.creature.craft = gifts.creature.craft.saturating_add(1);
        self.work.stage = WorkStage::Opening;
        Ok(format!(
            "{} errichtet · +1 Kreaturenerfahrung. Jetzt den Betrieb vorbereiten.",
            Blueprint::on(r).title
        ))
    }
    pub(super) fn dedicate_work(
        &mut self,
        r: &Region,
        gifts: &mut Gifts,
        choice: Dedication,
    ) -> GameResult {
        if self.work.stage != WorkStage::Opening {
            return Err("Dieses Werk kann gerade nicht eröffnet werden.".into());
        }
        if !self.work_partner(r) {
            return Err(self.work_hint(r));
        }
        let reward = match choice {
            Dedication::Companion => {
                gifts.creature.craft = gifts.creature.craft.saturating_add(4);
                "+4 Kreaturenerfahrung"
            }
            Dedication::Talent => {
                gifts.earn();
                "+1 Talentpunkt"
            }
            Dedication::Town => "Doppelte Leistung für die Stadt",
        };
        self.work.dedication = Some(choice);
        self.work.pinned = false;
        self.work.stage = WorkStage::Running;
        Ok(format!("{} eröffnet · {reward}", Blueprint::on(r).title))
    }
    pub(super) fn operate_work(&mut self, r: &Region, gifts: Gifts) {
        if self.work.stage != WorkStage::Running || !self.work_partner(r) {
            return;
        }
        let n = if self.work.dedication == Some(Dedication::Town) {
            2
        } else {
            1
        };
        match r.kind {
            Biome::Valley | Biome::Aegean => {
                let i = if r.kind == Biome::Valley {
                    Material::Food
                } else {
                    Material::Stone
                }
                .index();
                self.stock[i] =
                    self.stock[i].max(self.stock[i].saturating_add(2 * n).min(self.storage_cap()));
            }
            Biome::Fjord => self.shield = self.shield.max(f32::from(8 * n)),
            Biome::Terraces => self.rainfall = self.rainfall.max(f32::from(6 * n)),
            Biome::Lake => {
                self.mana = self
                    .mana
                    .max((self.mana + f32::from(4 * n)).min(self.mana_cap(r, gifts)))
            }
        }
        if self.work.rounds == 0 {
            self.note(format!(
                "{} · Erste Versorgungsrunde: {}",
                Blueprint::on(r).title,
                Blueprint::effect(r, n == 2)
            ));
        }
        self.work.rounds = self.work.rounds.saturating_add(1);
    }
}
