# ADR-003: Policy and Institution Modeling

**Date**: 2026-04-04  
**Status**: Proposed  
**Deciders**: Civis Architecture Team  
**Related**: ADR-001, ADR-002, SPEC.md  

## Context

Existing ABM platforms treat policy as external parameters or simple rules. Civis requires first-class policy and institution modeling because:

1. **Policy simulation is the primary use case** — users want to test "what if" scenarios
2. **Institutions shape agent behavior** — governance, laws, and norms are not emergent
3. **Multi-level governance is essential** — local, regional, national policies interact
4. **Policy diffusion and learning** — policies spread between institutions

### Gap Analysis

| Platform | Policy Support | Institution Modeling | Governance Levels |
|----------|---------------|---------------------|-------------------|
| NetLogo | Parameters only | None | None |
| MASON | Custom code | None | None |
| UrbanSim | Land use policies | Implicit | City only |
| Paradox games | Hardcoded laws | Simplified | National only |
| AnyLogic | Flowchart rules | None | None |

**Conclusion**: No existing platform provides the policy modeling depth required by Civis.

## Decision

**Adopt a multi-level policy architecture with the following components:**

1. **Institutions** — Organizations that create and enforce policies
2. **Policies** — Rules that constrain or modify agent behavior
3. **Governance Levels** — Hierarchical jurisdiction (local, regional, national, global)
4. **Policy Instruments** — Mechanisms for policy implementation (taxes, regulations, nudges)
5. **Legitimacy and Compliance** — Social acceptance and enforcement of policies

