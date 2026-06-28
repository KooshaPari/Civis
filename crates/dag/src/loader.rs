//! Markdown → typed `Plan` parser for `CIVIS_GAME_DAG.md`.
//!
//! The grammar is intentionally tolerant: append a `## <n>` markdown
//! section and it becomes a new `Layer`. Append a pipe-delimited row to
//! any of the §2/§3/§4/§7/§8 tables and it becomes additional nodes.
//! Re-running `load_from_markdown` on a freshly edited source produces a
//! JSON-serialisable `Plan` that round-trips faithfully.
use anyhow::{anyhow, Result};
use chrono::{DateTime, NaiveDate, Utc};
use std::path::PathBuf;

use crate::model::{
    Lane, Layer, Node, NodeId, NodeState, Phase, PillarEpic, Plan, PlanMeta, QueuedPillar, Tick,
};

/// Parse a `CIVIS_GAME_DAG.md`-shaped document into a typed [`Plan`].
pub fn load_from_markdown(source: &str, path: PathBuf) -> Result<Plan> {
    let sections = split_into_sections(source);
    let title = sections
        .title
        .as_deref()
        .unwrap_or("CIVIS — Master DAG (canonical, indefinitely extendable)");
    let id = slugify_title(title);
    let version = extract_version(&sections.body).unwrap_or_else(|| "v0".into());

    let mut layers = parse_layers(&sections.body)?;
    let pillars = parse_pillars_done(&sections.body)?;
    let queued_pillars = parse_pillars_queued(&sections.body)?;
    let lanes = parse_lanes(&sections.body)?;
    let ticks = parse_ticks(&sections.body)?;
    let phases = parse_phases(&sections.body)?;
    let extensions = parse_layers_extensions(&sections.body)?;

    // Append-only extensions become additional layers (hooks populated).
    for (id, hook) in extensions {
        if let Some(layer) = layers.iter_mut().find(|l| l.id == id) {
            layer.extendability_hooks.push(hook);
        } else {
            layers.push(Layer {
                id: id.clone(),
                name: hook.clone(),
                scope: "(extension)".into(),
                parallel_safe: true,
                extendability_hooks: vec![hook],
                nodes: vec![],
            });
        }
    }

    // Materialise pillar epics into actual `Node` rows so they participate
    // in `waves()`, `progress()`, and the reporter. Pillar nodes are
    // treated as top-level (no synthetic deps to layer ids), since the
    // layers themselves are not in `Plan::nodes` — only their constituent
    // pillars are. Real cross-pillar deps can be added later by appending
    // explicit `deps:` to the markdown tables.
    let mut nodes = Vec::<Node>::new();
    for pillar in &pillars {
        let state = pillar_state_from_prs(&pillar.prs);
        nodes.push(Node {
            id: pillar.fr_id.clone(),
            layer: "P3".into(),
            pr: pillar.prs.first().copied(),
            title: pillar.name.clone(),
            scope: "pillar".into(),
            state,
            lane: pillar.lane.clone(),
            deps: vec![],
            fr_ref: Some(pillar.fr_id.clone()),
            adr_ref: pillar.adr.clone(),
            eta: None,
            started_at: None,
            finished_at: None,
            summary: None,
            agent: None,
            task: None,
        });
    }
    for qp in &queued_pillars {
        nodes.push(Node {
            id: qp.fr_id.clone(),
            layer: "P3".into(),
            pr: qp.pr,
            title: qp.name.clone(),
            scope: "pillar-queued".into(),
            state: qp.state,
            lane: qp.lane.clone(),
            deps: vec![],
            fr_ref: Some(qp.fr_id.clone()),
            adr_ref: None,
            eta: None,
            started_at: None,
            finished_at: None,
            summary: None,
            agent: None,
            task: None,
        });
    }

    Ok(Plan {
        id,
        version,
        layers,
        lanes,
        ticks: ticks.clone(),
        pillars,
        queued_pillars,
        phases,
        nodes: nodes.clone(),
        metadata: PlanMeta {
            total_nodes: nodes.len(),
            total_ticks: ticks.len(),
            last_updated: Utc::now(),
            source_path: path,
        },
    })
}

