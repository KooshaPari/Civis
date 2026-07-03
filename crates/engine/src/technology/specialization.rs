//! Cluster technology specialization (FR-CIV-TECH).
//!
//! Each cluster develops a dominant technological specialization based on its
//! environment. The mapping is deterministic — same environment, same
//! specialization.

/// Describes the dominant technological focus of a cluster.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TechSpecialization {
    /// Fertile land and stable climate drive agricultural innovation.
    Agricultural,
    /// Coastal or riverine access enables maritime technology.
    Maritime,
    /// Hostile neighbours or scarce resources push military development.
    Military,
    /// High population density and institutional support favour scholarship.
    Scholarly,
    /// Resource-rich environments with settled populations go industrial.
    Industrial,
    /// No dominant specialization has emerged yet.
    None,
}

/// Environmental inputs that drive specialization.
///
/// All fields use normalized `f32` scores in `[0.0, 1.0]` unless noted.
#[derive(Debug, Clone)]
pub struct ClusterEnvironment {
    /// Proportion of fertile tiles in the cluster footprint.
    pub fertility: f32,
    /// `true` when the cluster has a coastline or navigable river.
    pub has_water_access: bool,
    /// Threat level from neighbouring hostile factions (`0.0` = safe).
    pub threat_level: f32,
    /// Current cluster population (used for density proxy).
    pub population: u32,
    /// Number of scholarly/institutional buildings.
    pub institution_count: u32,
    /// Proportion of ore/fuel resource tiles.
    pub resource_richness: f32,
}

/// Compute the dominant specialization for a cluster based on its environment.
///
/// Returns [`TechSpecialization::None`] when no dimension scores above its
/// threshold.
#[must_use]
pub fn compute_specialization(environment: &ClusterEnvironment) -> TechSpecialization {
    const FERTILITY_THRESHOLD: f32 = 0.6;
    const THREAT_THRESHOLD: f32 = 0.5;
    const SCHOLAR_POP: u32 = 500;
    const SCHOLAR_INST: u32 = 3;
    const INDUSTRIAL_RESOURCE: f32 = 0.5;
    const INDUSTRIAL_POP: u32 = 1_000;

    // Priority order: Military > Maritime > Scholarly > Industrial > Agricultural.
    if environment.threat_level >= THREAT_THRESHOLD {
        return TechSpecialization::Military;
    }
    if environment.has_water_access {
        return TechSpecialization::Maritime;
    }
    if environment.population >= SCHOLAR_POP
        && environment.institution_count >= SCHOLAR_INST
    {
        return TechSpecialization::Scholarly;
    }
    if environment.resource_richness >= INDUSTRIAL_RESOURCE
        && environment.population >= INDUSTRIAL_POP
    {
        return TechSpecialization::Industrial;
    }
    if environment.fertility >= FERTILITY_THRESHOLD {
        return TechSpecialization::Agricultural;
    }
    TechSpecialization::None
}

#[cfg(test)]
mod tests {
    use super::*;

    fn base_env() -> ClusterEnvironment {
        ClusterEnvironment {
            fertility: 0.0,
            has_water_access: false,
            threat_level: 0.0,
            population: 100,
            institution_count: 0,
            resource_richness: 0.0,
        }
    }

    #[test]
    fn high_threat_yields_military() {
        let env = ClusterEnvironment {
            threat_level: 0.8,
            ..base_env()
        };
        assert_eq!(compute_specialization(&env), TechSpecialization::Military);
    }

    #[test]
    fn water_access_yields_maritime() {
        let env = ClusterEnvironment {
            has_water_access: true,
            ..base_env()
        };
        assert_eq!(compute_specialization(&env), TechSpecialization::Maritime);
    }

    #[test]
    fn scholars_yield_scholarly() {
        let env = ClusterEnvironment {
            population: 600,
            institution_count: 4,
            ..base_env()
        };
        assert_eq!(compute_specialization(&env), TechSpecialization::Scholarly);
    }

    #[test]
    fn rich_resources_and_pop_yield_industrial() {
        let env = ClusterEnvironment {
            resource_richness: 0.7,
            population: 2_000,
            ..base_env()
        };
        assert_eq!(compute_specialization(&env), TechSpecialization::Industrial);
    }

    #[test]
    fn fertile_land_yields_agricultural() {
        let env = ClusterEnvironment {
            fertility: 0.9,
            ..base_env()
        };
        assert_eq!(
            compute_specialization(&env),
            TechSpecialization::Agricultural
        );
    }

    #[test]
    fn bare_environment_yields_none() {
        assert_eq!(compute_specialization(&base_env()), TechSpecialization::None);
    }

    #[test]
    fn military_priority_over_maritime() {
        let env = ClusterEnvironment {
            threat_level: 0.8,
            has_water_access: true,
            ..base_env()
        };
        // Military takes priority
        assert_eq!(compute_specialization(&env), TechSpecialization::Military);
    }
}
