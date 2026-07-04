use std::fs;
use std::path::{Path, PathBuf};

use regex::Regex;
use serde_json::Value;

use crate::CliResult;

#[derive(Debug, Default, Clone)]
pub struct CensusData {
    pub world_dims: Option<[u64; 3]>,
    pub total_cells: Option<u64>,
    pub non_air: Option<u64>,
    pub non_air_pct: Option<f64>,
    pub max_solid_y: Option<u64>,
    pub chunk_submeshes: Option<u64>,
    pub seed: Option<u64>,
}

pub fn find_latest_run_log(root: &Path) -> CliResult<Option<PathBuf>> {
    let candidates = [
        root.join("dist/diag-stderr.log"),
        root.join("dist/civ-stderr.log"),
        root.join(".process-compose/logs/civ-standalone.err.log"),
    ];

    let mut existing = Vec::<(PathBuf, std::time::SystemTime)>::new();
    for path in candidates.iter().filter(|p| p.exists()) {
        let meta = fs::metadata(path)?;
        let modified = meta.modified()?;
        existing.push((path.to_path_buf(), modified));
    }

    existing.sort_by_key(|(_, m)| *m);
    Ok(existing.pop().map(|(path, _)| path))
}

pub fn read_census_from_log(path: &Path) -> CliResult<CensusData> {
    let content = fs::read_to_string(path)?;
    Ok(parse_census_text(&content))
}

fn strip_ansi(line: &str) -> String {
    let ansi = Regex::new(r"\x1b\[[0-9;]*m").expect("ansi regex");
    ansi.replace_all(line, "").to_string()
}

pub fn parse_census_text(text: &str) -> CensusData {
    let mut data = CensusData::default();
    let world = Regex::new(
        r"\[voxel\] world dims=\[(\d+),\s*(\d+),\s*(\d+)\] total_cells=(\d+) non_air=(\d+) \(([\d.]+)%\) max_solid_y=(\d+)",
    )
    .expect("world regex");
    let chunks =
        Regex::new(r"\[voxel\] spawned (\d+) chunk-submesh entities").expect("chunks regex");
    let seed = Regex::new(r"seed=(\d+)").expect("seed regex");

    for raw_line in text.lines() {
        let line = strip_ansi(raw_line);
        if let Some(cap) = world.captures(&line) {
            data.world_dims = Some([
                cap[1].parse().unwrap_or(0),
                cap[2].parse().unwrap_or(0),
                cap[3].parse().unwrap_or(0),
            ]);
            data.total_cells = Some(cap[4].parse().unwrap_or(0));
            data.non_air = Some(cap[5].parse().unwrap_or(0));
            data.non_air_pct = Some(cap[6].parse().unwrap_or(0.0));
            data.max_solid_y = Some(cap[7].parse().unwrap_or(0));
        }
        if let Some(cap) = chunks.captures(&line) {
            data.chunk_submeshes = Some(cap[1].parse().unwrap_or(0));
        }
        if let Some(cap) = seed.captures(&line) {
            data.seed = Some(cap[1].parse().unwrap_or(0));
        }
    }

    data
}

pub fn census_to_json(data: &CensusData) -> Value {
    serde_json::json!({
        "world_dims": data.world_dims,
        "total_cells": data.total_cells,
        "non_air": data.non_air,
        "non_air_pct": data.non_air_pct,
        "max_solid_y": data.max_solid_y,
        "chunk_submeshes": data.chunk_submeshes,
        "seed": data.seed,
    })
}

#[cfg(test)]
mod tests {
    use super::parse_census_text;

    #[test]
    fn parses_ansi_world_line() {
        let raw = "\u{1b}[31m[voxel]\u{1b}[0m world dims=[64, 48, 64] total_cells=196608 non_air=92327 (47.0%) max_solid_y=47";
        let data = parse_census_text(raw);
        assert_eq!(data.world_dims, Some([64, 48, 64]));
        assert_eq!(data.total_cells, Some(196608));
        assert_eq!(data.non_air, Some(92327));
        assert_eq!(data.non_air_pct, Some(47.0));
        assert_eq!(data.max_solid_y, Some(47));
    }

    #[test]
    fn parses_submesh_and_seed() {
        let raw = "[voxel] spawned 166 chunk-submesh entities\nseed=12345";
        let data = parse_census_text(raw);
        assert_eq!(data.chunk_submeshes, Some(166));
        assert_eq!(data.seed, Some(12345));
    }
}