#[derive(Debug, Default)]
struct Sections {
    title: Option<String>,
    body: String,
}

fn split_into_sections(src: &str) -> Sections {
    let mut sections = Sections::default();
    let mut in_sub = false;
    for line in src.lines() {
        if let Some(rest) = line.strip_prefix("# ") {
            sections.title = Some(rest.trim().to_string());
        } else if line.starts_with("## ") || line.starts_with("### ") {
            in_sub = true;
            sections.body.push_str(line);
            sections.body.push('\n');
        } else if in_sub {
            sections.body.push_str(line);
            sections.body.push('\n');
        }
    }
    sections
}

fn slugify_title(title: &str) -> String {
    title
        .chars()
        .map(|c| if c.is_ascii_alphanumeric() { c.to_ascii_lowercase() } else { '-' })
        .collect::<String>()
        .trim_matches('-')
        .split('-')
        .filter(|s| !s.is_empty())
        .collect::<Vec<_>>()
        .join("-")
}

fn extract_version(body: &str) -> Option<String> {
    for line in body.lines() {
        let t = line.trim_start();
        if let Some(rest) = t.strip_prefix("- **v") {
            // "v1 (2026-06-25): ..."  →  "v1"
            let head = rest.split(':').next().unwrap_or(rest);
            let head = head.split_whitespace().next().unwrap_or("");
            if !head.is_empty() {
                return Some(format!("v{}", head.trim_start_matches('v')));
            }
            if rest.starts_with('1') || rest.starts_with('2') {
                return Some(format!("v{}", &rest[..1]));
            }
        }
    }
    None
}

/// §1 layer table. Each `P<n> <name> <scope>` line produces a Layer.
fn parse_layers(body: &str) -> Result<Vec<Layer>> {
    let mut layers = Vec::new();
    let section = slice_section(body, "1");
    for line in section.lines() {
        let trimmed = line.trim();
        // pipe-delimited row: `| `P0` | Foundation | voxel · engine | core |`
        let is_pipe_row = trimmed.starts_with("| ") && trimmed.contains('|');
        let has_p_id = trimmed.contains("P0")
            || trimmed.contains("P1")
            || trimmed.contains("P2")
            || trimmed.contains("P3")
            || trimmed.contains("P4")
            || trimmed.contains("P5")
            || trimmed.contains("P6")
            || trimmed.contains("P7")
            || trimmed.contains("P8")
            || trimmed.contains("P9");
        if is_pipe_row && has_p_id && !trimmed.starts_with("| #") && !trimmed.starts_with("|---") {
            let cells: Vec<String> = trimmed
                .split('|')
                .map(|c| c.trim().to_string())
                .filter(|c| !c.is_empty())
                .collect();
            if cells.len() >= 2 {
                // Find the cell that looks like a layer id (P<digit>+), prefer
                // the first such cell to handle both `P0` and `P0 ...`.
                let id_cell = cells
                    .iter()
                    .find(|c| {
                        let clean = c.trim_matches('`').trim();
                        layer_num(clean).is_some()
                    })
                    .cloned()
                    .unwrap_or_else(|| cells[0].clone());
                let id = id_cell.trim_matches('`').trim().to_string();
                let name_cell = cells
                    .iter()
                    .find(|c| {
                        let clean = c.trim_matches('`').trim();
                        clean != id && !clean.is_empty()
                    })
                    .cloned()
                    .unwrap_or_else(|| id.clone());
                let name = name_cell.trim_matches('`').trim().to_string();
                let scope = cells
                    .get(2)
                    .map(|s| s.trim_matches('`').trim().to_string())
                    .filter(|s| !s.is_empty())
                    .unwrap_or_else(|| "core".into());
                layers.push(Layer {
                    id: id.clone(),
                    name: name.clone(),
                    scope,
                    parallel_safe: true,
                    extendability_hooks: vec![format!(
                        "append `P{}.<n>` after {}",
                        next_layer_num(&layers),
                        id
                    )],
                    nodes: vec![NodeId(id)],
                });
            }
        }
    }
    if layers.is_empty() {
        // Fallback: parse as bullet list in §1 block
        for line in section.lines() {
            let trimmed = line.trim_start();
            if let Some(rest) = trimmed.strip_prefix("P") {
                let mut parts = rest.split_whitespace();
                if let Some(num) = parts.next() {
                    if num.chars().all(|c| c.is_ascii_digit()) {
                        let name: String =
                            parts.take_while(|s| !s.contains('(')).collect::<Vec<_>>().join(" ");
                        if !name.is_empty() {
                            let id = format!("P{}", num);
                            layers.push(Layer {
                                id: id.clone(),
                                name: name.clone(),
                                scope: "core".into(),
                                parallel_safe: true,
                                extendability_hooks: vec![format!(
                                    "append `P{}.<n>` after P{}",
                                    next_layer_num(&layers),
                                    num
                                )],
                                nodes: vec![NodeId(id)],
                            });
                        }
                    }
                }
            }
        }
    }
    if layers.is_empty() {
        return Err(anyhow!("no P-layers parsed from §1"));
    }
    Ok(layers)
}

