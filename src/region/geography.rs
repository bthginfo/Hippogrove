use std::sync::OnceLock;

use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use super::{AREA_COUNT, Biome, Region, Ridgeline, Tile};

#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
pub enum WorldLayout {
    #[default]
    Classic,
    Regional,
}

#[derive(Deserialize)]
struct TerrainPlan {
    river: Vec<[i16; 3]>,
    crossings: [i16; 2],
    ridges: Vec<Ridgeline>,
    starter_plots: Vec<Tile>,
    area_centers: Option<[Tile; AREA_COUNT]>,
    #[serde(default)]
    basins: Vec<[i16; 4]>,
    #[serde(default)]
    islets: Vec<[i16; 4]>,
}

#[derive(Clone, Copy, Debug)]
struct Basin {
    center: Vec2,
    radius: Vec2,
}

impl Basin {
    fn shore(self, point: Vec2) -> f32 {
        (((point - self.center) / self.radius).length() - 1.) * self.radius.min_element()
    }

    fn terrace_shore(self, point: Vec2) -> f32 {
        let edge = (point - self.center).abs() - self.radius + Vec2::splat(0.68);
        edge.max(Vec2::ZERO).length() + edge.max_element().min(0.) - 0.68
    }
}

#[derive(Clone, Debug)]
pub(super) struct Geography {
    first_column: i16,
    channel: Vec<(i16, i16)>,
    basins: Vec<Basin>,
    islets: Vec<Basin>,
}

impl Geography {
    pub(super) fn apply_to(region: &mut Region) -> Self {
        static PLANS: OnceLock<[[TerrainPlan; 3]; 5]> = OnceLock::new();
        let plans = PLANS.get_or_init(|| {
            serde_json::from_str(include_str!("../../assets/world/geography.json"))
                .expect("The embedded regional maps must be valid")
        });
        let terrain = &plans[region.kind as usize][region.chapter()];
        let layout = region.layout;
        region.crossings = terrain.crossings.map(|x| layout.tile(Tile(x, 0)).0);
        region.ridges = terrain
            .ridges
            .iter()
            .map(|&Ridgeline(start, end, width, height)| {
                Ridgeline(layout.tile(start), layout.tile(end), width, height)
            })
            .collect();
        region.starter_plots = terrain
            .starter_plots
            .iter()
            .map(|&tile| layout.plot(tile))
            .collect();
        if let Some(centers) = terrain.area_centers {
            for (area, center) in region.areas.iter_mut().zip(centers) {
                area.center = layout.tile(center);
            }
        }
        let anchors = protected_land(region);
        let first_column = region.bounds.0.0 - 10;
        let channel = (first_column..=region.bounds.1.0 + 10)
            .map(|column| Self::channel_column(region, column, &terrain.river, &anchors))
            .collect();
        let basin = |[x, z, width, depth]: [i16; 4]| {
            let x = if region.layout.mirror { -x } else { x };
            Basin {
                center: Tile(region.cross_x + x, region.river + z).local(),
                radius: Vec2::new(f32::from(width), f32::from(depth)),
            }
        };
        let basins = terrain
            .basins
            .iter()
            .copied()
            .map(basin)
            .filter(|shape| {
                let margin = Basin {
                    radius: shape.radius + Vec2::splat(2.),
                    ..*shape
                };
                anchors.iter().all(|tile| {
                    if region.kind == Biome::Terraces {
                        margin.terrace_shore(tile.local()) > 0.
                    } else {
                        margin.shore(tile.local()) > 0.
                    }
                }) && std::iter::once(region.cross_x)
                    .chain(region.crossings)
                    .all(|x| (f32::from(x) - shape.center.x).abs() > shape.radius.x + 1.)
            })
            .collect();
        Self {
            first_column,
            channel,
            basins,
            islets: terrain.islets.iter().copied().map(basin).collect(),
        }
    }

    fn channel_column(
        region: &Region,
        column: i16,
        points: &[[i16; 3]],
        anchors: &[Tile],
    ) -> (i16, i16) {
        let bank = region.classic_river_at(column);
        let span = region.classic_river_span(column);
        let delta = region.layout.original_x(column) - region.layout.original_x(region.cross_x);
        if delta.abs() <= 2 {
            return (bank, span);
        }
        let [bend, spread] = river_profile(points, delta);
        let mut north = bank + bend;
        let mut south = north + span + spread;
        for tile in anchors.iter().filter(|tile| tile.0.abs_diff(column) <= 1) {
            if tile.1 < bank {
                north = north.max(tile.1 + 1);
            } else if tile.1 >= bank + span {
                south = south.min(tile.1);
            }
        }
        if south - north >= 2 {
            (north, south - north)
        } else {
            (bank, span)
        }
    }