### Architecture Overview

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                      Civis Policy Architecture                               │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                             │
│  ┌──────────────────────────────────────────────────────────────────────┐  │
│  │                     Governance Levels                                   │  │
│  │                                                                         │  │
│  │   Global     → Treaties, international law, global standards        │  │
│  │   National   → Federal laws, national institutions                      │  │
│  │   Regional   → State/provincial laws, regional authorities             │  │
│  │   Local      → Municipal laws, local councils                           │  │
│  │                                                                         │  │
│  │   (Each level can create policies binding lower levels)                │  │
│  └──────────────────────────────────────────────────────────────────────┘  │
│                                                                             │
│  ┌──────────────────────────────────────────────────────────────────────┐  │
│  │                       Institution Types                                 │  │
│  │                                                                         │  │
│  │   Government       → Legislature, executive, judiciary                  │  │
│  │   Bureaucracy      → Administrative agencies, regulators                │  │
│  │   International    → UN, trade blocs, alliances                         │  │
│  │   Shadow           → Informal power networks, corruption                │  │
│  │   Media            → Press, social media platforms                      │  │
│  │   Economic         → Central banks, trade organizations               │  │
│  └──────────────────────────────────────────────────────────────────────┘  │
│                                                                             │
│  ┌──────────────────────────────────────────────────────────────────────┐  │
│  │                       Policy Types                                      │  │
│  │                                                                         │  │
│  │   Economic         → Taxation, subsidies, trade policy               │  │
│  │   Social           → Education, healthcare, welfare                     │  │
│  │   Environmental    → Emissions limits, conservation                   │  │
│  │   Security         → Military, police, surveillance                   │  │
│  │   Infrastructure   → Transport, utilities, communications             │  │
│  │                                                                         │  │
│  └──────────────────────────────────────────────────────────────────────┘  │
│                                                                             │
│  ┌──────────────────────────────────────────────────────────────────────┐  │
│  │                      Policy Instruments                                 │  │
│  │                                                                         │  │
│  │   Regulation       → Laws, standards, prohibitions                      │  │
│  │   Taxation         → Income tax, sales tax, carbon tax                │  │
│  │   Subsidy          → Grants, tax breaks, transfers                    │  │
│  │   Provision        → Direct service delivery                            │  │
│  │   Information      → Nudges, education, propaganda                    │  │
│  │   Property Rights  → Ownership, licensing, zoning                     │  │
│  └──────────────────────────────────────────────────────────────────────┘  │
│                                                                             │
└─────────────────────────────────────────────────────────────────────────────┘
```

## Detailed Design

### Component: Institution

```rust
/// ECS component marking an entity as an institution
#[derive(Clone, Debug)]
pub struct Institution {
    pub id: InstitutionId,
    pub name: String,
    pub level: GovernanceLevel,
    pub institution_type: InstitutionType,
    pub jurisdiction: Jurisdiction,
    pub budget: Currency,
    pub legitimacy: f32, // 0.0 to 1.0
    pub corruption: f32, // 0.0 to 1.0
    pub members: Vec<Entity>, // Agents in institution
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum GovernanceLevel {
    Local,
    Regional,
    National,
    Global,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum InstitutionType {
    Legislature,
    Executive,
    Judiciary,
    Bureaucracy,
    International,
    Shadow, // Informal power
    Media,
    Economic,
}

/// Geographic or functional jurisdiction
#[derive(Clone, Debug)]
pub enum Jurisdiction {
    Geographic { region_id: RegionId },
    Functional { scope: PolicyDomain },
    Global,
}
```

### Component: Policy

```rust
/// ECS component for active policies
#[derive(Clone, Debug)]
pub struct Policy {
    pub id: PolicyId,
    pub name: String,
    pub domain: PolicyDomain,
    pub instruments: Vec<PolicyInstrument>,
    pub enacting_institution: Entity,
    pub effective_date: Tick,
    pub expiration_date: Option<Tick>,
    pub compliance_rate: f32, // 0.0 to 1.0
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum PolicyDomain {
    Economic,
    Social,
    Environmental,
    Security,
    Infrastructure,
}

#[derive(Clone, Debug)]
pub enum PolicyInstrument {
    /// Prohibit certain behavior
    Regulation {
        target_behavior: BehaviorType,
        penalty: Penalty,
        enforcement_level: f32,
    },
    
    /// Tax on activity or property
    Taxation {
        base: TaxBase,
        rate: f32,
        collection_efficiency: f32,
    },
    
    /// Direct payment or benefit
    Subsidy {
        target: SubsidyTarget,
        amount: Currency,
        budget_limit: Currency,
    },
    
    /// Direct service provision
    Provision {
        service: ServiceType,
        capacity: u32,
        quality: f32,
    },
    
    /// Information campaign
    Information {
        message: MessageType,
        reach: f32,
        credibility: f32,
    },
    
    /// Property rights definition
    PropertyRights {
        resource: ResourceType,
        allocation: AllocationMethod,
    },
}
```

### Component: CitizenPolicyView

```rust
/// Citizen's view of a specific policy
#[derive(Clone, Debug)]
pub struct CitizenPolicyView {
    pub policy_id: PolicyId,
    pub awareness: f32,      // Knows about policy
    pub understanding: f32,  // Understands implications
    pub approval: f32,       // -1.0 (oppose) to 1.0 (support)
    pub compliance: f32,     // 0.0 to 1.0 (likelihood to comply)
    pub impact_estimate: f32, // Estimated personal impact
}

/// Component storing all policy views for a citizen
#[derive(Clone, Debug, Default)]
pub struct PolicyViews {
    pub views: Vec<CitizenPolicyView>,
}
```

### System: PolicyEnforcementSystem

```rust
pub struct PolicyEnforcementSystem;

impl PolicyEnforcementSystem {
    pub fn run(world: &mut World, rng: &mut ChaCha8Rng) {
        // Query all active policies
        let policy_query = world.query_mut::<(&Institution, &Policy)>();
        
        for (_, (institution, policy)) in policy_query {
            // Determine effective enforcement
            let effective_enforcement = institution.legitimacy 
                * (1.0 - institution.corruption)
                * policy.compliance_rate;
            
            // Apply policy effects based on instruments
            for instrument in &policy.instruments {
                match instrument {
                    PolicyInstrument::Taxation { base, rate, .. } => {
                        Self::apply_tax(world, institution, base, *rate);
                    }
                    PolicyInstrument::Regulation { target_behavior, penalty, .. } => {
                        Self::enforce_regulation(world, rng, target_behavior, penalty, effective_enforcement);
                    }
                    PolicyInstrument::Subsidy { target, amount, budget_limit } => {
                        Self::distribute_subsidy(world, target, *amount, *budget_limit);
                    }
                    // ... other instruments
                }
            }
        }
    }
    
    fn apply_tax(world: &mut World, institution: &Institution, base: &TaxBase, rate: f32) {
        let mut query = world.query_mut::<(&mut Wealth, &Position, &Citizen)>();
        
        for (_, (wealth, position, _)) in &mut query {
            // Check if in jurisdiction
            if !institution.jurisdiction.contains(position) {
                continue;
            }
            
            // Calculate tax based on base
            let tax_amount = match base {
                TaxBase::Income => wealth.amount * rate as f64,
                TaxBase::Property => /* property value */ Currency::from_num(0),
                TaxBase::Sales => /* sales tracking needed */ Currency::from_num(0),
            };
            
            // Deduct from citizen
            wealth.amount -= tax_amount;
            
            // Add to institution budget
            // (would need mutable reference to institution)
        }
    }
    
    fn enforce_regulation(
        world: &mut World, 
        rng: &mut ChaCha8Rng,
        behavior: &BehaviorType, 
        penalty: &Penalty,
        enforcement_rate: f32
    ) {
        // Find agents engaging in regulated behavior
        // Check for compliance based on enforcement rate
        // Apply penalties to non-compliant
    }
}
```

### System: PolicyFormationSystem

```rust
pub struct PolicyFormationSystem;

impl PolicyFormationSystem {
    pub fn run(world: &mut World, rng: &mut ChaCha8Rng) {
        // Query legislative institutions
        let mut query = world.query_mut::<(&Institution, &mut Agenda)>();
        
        for (_, (institution, agenda)) in query {
            if institution.institution_type != InstitutionType::Legislature {
                continue;
            }
            
            // Process agenda items
            for item in agenda.items.iter() {
                // Simulate voting
                let votes = Self::simulate_vote(world, institution, item, rng);
                
                if votes.approval > 0.5 {
                    // Create new policy
                    let policy = Self::create_policy(item, institution.id);
                    
                    // Add to world
                    world.spawn((policy,));
                }
            }
        }
    }
    
    fn simulate_vote(
        world: &World, 
        institution: &Institution, 
        item: &AgendaItem,
        rng: &mut ChaCha8Rng
    ) -> VoteResult {
        let mut approval = 0.0f32;
        
        // Query member views
        for member_id in &institution.members {
            if let Ok(views) = world.get::<PolicyViews>(*member_id) {
                if let Some(view) = views.views.iter().find(|v| v.policy_id == item.policy_id) {
                    approval += view.approval;
                }
            }
        }
        
        let member_count = institution.members.len().max(1) as f32;
        VoteResult {
            approval: approval / member_count,
            turnout: member_count / institution.members.len() as f32,
        }
    }
}
```

### System: OpinionDynamicsSystem

```rust
pub struct OpinionDynamicsSystem;

impl OpinionDynamicsSystem {
    /// Update citizen policy views based on experience and social influence
    pub fn run(world: &mut World, rng: &mut ChaCha8Rng) {
        let mut query = world.query_mut::<(&mut PolicyViews, &SocialNetwork, &Ideology)>();
        
        for (_, (views, network, ideology)) in &mut query {
            for view in views.views.iter_mut() {
                // Direct experience update
                let experience_impact = Self::calculate_experience_impact(view);
                
                // Social influence from network
                let social_impact = Self::calculate_social_influence(
                    world, network, view.policy_id
                );
                
                // Ideological consistency
                let ideological_pull = Self::ideological_alignment(
                    ideology, view.policy_id
                );
                
                // Update approval with bounded confidence
                let total_influence = experience_impact * 0.4 
                    + social_impact * 0.4 
                    + ideological_pull * 0.2;
                
                view.approval = Self::bounded_update(
                    view.approval, 
                    total_influence,
                    0.2 // confidence bound
                );
            }
        }
    }
    
    fn bounded_update(current: f32, influence: f32, bound: f32) -> f32 {
        if (influence - current).abs() < bound {
            // Within confidence bound: move toward influence
            current + (influence - current) * 0.1
        } else {
            // Outside confidence bound: ignore
            current
        }.clamp(-1.0, 1.0)
    }
}
```

## Policy Scenarios

### Scenario 1: Carbon Tax

```yaml
policy:
  name: "Carbon Tax Act"
  domain: Environmental
  instruments:
    - type: Taxation
      base: Emissions
      rate: 0.05  # $50/ton CO2
    - type: Subsidy
      target: GreenTechnology
      amount: 1000000
  jurisdiction:
    type: National
  expected_outcomes:
    - emissions_reduction: 0.30
    - gdp_impact: -0.02
    - public_approval: initial_negative, long_term_neutral
```

### Scenario 2: Universal Basic Income

```yaml
policy:
  name: "UBI Program"
  domain: Social
  instruments:
    - type: Subsidy
      target: AllAdults
      amount: 12000  # Annual
    - type: Taxation
      base: Income
      rate: 0.40  # Progressive adjustment
  jurisdiction:
    type: National
  expected_outcomes:
    - poverty_reduction: 0.50
    - employment_impact: -0.05
    - inflation: 0.03
```

### Scenario 3: Zoning Reform

```yaml
policy:
  name: "Upzoning Initiative"
  domain: Infrastructure
  instruments:
    - type: Regulation
      target: HousingDensity
      penalty: None  # Incentive-based
    - type: PropertyRights
      resource: LandUse
      allocation: MarketWithDensityBonus
  jurisdiction:
    type: Local
  expected_outcomes:
    - housing_supply_increase: 0.30
    - price_reduction: 0.15
    - neighborhood_change: varies
```

## Consequences

### Positive

1. **Realistic Policy Simulation**: Can test complex multi-instrument policies
2. **Legitimacy Modeling**: Captures why policies fail even if "optimal"
3. **Diffusion Studies**: Model how policies spread between jurisdictions
4. **Counterfactual Analysis**: Compare world with/without specific policies
5. **Institutional Design**: Test different governance structures

### Negative

1. **Complexity**: Policy modeling adds significant codebase complexity
2. **Calibration**: Requires extensive data to calibrate policy effects
3. **Validation**: Difficult to validate against real policy outcomes
4. **Political Sensitivity**: Policy simulations can be controversial
5. **Performance**: Additional systems increase computational load

### Mitigations

| Concern | Mitigation |
|---------|------------|
| Complexity | Modular design; optional policy module |
| Calibration | Default parameters from literature; sensitivity analysis |
| Validation | Scenario library with known outcomes |
| Sensitivity | Document assumptions; multiple perspectives |
| Performance | Lazy evaluation; optional policy detail levels |

## Alternatives Considered

### Alternative 1: Hardcoded Policies (Rejected)

**Approach**: Specific policy types as code (CarbonTax, UBI, etc.)

**Rejection**: Not flexible enough; every new policy requires code changes

### Alternative 2: Rule-Based Policies (Rejected)

**Approach**: Declarative rules in external DSL

**Rejection**: Rules too limited for complex interactions; debugging difficult

### Alternative 3: Neural Policy Agents (Rejected)

**Approach**: ML-trained policy makers

**Rejection**: Non-deterministic; opaque; hard to interpret

### Alternative 4: Game Theory Only (Rejected)

**Approach**: Pure game-theoretic modeling

**Rejection**: Can't capture institutional complexity; assumes rationality

## Implementation Roadmap

### Phase 1: Foundation

- [ ] Institution component and basic types
- [ ] Policy component with Regulation instrument
- [ ] PolicyEnforcementSystem
- [ ] Basic jurisdiction handling

### Phase 2: Instruments

- [ ] Taxation instrument
- [ ] Subsidy instrument
- [ ] Provision instrument
- [ ] Information instrument
- [ ] Property rights instrument

### Phase 3: Dynamics

- [ ] PolicyFormationSystem
- [ ] OpinionDynamicsSystem
- [ ] Policy diffusion between jurisdictions
- [ ] Legitimacy and corruption dynamics

### Phase 4: Validation

- [ ] Historical policy scenario library
- [ ] Sensitivity analysis tools
- [ ] Policy comparison framework

## References

- [UrbanSim](https://github.com/UDST/urbansim) — Land use policy simulation
- [POLIS](https://www.polis.iupui.edu/) — Policy simulation framework
- [Agent-Based Policy Modeling](https://www.jasss.org/20/1/6.html) — JASSS paper
- [Institutional Theory](https://en.wikipedia.org/wiki/Institutional_theory) — Wikipedia overview
- [Policy Diffusion](https://www.cambridge.org/core/books/policy-diffusion/7E4B7A0F9A5B9C8D5E1F2A3B4C5D6E7F) — Book on policy spread

---

*This ADR establishes policy modeling as a first-class concern in Civis architecture.*