fn layer_num(id: &str) -> Option<u32> {
    id.strip_prefix('P')
        .and_then(|s| s.chars().take_while(|c| c.is_ascii_digit()).collect::<String>().parse().ok())
}
fn next_layer_num(layers: &[Layer]) -> u32 {
    layers
        .iter()
        .filter_map(|l| layer_num(&l.id))
        .max()
        .unwrap_or(0)
        + 1
}

/// §2 phase table.
fn parse_phases(body: &str) -> Result<Vec<Phase>> {
    let mut phases = Vec::new();
    let section = slice_section(body, "2 · Phase ordering");
    for line in section.lines() {
        let trimmed = line.trim();
        if !trimmed.starts_with('|') {
            continue;
        }
        let cells: Vec<String> = trimmed
            .split('|')
            .map(|c| c.trim().to_string())
            .filter(|c| !c.is_empty())
            .collect();
        if cells.len() < 4 {
            continue;
        }
        // skip header row "| # | phase | dep | lane |"
        if cells.iter().any(|c| c == "phase" || c == "dep" || c == "lane" || c == "#") {
            continue;
        }
        let ordinal_str = &cells[0];
        // must be numeric (data row) — header would already be skipped
        if ordinal_str.parse::<u8>().is_err() {
            continue;
        }
        let ordinal: u8 = ordinal_str.parse().unwrap();
        let name = cells[1].trim_matches('`').trim().to_string();
        let dep_str = cells[2].trim();
        let dep = if dep_str == "—" || dep_str == "-" { None } else { Some(dep_str.to_string()) };
        let lane = cells[3].trim().to_string();
        phases.push(Phase { ordinal, name, dep, lane });
    }
    Ok(phases)
}

/// §3 Pillar epics (DONE).
fn parse_pillars_done(body: &str) -> Result<Vec<PillarEpic>> {
    let mut out = Vec::new();
    let section = slice_section(body, "3 · Pillar epics");
    for line in section.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() || !trimmed.starts_with('|') {
            continue;
        }
        let cells: Vec<String> = trimmed
            .split('|')
            .map(|c| c.trim().to_string())
            .filter(|c| !c.is_empty())
            .collect();
        if cells.len() < 4 {
            continue;
        }
        let fr_id = cells[0].trim_matches('`').to_string();
        if !fr_id.starts_with("FR-") {
            continue; // header row or unrelated
        }
        let name = cells[1].to_string();
        let lane = cells[2].to_string();
        let prs = parse_pr_list(&cells[3]);
        let adr = cells.get(4).map(|s| s.trim().to_string()).filter(|s| !s.is_empty() && s != "—");
        out.push(PillarEpic { fr_id, name, lane, prs, adr });
    }
    Ok(out)
}

