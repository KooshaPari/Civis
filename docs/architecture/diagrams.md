# Architecture Diagrams

This page surfaces the core architectural diagrams that define DINOForge's structure, pack lifecycle, and quality governance.

## Layer Stack

The architecture follows a layered design from the game engine up through the mod platform:

```mermaid
graph TD
    A["🎮 DINO Game<br/>Unity ECS + BepInEx"] --> B["Runtime<br/>Plugin Bootstrap<br/>ECS Bridge"]
    B --> C["SDK<br/>Registries<br/>Validators<br/>ContentLoader"]
    C --> D["Domain Plugins<br/>Warfare<br/>Economy<br/>Scenario<br/>UI"]
    D --> E["Packs<br/>Content Bundles<br/>Manifests<br/>Assets"]
    E --> F["User Mods<br/>Custom Themes<br/>Balance Tweaks<br/>Total Conversions"]
    
    style A fill:#ff6b6b
    style B fill:#4ecdc4
    style C fill:#45b7d1
    style D fill:#96ceb4
    style E fill:#ffeaa7
    style F fill:#dfe6e9
```

Each layer builds on the previous: the Runtime exposes the ECS bridge, the SDK provides registries and validators, domain plugins extend the SDK for specific gameplay areas, and packs instantiate content within those domains.

## Pack Load Sequence

When a pack is discovered, it flows through validation, compatibility checking, registry insertion, and runtime activation:

```mermaid
sequenceDiagram
    participant Discovery as Pack Discovery
    participant YAML as YAML Parse
    participant Validate as IValidatable
    participant CompatCheck as Compatibility
    participant Registry as Registry Insert
    participant Runtime as Runtime Active

    Discovery->>YAML: pack.yaml found
    YAML->>YAML: Deserialize manifest
    YAML->>Validate: Call Validate()
    alt Validation Success
        Validate-->>CompatCheck: ✓ Schema valid
        CompatCheck->>CompatCheck: Check deps exist<br/>Check conflicts absent
        alt Compat OK
            CompatCheck-->>Registry: ✓ Compatible
            Registry->>Registry: Insert entries<br/>(units, factions, etc.)
            Registry-->>Runtime: ✓ Pack Active
        else Conflict/Missing Dep
            CompatCheck-->>Runtime: ✗ Deactivate pack
        end
    else Validation Fails
        Validate-->>Runtime: ✗ Skip pack
    end
```

A pack must pass schema validation before compatibility is checked. Conflicts or missing dependencies cause the pack to be deactivated rather than loaded.

## Pattern Catalog Lifecycle

Quality patterns are detected, governed, and retired through a structured CI/remediation cycle:

```mermaid
graph LR
    A["🔍 Sweep<br/>Regex/AST scan<br/>Find all instances"] -->|Script runs| B["📊 Detect<br/>Count HIGH/MED/LOW<br/>Categorize violations"]
    B -->|CI Gate| C{"HIGH > Threshold?"}
    C -->|YES| D["⛔ CI Fails<br/>Blocks merge"]
    C -->|NO| E["📋 Governance<br/>Create allowlist<br/>Write CLAUDE.md"]
    D --> F["✏️ Remediation<br/>Fix code sites<br/>Update allowlist"]
    F --> G["✅ Verify<br/>Re-run detection<br/>HIGH = 0"]
    G -->|Complete| H["🏁 RETIRED<br/>Pattern closed<br/>Coverage gate"]
    E --> H
    
    style A fill:#4ecdc4
    style B fill:#45b7d1
    style C fill:#f7b731
    style D fill:#ff6b6b
    style E fill:#96ceb4
    style F fill:#ffeaa7
    style G fill:#74b9ff
    style H fill:#55efc4
```

Patterns are detected automatically on every CI run. If violations exceed the threshold, the CI gate blocks the merge and enforces remediation before retry. Passing violations are tracked in allowlists to ensure pattern closure.

## Smart-Contract Proof System Pipeline

The proof system creates cryptographically-verifiable bundles that certify game behavior, bridging the gap between autonomous testing and human-auditable proof. Each bundle is signed with a session HMAC and Merkle root, then validated against policy rules before archival:

```mermaid
graph TD
    A["🎮 Game ECS<br/>Entity State"] -->|F9/F10| B["BridgeReceipt<br/>HMAC-signed<br/>SessionHmac"]
    B -->|Serialize| C["JsonRpcResponse<br/>Proof payload"]
    C -->|TCP| D["GameClient<br/>Receive + Validate"]
    D -->|Verify| E["BridgeReceiptVerifier<br/>HMAC check<br/>Monotonic frame guard"]
    E -->|✓ Valid| F["Aggregator<br/>Batch receipts<br/>Compute Merkle root"]
    F -->|Per-session| G["Merkle Root<br/>Proof fingerprint"]
    G -->|Evaluate| H["proof_policy.yaml<br/>Condition rules<br/>SDD assertions"]
    H -->|Rule match| I["Policy Engine<br/>Pass/Fail decision"]
    I -->|✓ Pass| J["cosign sign<br/>Bundle artifact<br/>JSON + signature"]
    J -->|Write| K["docs/proof/bundles/<br/>session-manifest.json<br/>+ .sig file"]
    K -->|CI Gate| L["proof-gate.yml<br/>Verify bundle<br/>Require valid proof"]
    L -->|✓ Valid| M["✅ CI Pass<br/>Proof gates closed"]
    L -->|✗ Invalid| N["⛔ CI Fail<br/>Reject merge<br/>Audit required"]
    
    style A fill:#ff6b6b
    style B fill:#4ecdc4
    style C fill:#45b7d1
    style D fill:#96ceb4
    style E fill:#ffeaa7
    style F fill:#dfe6e9
    style G fill:#f7b731
    style H fill:#74b9ff
    style I fill:#a29bfe
    style J fill:#6c5ce7
    style K fill:#fd79a8
    style M fill:#55efc4
    style N fill:#ff7675
```

The proof pipeline ensures that game feature claims are backed by signed, policy-validated receipts. HMAC signatures authenticate that receipts originate from the running game session. Merkle roots compress the full receipt corpus into a single verifiable fingerprint. Policy rules (written in YAML) define what constitutes valid proof for each feature claim. CI gates require valid bundles before merge, preventing unsubstantiated claims from reaching the codebase.
