use super::*;

const STARTING_BUILDINGS: [Building; 3] = [Building::House, Building::Store, Building::Field];

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub enum VillageStart {
    Building,
    Visiting(usize),
    Acquainted(usize),
    Open,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum VillageTask {
    Build(Building),
    Collect(Material, u16),
    Deposit,
    Water(Tile),
    Grow(Tile),
    Harvest(Tile),
    Explore,
    Visiting(usize),
    Acquainted(usize),
}

impl Settlement {
    pub(super) fn has_village_start(region: &Region) -> bool {
        region.has_rival_creature() && region.chapter() == 0 && region.quests.is_some()
    }

    pub fn settling_in(&self) -> bool {
        matches!(
            self.village_start,
            Some(VillageStart::Building | VillageStart::Visiting(_))
        )
    }

    pub fn rival_waiting(&self) -> bool {
        self.village_start
            .is_some_and(|stage| stage != VillageStart::Open)
    }

    pub(super) fn migrate_village_start(&mut self, region: &Region) {
        if Self::has_village_start(region)
            && self.village_start.is_none()
            && self.quest.index() <= Quest::Fields.index()
            && self.outcome.is_none()
            && !self.contest.pact
            && self.projects.helped == 0
            && self
                .walkers
                .iter()
                .all(|w| !matches!(w.job, Some(Job::Visit(_))))
            && self.contest.towns.iter().all(|town| town.trust <= 0.)
        {
            if self.waiting_for_first_delivery() {
                self.opening = None;
            }
            self.village_start = Some(VillageStart::Building);
            self.raid_clock = 0.;
            self.contest.let_village_settle();
        }
    }

    pub fn village_task(&self, region: &Region) -> Option<VillageTask> {
        match self.village_start? {
            VillageStart::Visiting(i) => return Some(VillageTask::Visiting(i)),
            VillageStart::Acquainted(i) => return Some(VillageTask::Acquainted(i)),
            VillageStart::Open => return None,
            VillageStart::Building => {}
        }
        if self.held.is_some() {
            return Some(VillageTask::Deposit);
        }
        if let Some(kind) = STARTING_BUILDINGS
            .into_iter()
            .find(|&kind| self.count(kind) == 0)
        {
            if let Some(material) = self.missing_for(kind) {
                let index = material.index();
                return Some(VillageTask::Collect(
                    material,
                    kind.price()[index] - self.stock[index],
                ));
            }
            return Some(VillageTask::Build(kind));
        }
        if self.harvests > 0 && !region.villages.is_empty() {
            return Some(VillageTask::Explore);
        }
        self.buildings
            .iter()
            .filter(|site| site.kind == Building::Field)
            .map(|site| match site.crop {
                Crop::Ripe => (0, VillageTask::Harvest(site.tile)),
                Crop::Growing(_) => (1, VillageTask::Grow(site.tile)),
                Crop::Dry => (2, VillageTask::Water(site.tile)),
            })
            .min_by_key(|(priority, _)| *priority)
            .map(|(_, task)| task)
    }

    pub fn village_milestones(&self) -> usize {
        STARTING_BUILDINGS
            .into_iter()
            .filter(|&kind| self.count(kind) > 0)
            .count()
            + usize::from(self.harvests > 0)
    }

    pub fn reachable_neighbor(&self, region: &Region) -> Option<usize> {
        let origin = self.walkers[0].tile();
        let mut villages: Vec<_> = region.villages.iter().enumerate().collect();
        villages.sort_by_key(|(i, village)| {
            (
                self.contest.towns[*i].side != Allegiance::Neutral,
                origin.distance(village.center),
            )
        });
        villages
            .into_iter()
            .find(|(_, village)| {
                self.travel_route(region, origin, village.center, true, true)
                    .is_some()
            })
            .map(|(i, _)| i)
    }

    pub(super) fn meet_neighbor(&mut self, region: &Region, index: usize) -> GameResult {
        let allowed = self.village_task(region) == Some(VillageTask::Explore)
            || (self.village_start == Some(VillageStart::Visiting(index))
                && self.visitor(index, true).is_none());
        if !allowed {
            return Err(
                "Baue zuerst dein Dorf auf. Der Besuch beginnt nach deiner ersten Ernte.".into(),
            );
        }
        let message = self.dispatch_visitor(region, index, true)?;
        self.village_start = Some(VillageStart::Visiting(index));
        Ok(message)
    }

    pub(super) fn begin_rivalry(&mut self, region: &Region) -> GameResult {
        if !matches!(self.village_start, Some(VillageStart::Acquainted(_))) {
            return Err("Besuche zuerst ein Nachbardorf mit deinem Nilpferd.".into());
        }
        self.village_start = Some(VillageStart::Open);
        self.contest.let_village_settle();
        self.quest = Quest::Fields;
        self.faith = self.faith.max(region.faith);
        Ok("Die Rivalität beginnt. Der erste Gesandte lässt sich Zeit.".into())
    }

    pub(super) fn check_first_visit(&mut self, region: &Region) {
        let Some(VillageStart::Visiting(i)) = self.village_start else {
            return;
        };
        if !self
            .visitor(i, true)
            .is_some_and(|w| self.visiting(w, region, i))
        {
            return;
        }
        self.village_start = Some(VillageStart::Acquainted(i));
        self.announce_village(format!(
            "{} hat dein Nilpferd begrüßt.",
            region.villages[i].name
        ));
    }

    pub(super) fn valid_village_start(&self, region: &Region) -> bool {
        match self.village_start {
            None => true,
            Some(VillageStart::Building | VillageStart::Open) => Self::has_village_start(region),
            Some(VillageStart::Visiting(i) | VillageStart::Acquainted(i)) => {
                Self::has_village_start(region) && i < region.villages.len()
            }
        }
    }
}