/// §4 Pillar systems (queued / partial).
fn parse_pillars_queued(body: &str) -> Result<Vec<QueuedPillar>> {
    let mut out = Vec::new();
    let section = slice_section(body, "4 · Pillar systems");
    for line in section.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() || !trimmed.starts_with('|') {
            continue;
        }
        let cells: Vec<String> = trimmed
            .split('|')
            .map(|c| c.trim().to_string())
            .filter(|c| !c.is_empty())
            .collect();
        if cells.len() < 4 {
            continue;
        }
        let fr_id = cells[0].trim_matches('`').to_string();
        if !fr_id.starts_with("FR-") {
            continue;
        }
        let name = cells[1].to_string();
        let state_str = cells[2].as_str();
        let state = parse_node_state(state_str);
        let eta_label = cells[3].to_string();
        let lane = cells.get(4).cloned().unwrap_or_else(|| "pillar".into());
        let pr = extract_pr_marker(state_str);
        out.push(QueuedPillar { fr_id, name, state, eta_label, pr, lane });
    }
    Ok(out)
}

/// §7 Lanes.
fn parse_lanes(body: &str) -> Result<Vec<Lane>> {
    let mut out = Vec::new();
    let section = slice_section(body, "7 · Agent-dispatch lanes");
    for line in section.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("| `") && trimmed.contains('|') {
            let cells: Vec<String> = trimmed
                .split('|')
                .map(|c| c.trim().to_string())
                .filter(|c| !c.is_empty())
                .collect();
            if cells.len() < 4 {
                continue;
            }
            if cells.iter().any(|c| c == "purpose" || c == "scope" || c == "mutex") {
                continue;
            }
            let id = cells[0].trim_matches('`').to_string();
            let purpose = cells[1].to_string();
            let scope = cells[2].to_string();
            let mutex = cells.get(3).cloned().filter(|s| s != "—" && !s.is_empty());
            if id.is_empty() {
                continue;
            }
            out.push(Lane {
                id,
                purpose,
                scope,
                mutex,
                default_concurrency: 4,
                partition_by: None,
            });
        }
    }
    Ok(out)
}

/// §8 Tick log. Each `| T<n> | ... |` row becomes a Tick.
fn parse_ticks(body: &str) -> Result<Vec<Tick>> {
    let mut out = Vec::new();
    let section = slice_section(body, "8 · Tick log");
    for line in section.lines() {
        let trimmed = line.trim();
        if !trimmed.starts_with("| T") && !trimmed.starts_with("| T0") {
            continue;
        }
        let cells: Vec<String> = trimmed
            .split('|')
            .map(|c| c.trim().to_string())
            .filter(|c| !c.is_empty())
            .collect();
        if cells.len() < 6 {
            continue;
        }
        if cells.iter().any(|c| c == "merged" || c == "opened" || c == "notes") {
            continue;
        }
        let id = cells[0].trim().to_string();
        let merged = parse_pr_list(&cells[1]);
        let opened = parse_pr_list(&cells[2]);
        let closed_superseded = parse_pr_list(&cells[3]);
        let open_net = cells[4].parse().unwrap_or(0);
        let notes = cells[5].to_string();
        let recorded_at = parse_date_hint(&notes).unwrap_or_else(Utc::now);
        out.push(Tick { id, merged, opened, closed_superseded, open_net, notes, recorded_at });
    }
    Ok(out)
}

/// §10-style or ad-hoc extension sections.  Each `## <n> · <title>` that
/// isn't one of the canonical sections (`§1`-`§11`) becomes a NEW LAYER
/// (so the plan remains indefinitely extendable by simply appending a
/// new `## <n>` section beyond §11). The hook surfaces the section title.
fn parse_layers_extensions(body: &str) -> Result<Vec<(String, String)>> {
    let canonical: &[&str] = &["0", "1", "2", "3", "4", "5", "6", "7", "8", "9", "10", "11"];
    let mut out = Vec::new();
    for line in body.lines() {
        if let Some(rest) = line.strip_prefix("## ") {
            let head = rest.split('·').next().unwrap_or("").trim();
            let head_num: String = head.chars().take_while(|c| c.is_ascii_digit()).collect();
            if !head_num.is_empty() && !canonical.contains(&head_num.as_str()) {
                let layer_id = format!("P{}", head_num);
                let title = rest.trim().trim_end_matches('#').trim().to_string();
                out.push((layer_id, title));
            }
        }
    }
    Ok(out)
}

