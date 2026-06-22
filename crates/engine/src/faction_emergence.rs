use glam::IVec3;

const IDEOLOGY_DIMENSIONS: usize = 8;
const MAX_KMEANS_ITERS: usize = 100;
const CONVERGENCE_DELTA: f32 = 0.01;
const SPLIT_VARIANCE_THRESHOLD: f32 = 0.4;
const MERGE_SIMILARITY_THRESHOLD: f32 = 0.85;
const TERRITORY_SCALE: f32 = 100.0;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct AgentIdeology {
    pub values: [f32; IDEOLOGY_DIMENSIONS],
}

#[derive(Debug, Clone, PartialEq)]
pub struct FactionSeed {
    pub ideology_centroid: [f32; IDEOLOGY_DIMENSIONS],
    pub territory_center: IVec3,
    pub member_count: u32,
    pub id: u32,
}

pub fn cluster_into_factions(ideologies: &[AgentIdeology], k: usize) -> Vec<FactionSeed> {
    if ideologies.is_empty() || k == 0 {
        return Vec::new();
    }

    let cluster_count = k.min(ideologies.len());
    let mut centroids = ideologies
        .iter()
        .take(cluster_count)
        .map(|ideology| ideology.values)
        .collect::<Vec<_>>();
    let mut assignments = vec![0usize; ideologies.len()];

    for _ in 0..MAX_KMEANS_ITERS {
        let mut changed = false;

        for (index, ideology) in ideologies.iter().enumerate() {
            let cluster = nearest_centroid(&centroids, &ideology.values);
            if assignments[index] != cluster {
                assignments[index] = cluster;
                changed = true;
            }
        }

        rebalance_empty_clusters(ideologies, &mut assignments, &centroids);

        let (next_centroids, counts) = recompute_centroids(ideologies, &assignments, cluster_count);
        let max_delta = centroids
            .iter()
            .zip(next_centroids.iter())
            .map(|(current, next)| centroid_delta(current, next))
            .fold(0.0_f32, f32::max);

        centroids = next_centroids;

        if !changed || max_delta < CONVERGENCE_DELTA {
            return build_factions(&centroids, &counts);
        }
    }

    let mut final_assignments = vec![0usize; ideologies.len()];
    for (index, ideology) in ideologies.iter().enumerate() {
        final_assignments[index] = nearest_centroid(&centroids, &ideology.values);
    }
    rebalance_empty_clusters(ideologies, &mut final_assignments, &centroids);
    let (final_centroids, counts) =
        recompute_centroids(ideologies, &final_assignments, cluster_count);

    build_factions(&final_centroids, &counts)
}

pub fn should_faction_split(_f: &FactionSeed, members: &[AgentIdeology]) -> bool {
    if members.is_empty() {
        return false;
    }

    let mut sum = 0.0_f32;
    let mut count = 0usize;
    for member in members {
        for value in member.values {
            sum += value;
            count += 1;
        }
    }

    if count == 0 {
        return false;
    }

    let mean = sum / count as f32;
    let mut variance = 0.0_f32;
    for member in members {
        for value in member.values {
            let delta = value - mean;
            variance += delta * delta;
        }
    }

    (variance / count as f32) > SPLIT_VARIANCE_THRESHOLD
}

pub fn should_faction_merge(a: &FactionSeed, b: &FactionSeed) -> bool {
    cosine_similarity(&a.ideology_centroid, &b.ideology_centroid) > MERGE_SIMILARITY_THRESHOLD
}

fn build_factions(centroids: &[[f32; IDEOLOGY_DIMENSIONS]], counts: &[usize]) -> Vec<FactionSeed> {
    let mut factions = Vec::new();

    for (cluster_index, centroid) in centroids.iter().enumerate() {
        let member_count = counts.get(cluster_index).copied().unwrap_or(0);
        if member_count == 0 {
            continue;
        }

        factions.push(FactionSeed {
            ideology_centroid: *centroid,
            territory_center: territory_center_from_centroid(centroid),
            member_count: member_count as u32,
            id: cluster_index as u32,
        });
    }
    factions
}

fn recompute_centroids(
    ideologies: &[AgentIdeology],
    assignments: &[usize],
    cluster_count: usize,
) -> (Vec<[f32; IDEOLOGY_DIMENSIONS]>, Vec<usize>) {
    let mut sums = vec![[0.0_f32; IDEOLOGY_DIMENSIONS]; cluster_count];
    let mut counts = vec![0usize; cluster_count];

    for (ideology, &cluster) in ideologies.iter().zip(assignments.iter()) {
        counts[cluster] += 1;
        for dimension in 0..IDEOLOGY_DIMENSIONS {
            sums[cluster][dimension] += ideology.values[dimension];
        }
    }

    let mut centroids = vec![[0.0_f32; IDEOLOGY_DIMENSIONS]; cluster_count];
    for cluster in 0..cluster_count {
        if counts[cluster] == 0 {
            continue;
        }

        for dimension in 0..IDEOLOGY_DIMENSIONS {
            centroids[cluster][dimension] = sums[cluster][dimension] / counts[cluster] as f32;
        }
    }

    (centroids, counts)
}

fn rebalance_empty_clusters(
    ideologies: &[AgentIdeology],
    assignments: &mut [usize],
    centroids: &[[f32; IDEOLOGY_DIMENSIONS]],
) {
    let cluster_count = centroids.len();
    if cluster_count == 0 || ideologies.is_empty() {
        return;
    }

    loop {
        let mut counts = vec![0usize; cluster_count];
        for &cluster in assignments.iter() {
            counts[cluster] += 1;
        }

        let empty_cluster = counts.iter().position(|&count| count == 0);
        let Some(empty_cluster) = empty_cluster else {
            break;
        };

        if let Some((point_index, _)) = farthest_point(ideologies, assignments, centroids) {
            assignments[point_index] = empty_cluster;
        } else {
            break;
        }
    }
}