    pub(super) fn channel(&self, column: i16) -> (i16, i16) {
        let index = (column - self.first_column).clamp(0, self.channel.len() as i16 - 1);
        self.channel[index as usize]
    }

    pub(super) fn basin_shore(&self, kind: Biome, point: Vec2) -> f32 {
        self.basins
            .iter()
            .map(|basin| {
                if kind == Biome::Terraces {
                    basin.terrace_shore(point)
                } else {
                    basin.shore(point)
                }
            })
            .fold(f32::INFINITY, f32::min)
    }

    pub(super) fn islet_shore(&self, point: Vec2) -> f32 {
        self.islets
            .iter()
            .map(|islet| -islet.shore(point))
            .fold(f32::NEG_INFINITY, f32::max)
    }
}

fn river_profile(points: &[[i16; 3]], position: i16) -> [i16; 2] {
    let end = points.partition_point(|[x, _, _]| *x <= position);
    let [left, bend, width] = points[end.saturating_sub(1)];
    let [right, next_bend, next_width] = points[end.min(points.len() - 1)];
    if left == right {
        return [bend, width];
    }
    let progress = crate::terrain::smooth(f32::from(position - left) / f32::from(right - left));
    [
        f32::from(bend).lerp(f32::from(next_bend), progress).round() as i16,
        f32::from(width)
            .lerp(f32::from(next_width), progress)
            .round() as i16,
    ]
}

fn protected_land(region: &Region) -> Vec<Tile> {
    let towns = [
        region.start,
        region.depot,
        region.frontier,
        region.rival_shrine,
    ];
    towns
        .into_iter()
        .flat_map(|tile| std::iter::once(tile).chain(tile.neighbors()))
        .chain(region.sources.iter().map(|source| source.tile))
        .chain(region.guards.iter().copied())
        .chain(region.starter_plots.iter().flat_map(|tile| {
            [
                *tile,
                tile.offset(1, 0),
                tile.offset(0, 1),
                tile.offset(1, 1),
            ]
        }))
        .chain(
            region
                .areas
                .iter()
                .flat_map(|area| std::iter::once(area.center).chain(area.center.neighbors())),
        )
        .chain(region.rival_buildings())
        .chain(
            region
                .districts
                .iter()
                .enumerate()
                .flat_map(|(index, district)| {
                    [district.center, district.center.offset(0, 2)]
                        .into_iter()
                        .chain(region.district_houses(index))
                }),
        )
        .collect()
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use super::*;

    #[test]
    fn regional_maps_have_distinct_profiles_and_clear_bridge_approaches() {
        let mut profiles = HashSet::new();
        let mut crossings = HashSet::new();
        for biome in Biome::ALL {
            for chapter in 0..3 {
                let region = Region::themed(biome, chapter, WorldLayout::Regional);
                let direction = if region.layout.mirror { -1 } else { 1 };
                let channel = (-46..=46)
                    .map(|x| region.cross_x + x * direction)
                    .map(|x| (region.river_at(x) - region.river, region.river_span(x)))
                    .collect::<Vec<_>>();
                assert!(profiles.insert(channel), "{biome:?} {chapter}");
                assert_eq!(region.river_at(region.cross_x), region.river);
                assert_eq!(region.river_span(region.cross_x), region.water_width);
                assert!(region.bridge_cell(region.bridge()));
                assert!(
                    crossings.insert(region.crossings.map(|x| (x - region.cross_x) * direction))
                );
                let walk = region.walk_grid(true);
                for x in std::iter::once(region.cross_x).chain(region.crossings) {
                    let bank = region.river_at(x);
                    let opposite = bank + region.river_span(x);
                    for z in [bank - 1, opposite] {
                        let tile = Tile(x, z);
                        assert!(!region.water(tile), "{biome:?} {chapter} {tile:?}");
                        assert!(
                            region.bounds.index(tile).is_some_and(|index| walk[index]),
                            "{biome:?} {chapter} blocked bridge approach at {tile:?}"
                        );
                    }
                }
                assert!(
                    region
                        .bounds
                        .tiles()
                        .any(|tile| region.basin_shore(tile.local()) < 0.)
                );
                for tile in [region.start, region.depot, region.rival_shrine]
                    .into_iter()
                    .chain(region.districts.iter().map(|district| district.center))
                    .chain(region.areas.iter().map(|area| area.center))
                {
                    assert!(!region.water(tile), "{biome:?} {chapter} {tile:?}");
                }
            }
        }
    }
}
