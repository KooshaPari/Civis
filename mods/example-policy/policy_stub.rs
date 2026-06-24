// Example PolicyMod outline for `example-policy` (CIV-0700 §5).
//
// Not compiled to WASM yet — host-side compile check only via `civlab-sdk` tests.
// Imports are provided by the including test module in `civlab-sdk/src/policy.rs`.

/// Stub carbon-tax policy: raises tax when CO₂ exceeds a threshold.
#[derive(Debug)]
pub struct ExampleCarbonTaxPolicy {
    co2_threshold_milliunits: i64,
}

impl ExampleCarbonTaxPolicy {
    /// Default CO₂ threshold (420 ppm expressed as milli-ppm).
    const DEFAULT_CO2_THRESHOLD: i64 = 420_000;
}

impl Default for ExampleCarbonTaxPolicy {
    fn default() -> Self {
        Self {
            co2_threshold_milliunits: Self::DEFAULT_CO2_THRESHOLD,
        }
    }
}

impl PolicyMod for ExampleCarbonTaxPolicy {
    fn on_tick(&mut self, ctx: &PolicyContext) -> Vec<PolicyAction> {
        let co2 = ctx
            .climate
            .map(|c| c.co2_ppm_milliunits)
            .unwrap_or_default();
        if co2 > self.co2_threshold_milliunits {
            vec![PolicyAction::SetTaxRate {
                rate_permille: 150,
            }]
        } else {
            Vec::new()
        }
    }

    fn on_event(&mut self, event: &SimEvent) -> Vec<PolicyAction> {
        let _ = event;
        Vec::new()
    }

    fn metadata(&self) -> ModMetadata {
        ModMetadata {
            id: "example-policy".to_owned(),
            name: "Example Carbon Tax Policy".to_owned(),
            version: "0.1.0".to_owned(),
            subscribed_event_hashes: Vec::new(),
        }
    }
}