fn farthest_point(
    ideologies: &[AgentIdeology],
    assignments: &[usize],
    centroids: &[[f32; IDEOLOGY_DIMENSIONS]],
) -> Option<(usize, usize)> {
    let mut best: Option<(usize, usize, f32)> = None;

    for (index, ideology) in ideologies.iter().enumerate() {
        let cluster = assignments[index];
        let distance = squared_distance(&ideology.values, &centroids[cluster]);
        match best {
            Some((_, _, best_distance)) if distance <= best_distance => {}
            _ => best = Some((index, cluster, distance)),
        }
    }

    best.map(|(index, cluster, _)| (index, cluster))
}

fn nearest_centroid(
    centroids: &[[f32; IDEOLOGY_DIMENSIONS]],
    values: &[f32; IDEOLOGY_DIMENSIONS],
) -> usize {
    let mut best_index = 0usize;
    let mut best_distance = f32::INFINITY;

    for (index, centroid) in centroids.iter().enumerate() {
        let distance = squared_distance(values, centroid);
        if distance < best_distance {
            best_distance = distance;
            best_index = index;
        }
    }

    best_index
}

fn squared_distance(a: &[f32; IDEOLOGY_DIMENSIONS], b: &[f32; IDEOLOGY_DIMENSIONS]) -> f32 {
    let mut sum = 0.0_f32;
    for dimension in 0..IDEOLOGY_DIMENSIONS {
        let delta = a[dimension] - b[dimension];
        sum += delta * delta;
    }
    sum
}

fn centroid_delta(current: &[f32; IDEOLOGY_DIMENSIONS], next: &[f32; IDEOLOGY_DIMENSIONS]) -> f32 {
    squared_distance(current, next).sqrt()
}

fn cosine_similarity(a: &[f32; IDEOLOGY_DIMENSIONS], b: &[f32; IDEOLOGY_DIMENSIONS]) -> f32 {
    let mut dot = 0.0_f32;
    let mut norm_a = 0.0_f32;
    let mut norm_b = 0.0_f32;

    for dimension in 0..IDEOLOGY_DIMENSIONS {
        dot += a[dimension] * b[dimension];
        norm_a += a[dimension] * a[dimension];
        norm_b += b[dimension] * b[dimension];
    }

    if norm_a == 0.0 || norm_b == 0.0 {
        return 0.0;
    }

    dot / (norm_a.sqrt() * norm_b.sqrt())
}

fn territory_center_from_centroid(centroid: &[f32; IDEOLOGY_DIMENSIONS]) -> IVec3 {
    IVec3::new(
        (centroid[0] * TERRITORY_SCALE).round() as i32,
        (centroid[1] * TERRITORY_SCALE).round() as i32,
        (centroid[2] * TERRITORY_SCALE).round() as i32,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ideology(values: [f32; IDEOLOGY_DIMENSIONS]) -> AgentIdeology {
        AgentIdeology { values }
    }

    fn faction(values: [f32; IDEOLOGY_DIMENSIONS]) -> FactionSeed {
        FactionSeed {
            ideology_centroid: values,
            territory_center: territory_center_from_centroid(&values),
            member_count: 1,
            id: 0,
        }
    }

    #[test]
    fn cluster_into_factions_discovers_two_clusters() {
        let ideologies = vec![
            ideology([0.00, 0.00, 0.00, 0.00, 0.02, 0.01, 0.00, 0.00]),
            ideology([0.04, 0.01, 0.00, 0.02, 0.01, 0.00, 0.00, 0.00]),
            ideology([1.00, 1.00, 1.00, 1.00, 0.98, 1.02, 1.00, 1.01]),
            ideology([0.96, 1.01, 1.00, 0.99, 1.00, 1.00, 1.02, 0.98]),
        ];

        let factions = cluster_into_factions(&ideologies, 2);

        assert_eq!(factions.len(), 2);
        assert_eq!(factions.iter().map(|f| f.member_count).sum::<u32>(), 4);
        assert!(factions
            .iter()
            .any(|f| f.member_count == 2 && f.ideology_centroid[0] < 0.5));
        assert!(factions
            .iter()
            .any(|f| f.member_count == 2 && f.ideology_centroid[0] > 0.5));
    }

    #[test]
    fn should_faction_split_triggers_on_high_variance() {
        let members = vec![
            ideology([0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0]),
            ideology([1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0]),
        ];

        assert!(should_faction_split(
            &faction([0.5; IDEOLOGY_DIMENSIONS]),
            &members
        ));
    }

    #[test]
    fn should_faction_split_stays_false_below_threshold() {
        let members = vec![
            ideology([0.49, 0.50, 0.51, 0.50, 0.49, 0.50, 0.51, 0.50]),
            ideology([0.50, 0.49, 0.50, 0.51, 0.50, 0.51, 0.50, 0.49]),
        ];

        assert!(!should_faction_split(
            &faction([0.5; IDEOLOGY_DIMENSIONS]),
            &members
        ));
    }

    #[test]
    fn should_faction_merge_checks_cosine_similarity() {
        let aligned = faction([0.9, 0.9, 0.9, 0.9, 0.1, 0.1, 0.1, 0.1]);
        let nearby = faction([0.88, 0.91, 0.92, 0.87, 0.12, 0.08, 0.11, 0.09]);
        let orthogonal = faction([0.0, 0.0, 0.0, 0.0, 1.0, 1.0, 1.0, 1.0]);

        assert!(should_faction_merge(&aligned, &nearby));
        assert!(!should_faction_merge(&aligned, &orthogonal));
    }
}
