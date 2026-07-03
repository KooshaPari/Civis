//! Tech diffusion via trade routes (FR-CIV-TECH).
//!
//! When two clusters share a trade route, technologies known to one cluster can
//! probabilistically spread to the other at a configurable rate per tick.

use std::collections::HashMap;

use civ_agents::ClusterId;
use rand::Rng;

use super::tech_tree::TechId;

/// Diffuse technologies across clusters connected by trade routes.
///
/// For each directed trade link `(a, b)`, every tech known to `a` but not
/// yet known to `b` has a `diffusion_rate` chance of spreading to `b` that
/// tick. The same logic runs in the reverse direction.
///
/// # Parameters
/// - `trade_routes`: Slice of `(ClusterId, ClusterId)` pairs representing
///   active trade connections. Directionality is symmetric — each pair is
///   treated bidirectionally.
/// - `cluster_techs`: Mutable map from cluster to its known tech ids.
/// - `diffusion_rate`: Probability in `[0.0, 1.0]` that a given tech spreads
///   per tick per link.
/// - `rng`: Caller-supplied RNG so callers control determinism.
pub fn diffuse_tech(
    trade_routes: &[(ClusterId, ClusterId)],
    cluster_techs: &mut HashMap<ClusterId, Vec<TechId>>,
    diffusion_rate: f32,
    rng: &mut impl Rng,
) {
    // Collect all (source, target) ordered pairs for bidirectional links.
    let pairs: Vec<(ClusterId, ClusterId)> = trade_routes
        .iter()
        .flat_map(|&(a, b)| [(a, b), (b, a)])
        .collect();

    let mut to_add: Vec<(ClusterId, TechId)> = Vec::new();

    for (src, dst) in &pairs {
        let src_techs = cluster_techs.get(src).cloned().unwrap_or_default();
        let dst_techs = cluster_techs.get(dst).cloned().unwrap_or_default();

        for tech in &src_techs {
            if dst_techs.contains(tech) {
                continue;
            }
            let roll: f32 = rng.gen();
            if roll < diffusion_rate {
                to_add.push((*dst, *tech));
            }
        }
    }

    for (cluster, tech) in to_add {
        let techs = cluster_techs.entry(cluster).or_default();
        if !techs.contains(&tech) {
            techs.push(tech);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::SeedableRng;
    use rand_chacha::ChaCha8Rng;

    #[test]
    fn diffusion_spreads_tech_between_connected_clusters() {
        let a = ClusterId(1);
        let b = ClusterId(2);
        let tech = TechId(0);

        let mut cluster_techs: HashMap<ClusterId, Vec<TechId>> = HashMap::new();
        cluster_techs.insert(a, vec![tech]);
        cluster_techs.insert(b, vec![]);

        let routes = vec![(a, b)];
        // rate = 1.0 guarantees spread in one tick.
        let mut rng = ChaCha8Rng::seed_from_u64(42);
        diffuse_tech(&routes, &mut cluster_techs, 1.0, &mut rng);

        assert!(
            cluster_techs[&b].contains(&tech),
            "tech should have diffused from a to b"
        );
    }

    #[test]
    fn diffusion_zero_rate_never_spreads() {
        let a = ClusterId(1);
        let b = ClusterId(2);
        let tech = TechId(0);

        let mut cluster_techs: HashMap<ClusterId, Vec<TechId>> = HashMap::new();
        cluster_techs.insert(a, vec![tech]);
        cluster_techs.insert(b, vec![]);

        let routes = vec![(a, b)];
        let mut rng = ChaCha8Rng::seed_from_u64(0);
        diffuse_tech(&routes, &mut cluster_techs, 0.0, &mut rng);

        assert!(
            !cluster_techs[&b].contains(&tech),
            "zero diffusion rate: tech must not spread"
        );
    }

    #[test]
    fn diffusion_deterministic_same_seed() {
        let a = ClusterId(10);
        let b = ClusterId(20);
        let techs: Vec<TechId> = (0..5).map(TechId).collect();

        let mut map1: HashMap<ClusterId, Vec<TechId>> = HashMap::new();
        map1.insert(a, techs.clone());
        map1.insert(b, vec![]);

        let mut map2 = map1.clone();

        let routes = vec![(a, b)];

        let mut rng1 = ChaCha8Rng::seed_from_u64(77);
        diffuse_tech(&routes, &mut map1, 0.5, &mut rng1);

        let mut rng2 = ChaCha8Rng::seed_from_u64(77);
        diffuse_tech(&routes, &mut map2, 0.5, &mut rng2);

        assert_eq!(
            map1[&b], map2[&b],
            "same seed must yield identical diffusion results"
        );
    }

    #[test]
    fn no_routes_means_no_diffusion() {
        let a = ClusterId(1);
        let b = ClusterId(2);
        let tech = TechId(0);

        let mut cluster_techs: HashMap<ClusterId, Vec<TechId>> = HashMap::new();
        cluster_techs.insert(a, vec![tech]);
        cluster_techs.insert(b, vec![]);

        let mut rng = ChaCha8Rng::seed_from_u64(0);
        diffuse_tech(&[], &mut cluster_techs, 1.0, &mut rng);

        assert!(
            !cluster_techs[&b].contains(&tech),
            "no routes: tech must not diffuse"
        );
    }
}