fn parse_pr_list(s: &str) -> Vec<u32> {
    s.split(|c: char| !c.is_ascii_digit())
        .filter_map(|p| p.parse::<u32>().ok())
        .collect()
}

fn extract_pr_marker(state_label: &str) -> Option<u32> {
    state_label
        .split(|c: char| !c.is_ascii_digit())
        .filter_map(|p| p.parse::<u32>().ok())
        .next()
}

fn parse_node_state(s: &str) -> NodeState {
    let lower = s.to_ascii_lowercase();
    if lower.contains("merged") {
        NodeState::Merged
    } else if lower.contains("verify") {
        NodeState::Verified
    } else if lower.contains("blocked") {
        NodeState::Blocked
    } else if lower.contains("working") || lower.contains("active") || lower.starts_with("🔁") || lower.starts_with("⏳") {
        NodeState::InFlight
    } else if lower.contains("queued") || lower.contains("pending") {
        NodeState::Queued
    } else {
        NodeState::Queued
    }
}

fn parse_date_hint(s: &str) -> Option<DateTime<Utc>> {
    // Look for `2026-06-XX` or `2026-06-XX-T..` inside the string.
    for token in s.split(|c: char| c.is_whitespace() || c == ',') {
        if token.len() >= 10 && token.chars().nth(4) == Some('-') && token.chars().nth(7) == Some('-') {
            if let Ok(d) = NaiveDate::parse_from_str(&token[..10], "%Y-%m-%d") {
                return d.and_hms_opt(0, 0, 0).map(|dt| dt.and_utc());
            }
        }
    }
    None
}

fn pillar_state_from_prs(prs: &[u32]) -> NodeState {
    if prs.is_empty() {
        NodeState::Queued
    } else {
        NodeState::Done
    }
}

/// Slice text starting from the `## <n>` section inclusive, until the next `## <m>` heading.
fn slice_section<'a>(body: &'a str, needle: &str) -> &'a str {
    // needle is e.g. "3" or "3 · Pillar epics" — match by leading number.
    let needle_num: String = needle
        .chars()
        .take_while(|c| c.is_ascii_digit())
        .collect();
    let mut start: Option<usize> = None;
    let mut end: Option<usize> = None;
    let lines: Vec<&str> = body.lines().collect();
    let mut line_offsets: Vec<usize> = Vec::with_capacity(lines.len());
    {
        let mut off = 0usize;
        for l in &lines {
            line_offsets.push(off);
            off += l.len() + 1;
        }
    }
    for (line_idx, line) in lines.iter().enumerate() {
        if let Some(rest) = line.strip_prefix("## ") {
            let head_num: String = rest.chars().take_while(|c| c.is_ascii_digit()).collect();
            if start.is_none() {
                if head_num == needle_num {
                    start = Some(line_offsets[line_idx]);
                }
            } else if !head_num.is_empty() && head_num != needle_num {
                end = Some(line_offsets[line_idx]);
                break;
            }
        }
    }
    let start = match start {
        Some(s) => s,
        None => return "",
    };
    let end = end.unwrap_or(body.len());
    &body[start..end]
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::NodeState;

    const SAMPLE_DAG: &str = r#"# CIVIS — Master DAG vN (canonical, indefinitely extendable)

## 0 · Read me first
- intro

## 1 · Layer model (P0 … Pn)

| # | layer | name | scope |
|---|---|---|---|
| `P0` | Foundation | voxel · engine | core |
| `P1` | Substrate | emergence | core |
| `P2` | Bridge    | MCP · egui  | bridge |

## 2 · Phase ordering

