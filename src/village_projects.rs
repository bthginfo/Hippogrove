use super::*;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Appeal {
    pub village: usize,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Pilgrims {
    pub from: usize,
    pub to: usize,
    pub tile: Tile,
    path: VecDeque<Tile>,
    stride: f32,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(default)]
pub struct Projects {
    pub appeal: Option<Appeal>,
    pub pilgrims: Option<Pilgrims>,
    generation: usize,
    rest: f32,
    pub helped: u16,
}
impl Default for Projects {
    fn default() -> Self {
        Self {
            appeal: None,
            pilgrims: None,
            generation: 0,
            rest: 40.,
            helped: 0,
        }
    }
}
pub struct Need {
    pub title: &'static str,
    pub request: &'static str,
    pub feat: &'static str,
    pub material: Material,
}
impl Need {
    pub fn on(biome: Biome) -> Self {
        let (title, request, feat, material) = match biome {
            Biome::Valley => (
                "Hochwasser",
                "Der Mühlbach steigt. Die Familien müssen über die Furt.",
                "Familien ans Ufer tragen",
                Material::Wood,
            ),
            Biome::Fjord => (
                "Eingeschneiter Pass",
                "Eine Schneewehe hat den Winterweg versperrt. Reisende sitzen fest.",
                "Schneise durch den Schnee bahnen",
                Material::Food,
            ),
            Biome::Terraces => (
                "Gebrochener Damm",
                "Der Damm der Reisterrassen gibt nach. Wasser reißt die Setzlinge mit.",
                "Bresche mit Felsblöcken schließen",
                Material::Stone,
            ),
            Biome::Lake => (
                "Fähre im Schilf",
                "Die Papyrusfähre steckt im Schlick. Die Fährleute brauchen Hilfe.",
                "Fähre ins tiefe Wasser ziehen",
                Material::Wood,
            ),
            Biome::Aegean => (
                "Sturm im Hafen",
                "Fischerboote treiben auf das Riff. Am Kai fehlen helfende Hände.",
                "Boote in die Bucht ziehen",
                Material::Wood,
            ),
        };
        Self {
            title,
            request,
            feat,
            material,
        }
    }
}
impl Settlement {
    pub fn aid_ready(&self, r: &Region, i: usize, hippo: bool) -> Result<(), String> {
        if !self
            .projects
            .appeal
            .as_ref()
            .is_some_and(|a| a.village == i)
        {
            return Err("Hier liegt gerade kein Hilferuf vor.".into());
        }
        let visitor = self.visitor(i, hippo).filter(|w| self.visiting(w, r, i));
        if visitor.is_none() {
            return Err(if hippo {
                "Schick dein Nilpferd und warte am Dorfplatz."
            } else {
                "Schick einen Gläubigen und warte am Dorfplatz."
            }
            .into());
        }
        if hippo && self.energy < 18. {
            return Err("Dein Nilpferd braucht 18 Kraft. Füttere es oder lass es ruhen.".into());
        }
        let m = Need::on(r.kind).material;
        if !hippo && self.stock[m.index()] < 4 {
            return Err(format!("Es fehlen Vorräte für 4 {}.", r.kind.material(m)));
        }
        Ok(())
    }
    pub(super) fn aid(&mut self, r: &Region, i: usize, hippo: bool) -> GameResult {
        self.aid_ready(r, i, hippo)?;
        if hippo {
            self.energy -= 18.;
            self.contest.creature_visits = self.contest.creature_visits.saturating_add(1);
        } else {
            self.stock[Need::on(r.kind).material.index()] -= 4;
            self.contest.towns[i].protection = 60.;
        }
        self.projects.appeal = None;
        self.projects.rest = 170.;
        self.projects.helped = self.projects.helped.saturating_add(1);
        let gain = (100. - self.contest.towns[i].trust).min(if hippo { 55. } else { 35. });
        self.change_trust(r, i, gain);
        self.faith = self.faith.saturating_add(4);
        let extra = if hippo {
            "Kreaturenerfahrung"
        } else {
            "60 s Rückhalt"
        };
        Ok(format!(
            "{} gerettet · +4 Glauben · +{gain:.0} Vertrauen · {extra}",
            r.villages[i].name
        ))
    }
    pub fn sponsor(&self, r: &Region, to: usize) -> Result<usize, String> {
        let v = r.villages.get(to).ok_or("Wähle ein Ziel.")?;
        if self.projects.pilgrims.is_some() {
            return Err("Ein Pilgerzug ist bereits unterwegs.".into());
        }
        if self.contest.towns[to].side == Allegiance::Ours {
            return Err("Wähle ein neutrales oder gegnerisches Ziel.".into());
        }
        if self.stock[2] < 4 {
            return Err("Der Wegproviant kostet 4 Nahrung.".into());
        }
        self.contest
            .towns
            .iter()
            .enumerate()
            .filter(|(i, t)| {
                *i != to && t.side == Allegiance::Ours && t.trust >= 50. && t.pilgrim_rest <= 0.
            })
            .min_by_key(|(i, _)| r.villages[*i].center.distance(v.center))
            .map(|(i, _)| i)
            .ok_or_else(|| {
                "Ein eigenes Dorf mit 50 % Bindung muss bereit sein. Nach jedem Zug ruht es 55 s."
                    .into()
            })
    }
    pub(super) fn procession(&mut self, r: &Region, to: usize) -> GameResult {
        let from = self.sponsor(r, to)?;
        let target = r.villages[to].center;
        let (tile, path) = r.villages[from]
            .center
            .neighbors()
            .into_iter()
            .filter(|&t| self.walkable(r, t))
            .find_map(|t| self.travel_route(r, t, target, true, true).map(|p| (t, p)))
            .ok_or("Von dort führt kein offener Pilgerweg zum Ziel.")?;
        self.stock[2] -= 4;
        self.contest.towns[from].trust -= 20.;
        self.contest.towns[from].pilgrim_rest = 55.;
        self.projects.pilgrims = Some(Pilgrims {
            from,
            to,
            tile,
            path,
            stride: 0.,
        });
        Ok(format!(
            "{} schickt Pilger nach {}. Empfange sie am Dorfplatz.",
            r.villages[from].name, r.villages[to].name
        ))
    }
    fn march(&mut self, r: &Region, dt: f32) {
        let Some(mut p) = self.projects.pilgrims.take() else {
            return;
        };
        p.stride -= dt;
        if p.stride <= 0.
            && let Some(next) = p.path.pop_front()
        {
            if self.walkable(r, next) || r.ford(next) {
                p.tile = next;
                p.stride = 0.38;
            } else if let Some(path) =
                self.travel_route(r, p.tile, r.villages[p.to].center, true, true)
            {
                p.path = path;
                p.stride = 0.5;
            } else {
                self.announce_village(format!(
                    "Weg versperrt · Die Pilger aus {} kehren um.",
                    r.villages[p.from].name
                ));
                return;
            }
        }
        if p.path.is_empty() {
            let welcome = self.walkers.iter().any(|w| self.visiting(w, r, p.to));
            let gain = (100. - self.contest.towns[p.to].trust).min(if welcome { 55. } else { 30. });
            self.change_trust(r, p.to, gain);
            self.announce_village(format!(
                "{} · {} · +{gain:.0} Vertrauen",
                r.villages[p.to].name,
                if welcome {
                    "Pilger persönlich empfangen"
                } else {
                    "Pilger angekommen"
                }
            ));
        } else {
            self.projects.pilgrims = Some(p);
        }
    }
    pub(super) fn projects_tick(&mut self, r: &Region, dt: f32) {
        if r.villages.is_empty() {
            return;
        }
        for town in &mut self.contest.towns {
            town.protection = (town.protection - dt).max(0.);
            town.pilgrim_rest = (town.pilgrim_rest - dt).max(0.);
        }
        self.march(r, dt);
        if self.settling_in()
            || self.projects.appeal.is_some()
            || self.projects.pilgrims.is_some()
            || self.raid_warning(r)
        {
            return;
        }
        let neighbors: Vec<_> = (0..r.villages.len())
            .filter(|&i| {
                self.contest.towns[i].side == Allegiance::Ours
                    || self.walkers.iter().any(|w| self.visiting(w, r, i))
            })
            .collect();
        if neighbors.is_empty() {
            return;
        }
        self.projects.rest -= dt;
        if self.projects.rest <= 0. {
            let village = neighbors[self.projects.generation % neighbors.len()];
            self.projects.generation = self.projects.generation.wrapping_add(1);
            self.projects.appeal = Some(Appeal { village });
            self.note(format!(
                "{} bittet um Hilfe bei „{}“.",
                r.villages[village].name,
                Need::on(r.kind).title
            ));
        }
    }

    pub(super) fn defer_appeal(&mut self, i: usize) -> GameResult {
        if !self
            .projects
            .appeal
            .as_ref()
            .is_some_and(|a| a.village == i)
        {
            return Err("Diese Dorfbitte ist bereits erledigt.".into());
        }
        self.projects.appeal = None;
        self.projects.rest = 170.;
        Ok("Dorfbitte zurückgestellt. Du kannst in Ruhe weiterspielen.".into())
    }

    pub(super) fn valid_projects(&self, r: &Region) -> bool {
        self.projects
            .appeal
            .as_ref()
            .is_none_or(|a| a.village < r.villages.len())
            && self.projects.pilgrims.as_ref().is_none_or(|p| {
                p.from < r.villages.len()
                    && p.to < r.villages.len()
                    && r.bounds.contains(p.tile)
                    && p.path.iter().all(|t| r.bounds.contains(*t))
            })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn arrive(city: &mut Settlement, r: &Region, i: usize, hippo: bool) {
        let w = city.walkers.iter_mut().find(|w| w.hippo == hippo).unwrap();
        w.job = Some(Job::Visit(i));
        w.position = r.villages[i].center.local().to_array();
        w.route.clear();
        w.bundle = None;
    }

    #[test]
    fn an_appeal_requires_a_present_visitor_and_spends_and_rewards_only_once() {
        let game = Game::start(Mode::Campaign(Biome::Valley));
        let r = game.region();
        let i = 0;
        for hippo in [true, false] {
            let mut city = game.city().clone();
            city.projects.appeal = Some(Appeal { village: i });
            city.contest.towns[i].side = Allegiance::Ours;
            city.contest.towns[i].trust = 96.;
            city.energy = 17.;
            let m = Need::on(r.kind).material.index();
            city.stock[m] = 3;
            assert!(city.aid(r, i, hippo).is_err());
            arrive(&mut city, r, i, hippo);
            assert!(city.aid(r, i, hippo).is_err());
            city.energy = 30.;
            city.stock[m] = 6;
            let faith = city.faith;
            let before = city.contest.creature_visits;
            let result = city.aid(r, i, hippo).unwrap();
            assert!(result.contains("+4 Vertrauen"));
            assert_eq!(city.energy, if hippo { 12. } else { 30. });
            assert_eq!(city.stock[m], if hippo { 6 } else { 2 });
            assert_eq!(city.contest.creature_visits, before + u16::from(hippo));
            assert_eq!(
                city.contest.towns[i].protection,
                if hippo { 0. } else { 60. }
            );
            let mut restored: Settlement =
                serde_json::from_slice(&serde_json::to_vec(&city).unwrap()).unwrap();
            assert!(restored.aid(r, i, hippo).is_err());
            assert_eq!(restored.faith, faith + 4);
            assert_eq!(restored.projects.helped, 1);
            restored.village_start = Some(VillageStart::Acquainted(i));
            restored.projects_tick(r, 171.);
            let next = restored.projects.appeal.as_ref().unwrap().village;
            restored.projects_tick(r, 360.);
            assert_eq!(restored.projects.appeal.as_ref().unwrap().village, next);
            restored.defer_appeal(next).unwrap();
            assert!(restored.aid(r, next, hippo).is_err());
            assert_eq!(restored.projects.helped, 1);
        }
    }

    #[test]
    fn a_pilgrimage_survives_loading_and_rewards_a_personal_welcome() {
        let game = Game::start(Mode::Campaign(Biome::Fjord));
        let r = game.region();
        let mut city = game.city().clone();
        let (from, to) = (0, 1);
        city.contest.towns[from].side = Allegiance::Ours;
        city.contest.towns[from].trust = 100.;
        city.stock[2] = 3;
        assert!(city.procession(r, to).is_err());
        city.stock[2] = 10;
        city.procession(r, to).unwrap();
        assert_eq!(city.stock[2], 6);
        assert_eq!(city.contest.towns[from].trust, 80.);
        assert_eq!(city.contest.towns[from].pilgrim_rest, 55.);
        assert!(city.procession(r, to).is_err());
        city.projects_tick(r, 0.5);
        let saved = serde_json::to_vec(&city).unwrap();
        for welcome in [false, true] {
            let mut continued: Settlement = serde_json::from_slice(&saved).unwrap();
            if welcome {
                arrive(&mut continued, r, to, true);
            }
            for _ in 0..240 {
                continued.projects_tick(r, 0.5);
                if continued.projects.pilgrims.is_none() {
                    break;
                }
            }
            assert!(continued.projects.pilgrims.is_none());
            assert_eq!(continued.stock[2], 6);
            assert_eq!(
                continued.contest.towns[to].trust,
                if welcome { 55. } else { 30. }
            );
            continued.projects_tick(r, 0.5);
            assert_eq!(
                continued.contest.towns[to].trust,
                if welcome { 55. } else { 30. }
            );
        }
    }
}
