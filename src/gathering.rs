use super::*;

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub enum Rite {
    Harvest,
    Timber,
    Stone,
    Council,
}

impl Rite {
    pub const ALL: [Self; 4] = [Self::Harvest, Self::Timber, Self::Stone, Self::Council];
    pub fn name(self) -> &'static str {
        match self {
            Self::Harvest => "Erntetanz",
            Self::Timber => "Holzzug",
            Self::Stone => "Steinzug",
            Self::Council => "Inselrat",
        }
    }
    pub fn icon(self) -> usize {
        match self {
            Self::Harvest => 2,
            Self::Timber => 0,
            Self::Stone => 1,
            Self::Council => 3,
        }
    }
    pub fn material(self) -> Option<Material> {
        match self {
            Self::Timber => Some(Material::Wood),
            Self::Stone => Some(Material::Stone),
            _ => None,
        }
    }
    pub fn finish(self) -> &'static str {
        match self {
            Self::Harvest => "Ernte einbringen",
            Self::Council => "Frieden stiften",
            _ => "Gemeinsam bergen",
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Circle {
    pub kind: Rite,
    pub center: Tile,
    pub guests: Vec<(usize, Tile)>,
}

impl Circle {
    pub fn present(&self, city: &Settlement) -> (usize, bool) {
        let mut count = 0;
        let mut hippo = false;
        for &(i, spot) in &self.guests {
            if city
                .walkers
                .get(i)
                .is_some_and(|w| w.tile() == spot && w.route.is_empty())
            {
                count += 1;
                hippo |= i == 0;
            }
        }
        (count, hippo)
    }

    pub fn ready(&self, city: &Settlement) -> bool {
        let (count, hippo) = self.present(city);
        hippo && count >= 2
    }

    pub fn status(&self, city: &Settlement) -> String {
        let (count, hippo) = self.present(city);
        let what = if !hippo {
            "Nilpferd unterwegs"
        } else if count < 2 {
            "Warte auf das Dorf"
        } else {
            self.kind.finish()
        };
        format!("{count}/{} im Kreis · {what}", self.guests.len())
    }
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[serde(default)]
pub struct Gatherings {
    pub circle: Option<Circle>,
    pub rest: f32,
    pub renown: u16,
    pub hauls: u16,
    pub welcomed: [bool; 2],
}

impl Settlement {
    pub fn gathering_place(&self, region: &Region, kind: Rite, near: Tile) -> Result<Tile, String> {
        if region.tutorial {
            return Err("Dorfrufe gibt es auf den Kampagneninseln und im freien Bauen.".into());
        }
        if let Some(mat) = kind.material() {
            let inventory = mat.index();
            if self.stock[inventory] >= self.storage_cap() {
                return Err(format!(
                    "Das Lager für {} ist voll.",
                    region.kind.material(mat)
                ));
            }
            return region
                .sources
                .iter()
                .enumerate()
                .filter(|(i, s)| {
                    s.material == mat && self.nodes[*i] > 0 && self.influence(region, s.tile)
                })
                .min_by_key(|(_, s)| s.tile.distance(near))
                .map(|(_, s)| s.tile)
                .ok_or_else(|| {
                    format!(
                        "Im Einflussgebiet ist gerade kein {} verfügbar.",
                        region.kind.material(mat)
                    )
                });
        }
        if kind == Rite::Harvest {
            if self.stock[2] >= self.storage_cap() {
                return Err("Das Nahrungslager ist voll.".into());
            }
            return self
                .buildings
                .iter()
                .filter(|s| s.kind == Building::Field)
                .min_by_key(|s| s.tile.distance(near))
                .map(|s| s.tile)
                .ok_or_else(|| "Lege ein Feld für den ersten Erntetanz an.".into());
        }
        if self.quest == Quest::Bridge && self.count(Building::Shrine) > 0 {
            return Ok(region.frontier);
        }
        if self.quest == Quest::Districts
            && let Some((d, _)) = region
                .districts
                .iter()
                .zip(&self.districts)
                .filter(|(_, s)| s.outcome.is_none())
                .min_by_key(|(d, _)| d.center.distance(near))
        {
            return Ok(d.center);
        }
        Ok(region.depot)
    }

    pub(super) fn gather(&mut self, region: &Region, kind: Rite, near: Tile) -> GameResult {
        if self.gatherings.circle.is_some() {
            return Err("Es läuft schon ein Dorfruf.".into());
        }
        if self.gatherings.rest > 0. {
            return Err(format!(
                "Das Dorf ruht noch {:.0} s.",
                self.gatherings.rest.ceil()
            ));
        }
        let center = self.gathering_place(region, kind, near)?;
        let mut spots: Vec<_> = crate::region::Bounds(center.offset(-3, -3), center.offset(3, 3))
            .tiles()
            .filter(|&t| t != center && t.within(center, 3) && self.walkable(region, t))
            .collect();
        spots.sort_by_key(|s| (s.distance(center).abs_diff(2), s.1, s.0));
        let mut guests: Vec<(usize, Tile)> = Vec::new();
        let mut candidates: Vec<_> = self
            .walkers
            .iter()
            .enumerate()
            .filter(|(_, w)| !w.guard && w.childhood <= 0.)
            .collect();
        candidates.sort_by_key(|(i, w)| (*i != 0, w.bundle.is_some(), w.tile().distance(center)));
        for (i, w) in candidates {
            if let Some(n) = spots
                .iter()
                .position(|&s| self.route_to(region, w.tile(), s, false).is_some())
            {
                guests.push((i, spots.remove(n)));
                if guests.len() == 4 {
                    break;
                }
            }
        }
        if guests.len() < 2 || guests[0].0 != 0 {
            return Err("Dein Nilpferd und mindestens ein Dorfbewohner brauchen einen begehbaren Weg zum Kreis.".into());
        }
        for &(i, _) in &guests {
            let w = &mut self.walkers[i];
            if w.bundle.is_none() {
                w.route.clear();
                w.pause = 0.;
            }
        }
        self.gatherings.circle = Some(Circle {
            kind,
            center,
            guests,
        });
        Ok(format!(
            "{} beginnt · Dorf und Nilpferd sind auf dem Weg.",
            kind.name()
        ))
    }

    pub(super) fn dismiss_circle(&mut self) -> GameResult {
        let circle = self
            .gatherings
            .circle
            .take()
            .ok_or("Gerade läuft kein Dorfruf.")?;
        for (i, _) in circle.guests {
            let w = &mut self.walkers[i];
            if w.bundle.is_none() {
                w.route.clear();
                w.pause = 0.;
            }
        }
        Ok("Das Dorf geht wieder seiner Arbeit nach.".into())
    }

    pub fn council_threat(&self, region: &Region, center: Tile) -> bool {
        (self.quest == Quest::Bridge
            && center == region.frontier
            && (self.guards.iter().any(|g| *g > 0) || self.rival_active(region)))
            || region.district_at(center).is_some_and(|i| {
                self.districts[i].outcome.is_none() && self.districts[i].resolve > 0
            })
    }

    pub(super) fn finish_circle(&mut self, region: &Region, awe: bool) -> GameResult {
        let c = self
            .gatherings
            .circle
            .as_ref()
            .ok_or("Rufe zuerst dein Dorf zusammen.")?;
        let (power, hippo) = c.present(self);
        if !hippo || power < 2 {
            return Err(c.status(self));
        }
        if self.mana < 6. {
            return Err("Der Kreis braucht 6 Wunderkraft.".into());
        }
        let (kind, center) = (c.kind, c.center);
        if awe && (kind != Rite::Council || !self.council_threat(region, center)) {
            return Err("Hier widersetzt sich niemand dem Dorf.".into());
        }
        let power = power as u16;
        let result = if let Some(material) = kind.material() {
            let i = material.index();
            let limit = (3 * power).min(self.storage_cap().saturating_sub(self.stock[i]));
            let source = region
                .source_at(center)
                .filter(|&n| region.sources[n].material == material)
                .ok_or("An diesem Platz gibt es keine passende Quelle.")?;
            let total = limit.min(self.nodes[source]);
            if total == 0 {
                return Err("Quelle leer oder Lager voll. Du kannst den Kreis auflösen.".into());
            }
            self.nodes[source] -= total;
            self.stock[i] += total;
            self.lesson = Some(Lesson::Practiced(Job::Gather(source)));
            self.gatherings.hauls = self.gatherings.hauls.saturating_add(1);
            format!(
                "{power} Helfende · +{total} {} im Lager",
                region.kind.material(material)
            )
        } else if kind == Rite::Harvest {
            if self.stock[2] >= self.storage_cap() {
                return Err("Im Nahrungslager ist kein Platz.".into());
            }
            let mut fields: Vec<_> = self
                .buildings
                .iter()
                .enumerate()
                .filter(|(_, s)| s.kind == Building::Field && s.tile.within(center, 6))
                .map(|(i, s)| (s.tile.distance(center), i))
                .collect();
            fields.sort_unstable();
            let mut food = 0;
            let mut plots = 0;
            for (_, i) in fields.into_iter().take(usize::from(power - 1)) {
                if self.stock[2] >= self.storage_cap() {
                    break;
                }
                self.buildings[i].crop = Crop::Ripe;
                food += self.harvest(region, self.buildings[i].tile)?;
                plots += 1;
            }
            self.hippo_harvests = self.hippo_harvests.saturating_add(plots);
            self.lesson = Some(Lesson::Practiced(Job::Tend(center)));
            let unit = if plots == 1 { "Feld" } else { "Felder" };
            format!("{plots} {unit} geerntet · +{food} Nahrung")
        } else if awe {
            let force = (power - 1) as u8;
            let resistance = if let Some(i) = region.district_at(center) {
                &mut self.districts[i].resolve
            } else if let Some(guard) = self.guards.iter_mut().find(|g| **g > 0) {
                guard
            } else {
                &mut self.rival_resolve
            };
            let loss = force.min(*resistance);
            *resistance -= loss;
            self.society.morale = self.society.morale.saturating_sub(4);
            self.hippo_watches = self.hippo_watches.saturating_add(1);
            format!("Die Wachen weichen · −{loss} Widerstand · −4 Stimmung")
        } else {
            let gain = power.min(8 - self.gatherings.renown.min(8));
            self.gatherings.renown += gain;
            let duration = 10. * f32::from(power);
            self.shield = self.shield.max(duration);
            self.hippo_watches = self.hippo_watches.saturating_add(1);
            self.lesson = Some(Lesson::Practiced(Job::Watch(center)));
            if power >= 3
                && let Some(i) = region.district_at(center)
                && self.districts[i].outcome.is_none()
            {
                self.gatherings.welcomed[i] = true;
                format!(
                    "{} vertraut dir · Für das Bündnis fehlt nur die Dorfgabe.",
                    region.districts[i].name
                )
            } else if gain == 0 {
                format!("Schutz für {duration:.0} s")
            } else {
                format!("Schutz für {duration:.0} s · +{gain} Ansehen")
            }
        };
        if !awe {
            self.society.morale = (self.society.morale + power).min(100);
        }
        self.mana -= 6.;
        self.dismiss_circle()?;
        self.gatherings.rest = 36.;
        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn assemble(game: &mut Game) -> usize {
        for _ in 0..3600 {
            let city = game.city();
            let circle = city.gatherings.circle.as_ref().unwrap();
            let (present, hippo) = circle.present(city);
            if hippo && present == circle.guests.len() {
                return present;
            }
            game.step(0.05);
        }
        panic!(
            "Die Gruppe erreicht den Kreis auf {} nicht",
            game.region().name
        );
    }

    #[test]
    fn local_crews_haul_only_from_the_chosen_source_in_all_five_campaigns() {
        for biome in Biome::ALL {
            let mut game = Game::start(Mode::Campaign(biome));
            let aim = game.region().depot;
            game.act(Action::Call(Rite::Timber, aim)).unwrap();
            let before = game.city().mana;
            assert!(game.act(Action::Invoke(false)).is_err());
            assert_eq!(game.city().mana, before);
            let count = assemble(&mut game);
            let saved = serde_json::to_vec(&game.saved_games()).unwrap();
            let mut game = Game::resume(serde_json::from_slice(&saved).unwrap()).unwrap();
            let center = game.city().gatherings.circle.as_ref().unwrap().center;
            let source = game.region().source_at(center).unwrap();
            let city = game.city();
            let mut nodes = city.nodes.clone();
            let stock = city.stock[0];
            let mana = city.mana;
            let take = (count as u16 * 3)
                .min(city.storage_cap() - stock)
                .min(nodes[source]);
            nodes[source] -= take;
            let result = game.act(Action::Invoke(false)).unwrap();
            assert!(result.contains(&format!("+{take}")));
            assert_eq!(game.city().nodes, nodes);
            assert_eq!(game.city().stock[0], stock + take);
            assert_eq!(game.city().mana, mana - 6.);
            assert_eq!(game.city().gatherings.hauls, 1);
            assert_eq!(
                game.city().supply_lesson(game.region()),
                Some(Material::Wood)
            );
            assert!(game.act(Action::Invoke(false)).is_err());
            assert!(game.act(Action::Call(Rite::Council, aim)).is_err());
            assert!(game.city().journal.iter().any(|s| *s == result));
        }
    }

    #[test]
    fn cancelling_a_call_keeps_mana_and_the_creatures_previous_job() {
        let mut game = Game::start(Mode::Campaign(Biome::Lake));
        let at = game.region().sources[0].tile;
        game.act(Action::Assign(at, true)).unwrap();
        let job = game.city().walkers[0].job;
        let mana = game.city().mana;
        game.act(Action::Call(Rite::Council, at)).unwrap();
        game.act(Action::DismissCircle).unwrap();
        assert_eq!(game.city().walkers[0].job, job);
        assert_eq!(game.city().mana, mana);
        assert_eq!(game.city().gatherings.rest, 0.);
        game.act(Action::Call(Rite::Stone, at)).unwrap();
        game.act(Action::Rest).unwrap();
        assert!(game.city().gatherings.circle.is_none());
        assert!(game.city().walkers[0].job.is_none());
    }

    #[test]
    fn zero_room_and_insufficient_mana_leave_the_ritual_and_its_source_intact() {
        let mut game = Game::start(Mode::Campaign(Biome::Terraces));
        let at = game.region().depot;
        game.act(Action::Call(Rite::Stone, at)).unwrap();
        assemble(&mut game);
        let city = game.campaign.settlements[0].as_mut().unwrap();
        let cap = city.storage_cap();
        city.stock[1] = cap;
        let nodes = city.nodes.clone();
        let mana = city.mana;
        assert!(game.act(Action::Invoke(false)).is_err());
        assert_eq!(game.city().mana, mana);
        let city = game.campaign.settlements[0].as_mut().unwrap();
        city.stock[1] -= 1;
        city.mana = 5.;
        assert!(game.act(Action::Invoke(false)).is_err());
        assert_eq!(game.city().nodes, nodes);
        assert_eq!(game.city().stock[1], cap - 1);
        game.campaign.settlements[0].as_mut().unwrap().mana = 6.;
        game.act(Action::Invoke(false)).unwrap();
        assert_eq!(game.city().stock[1], cap);
        assert_eq!(game.city().mana, 0.);
    }

    #[test]
    fn a_council_can_win_a_villages_trust_or_reduce_its_resistance() {
        let mut game = Game::start(Mode::Sandbox(Biome::Fjord));
        let region = game.region().clone();
        let at = region.districts[0].center;
        // Use this island's open crossing and populated village for the district encounter.
        let city = game.campaign.settlements[0].as_mut().unwrap();
        city.quest = Quest::Districts;
        city.districts = [DistrictStatus::default(); 2];
        game.act(Action::Call(Rite::Council, at)).unwrap();
        assert!(assemble(&mut game) >= 3);
        assert_eq!(game.city().gatherings.circle.as_ref().unwrap().center, at);
        let mut sterner = Game::resume(game.saved_games()).unwrap();
        let resolve = sterner.city().districts[0].resolve;
        let morale = sterner.city().society.morale;
        sterner.act(Action::Invoke(true)).unwrap();
        assert!(sterner.city().districts[0].resolve < resolve);
        assert_eq!(sterner.city().society.morale, morale - 4);
        assert!(!sterner.city().gatherings.welcomed[0]);
        game.act(Action::Invoke(false)).unwrap();
        assert!(game.city().gatherings.welcomed[0]);
        let expected = region.districts[0].gift;
        let city = game.campaign.settlements[0].as_mut().unwrap();
        city.stock = [0; 3];
        assert!(game.act(Action::Settle(0)).is_err());
        game.campaign.settlements[0].as_mut().unwrap().stock = expected;
        game.act(Action::Settle(0)).unwrap();
        assert_eq!(game.city().stock, [0; 3]);
        assert_eq!(game.city().districts[0].outcome, Some(Outcome::Alliance));
    }
}