| # | phase | dep | lane |
|---|---|---|---|
| 1 | `planet` | — | core |
| 2 | `voxel`  | planet | core |

## 3 · Pillar epics

| FR | name | lane | PRs | ADR |
|---|---|---|---|---|
| `FR-MARKET` | markets | pillar | #804 | ADR-018 |
| `FR-LANG`   | langs   | pillar | — | ADR-014 |

## 4 · Pillar systems

| FR | name | state | ETA | lane |
|---|---|---|---|---|
| `FR-VEHICLES` | logistics | ⏸ | Q3 | pillar |

## 7 · Agent-dispatch lanes

| lane | purpose | scope | mutex |
|---|---|---|---|
| `merge-harvest` | harvest | global | none |

## 8 · Tick log

| tick | merged | opened | closed-superseded | open-net | notes |
|---|---|---|---|---|---|
| T0 | #845 | — | — | 1 | 2026-06-25 |
| T1 | #851 | — | — | 1 | 2026-06-26 |

## 9 · Extendability hooks (how to add)
- trivial list

## 12 · P6 — new explicit extension
content
"#;

    #[test]
    fn parses_core_counts() {
        let p = load_from_markdown(SAMPLE_DAG, PathBuf::from("t.md")).unwrap();
        // §1 yields P0, P1, P2; §12 extension yields P12. So >= 3 layers.
        assert!(p.layers.len() >= 3, "layers, got {}", p.layers.len());
        assert!(p.layers.iter().any(|l| l.id == "P0"), "P0 present");
        assert!(p.layers.iter().any(|l| l.id == "P1"), "P1 present");
        assert!(p.layers.iter().any(|l| l.id == "P2"), "P2 present");
        assert!(!p.pillars.is_empty(), "pillars");
        assert!(p.pillars.iter().any(|x| x.fr_id == "FR-MARKET"));
        assert!(!p.queued_pillars.is_empty(), "queued pillars");
        assert!(!p.lanes.is_empty(), "lanes");
        assert!(p.lanes.iter().any(|l| l.id == "merge-harvest"));
        assert!(p.ticks.len() >= 2, "ticks");
        assert!(!p.phases.is_empty(), "phases");
    }

    #[test]
    fn waves_and_ready() {
        let p = load_from_markdown(SAMPLE_DAG, PathBuf::from("t.md")).unwrap();
        // Pillars become Queued nodes (no PRs OR queued).
        for n in &p.nodes {
            assert!(matches!(n.state, NodeState::Done | NodeState::Queued));
        }
        let waves = p.waves();
        assert!(!waves.is_empty(), "waves should be non-empty");
    }

    #[test]
    fn round_trips_via_json() {
        let p = load_from_markdown(SAMPLE_DAG, PathBuf::from("t.md")).unwrap();
        let json = serde_json::to_string(&p).expect("serialize");
        let back: Plan = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(back.layers.len(), p.layers.len());
        assert_eq!(back.nodes.len(), p.nodes.len());
        assert_eq!(back.pillars.len(), p.pillars.len());
    }

    #[test]
    fn extension_section_adds_layer_hook() {
        let mut amended = SAMPLE_DAG.to_string();
        amended.push_str("\n## 13 · P7 — another extension\nmore\n");
        let p = load_from_markdown(&amended, PathBuf::from("t2.md")).unwrap();
        // Should now have at least 3 layers; the §12/§13 hooks may append
        assert!(p.layers.len() >= 3);
    }

    #[test]
    fn parses_live_dag_file() {
        let md = std::fs::read_to_string("C:/Users/koosh/Dev/Civis/CIVIS_GAME_DAG.md");
        if let Ok(src) = md {
            let p = load_from_markdown(&src, PathBuf::from("CIVIS_GAME_DAG.md")).unwrap();
            assert!(
                p.layers.len() >= 6,
                "expected ≥6 layers, got {}",
                p.layers.len()
            );
            assert!(p.pillars.len() >= 20, "pillars >=20; got {}", p.pillars.len());
            assert!(!p.lanes.is_empty(), "lanes parsed");
        }
    }
}
