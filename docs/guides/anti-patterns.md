# Anti-Pattern Detection Hooks

Six hooks that detect and prevent common code anti-patterns. Each runs on `PreToolUse:Write/Edit` events and provides actionable fix suggestions.

## Hook Summary

| Hook | Pattern Detected | Severity | Languages |
|------|-----------------|----------|-----------|
| suppress-custom-retry.sh | Custom retry loops when tenacity available | WARNING | Python |
| suppress-v2-files.sh | `_v2`, `_new`, `_old` file naming | ERROR | All |
| suppress-hardcoded-strings.sh | Hardcoded provider/model/URL strings | WARNING | Python, TS, Go |
| suppress-print-statements.sh | print()/console.log when structured logger available | WARNING | Python, TS, Go |
| suppress-isolated-classes.sh | God classes (>15 methods or >300 lines) | WARNING | Python, TS |
| suppress-direct-http.sh | Direct HTTP calls without client abstraction | WARNING | Python, TS, Go |

## Hook Details

### suppress-custom-retry.sh

**What it detects**: Hand-rolled retry/backoff loops in Python when `tenacity` is declared in project dependencies.

**Patterns caught**:
- `for attempt in range(N)` with `sleep` + `try/except`
- `while True` retry loops with attempt counters
- Manual exponential backoff (`2 ** attempt`)

**Fix**:
```python
# Before (anti-pattern)
for attempt in range(5):
    try:
        result = httpx.get(url)
        break
    except Exception:
        time.sleep(2 ** attempt)

# After (correct)
from tenacity import retry, stop_after_attempt, wait_exponential

@retry(stop=stop_after_attempt(5), wait=wait_exponential())
def fetch(url: str) -> httpx.Response:
    return httpx.get(url, timeout=10)
```

---

### suppress-v2-files.sh

**What it detects**: Files with `_v2`, `_new`, `_old`, `_backup`, `_copy`, `_temp` suffixes.

**Why blocked (ERROR)**: v2 files lead to:
- Import confusion (which version to use?)
- Stale code copies that diverge
- No clear migration path

**Fix options**:
1. Modify the original file directly
2. Use feature flags for behavioral changes
3. Use interface versioning (APIv2 endpoint, not handler_v2.py)
4. If migrating, rename the original and update all imports in one commit

---

### suppress-hardcoded-strings.sh

**What it detects**: Hardcoded provider names, model identifiers, and API URLs in source code.

**Patterns caught**:
- LLM model names: `"gpt-4"`, `"claude-3"`, `"gemini-1.5"`
- API URLs: `"https://api.openai.com/..."`
- Provider identifiers: `"aws"`, `"openai"`, `"anthropic"` as string literals

**Excludes**: Test files, imports, comments.

**Fix**:
```python
# Before
response = client.chat("gpt-4", messages)

# After
from config import settings
response = client.chat(settings.default_model, messages)
```

---

### suppress-print-statements.sh

**What it detects**: Unstructured logging (print, console.log, fmt.Println) when a structured logging library is in project dependencies.

**Dependency triggers**:
- Python: `structlog` in deps -> blocks `print()` and `logging.getLogger()`
- TypeScript: `pino`/`winston` in deps -> blocks `console.log()`
- Go: `zerolog`/`zap` in deps -> blocks `fmt.Println()`

**Fix**:
```python
# Before
print(f"Processing user {user_id}")

# After
import structlog
log = structlog.get_logger()
log.info("processing_user", user_id=user_id)
```

---

### suppress-isolated-classes.sh

**What it detects**: Classes that are too large ("God classes").

**Thresholds**:
- More than 15 methods
- More than 300 lines

**Fix**:
1. **SRP decomposition**: Extract cohesive method groups into separate classes
2. **Composition**: Break into composed components instead of one monolith
3. **Strategy pattern**: Extract behavioral variations into strategy classes
4. **DTO extraction**: Move data-only methods into separate dataclasses

---

### suppress-direct-http.sh

**What it detects**: Direct HTTP client calls (requests.get, fetch(), http.Get) in business logic.

**Why**: Direct calls scatter URL construction, authentication, retry logic, timeout handling, and error mapping across the codebase.

**Excludes**: Files in `clients/`, `adapters/`, `infrastructure/`, `transport/` directories. Files named `*client*`, `*http*`, `*fetcher*`.

**Fix**:
```python
# Before (scattered in business logic)
response = requests.get(f"{BASE_URL}/users/{user_id}", headers=auth_headers)

# After (centralized client)
class UserClient:
    def __init__(self, base_url: str, auth: Auth):
        self.client = httpx.AsyncClient(base_url=base_url, auth=auth)

    async def get_user(self, user_id: str) -> User:
        response = await self.client.get(f"/users/{user_id}")
        response.raise_for_status()
        return User.model_validate(response.json())
```

---

## CIV-Specific Anti-Patterns

### suppress-floating-point-simulation.md

**What it detects**: Use of floating-point arithmetic for simulation state.

**Why blocked**: Floating-point rounding errors accumulate across simulation ticks, making determinism impossible. CIV requires bit-exact reproducibility.

**Fix**:
```rust
// Before (ANTI-PATTERN)
pub struct Resource {
    amount: f64,
}

// After (CORRECT)
pub struct Resource {
    amount: i64,  // Fixed-point: represents smallest unit
}

// Helper functions for scaling
const RESOURCE_SCALE: i64 = 1_000_000; // 6 decimal places
fn to_scaled(f: f64) -> i64 { (f * RESOURCE_SCALE as f64) as i64 }
fn from_scaled(i: i64) -> f64 { i as f64 / RESOURCE_SCALE as f64 }
```

---

### suppress-hashmap-usage.sh

**What it detects**: Use of HashMap when deterministic iteration order is required.

**Why**: HashMap iteration order is undefined. Any code that traverses the map for simulation events must use BTreeMap for consistent ordering across runs.

**Fix**:
```rust
// Before (ANTI-PATTERN)
use std::collections::HashMap;
let mut events: HashMap<String, Event> = HashMap::new();

// After (CORRECT)
use std::collections::BTreeMap;
let mut events: BTreeMap<String, Event> = BTreeMap::new();
```

---

### suppress-allocation-regime-fallbacks.sh

**What it detects**: Code that falls back to defaults when an allocation regime is not configured.

**Why blocked**: Fallbacks hide configuration errors. CIV simulation must fail loudly if regime is missing.

**Fix**:
```rust
// Before (ANTI-PATTERN)
let regime = allocation_config
    .get("resource_regime")
    .unwrap_or_else(|| default_regime());  // Fallback!

// After (CORRECT)
let regime = allocation_config
    .get("resource_regime")
    .expect("FATAL: resource_regime not configured");
```

---

### suppress-sim-state-reads-during-tick.sh

**What it detects**: Reading live simulation state during a tick instead of using the snapshotted state from tick start.

**Why**: Live reads can cause race conditions and non-deterministic behavior. Each tick must operate on an immutable snapshot.

**Fix**:
```rust
// Before (ANTI-PATTERN)
fn tick(sim: &mut Simulation) {
    for citizen in &mut sim.citizens {  // LIVE mutations!
        citizen.hunger = sim.get_current_food() - citizen.eaten;
    }
}

// After (CORRECT)
fn tick(sim_state: &SimulationState, changes: &mut Changes) {
    // sim_state is immutable snapshot from tick start
    for citizen_id in &sim_state.citizens {
        let food = sim_state.get_food(citizen_id);
        changes.update_hunger(citizen_id, food);
    }
}
```

---

## Integration

### Claude Code Hooks (settings.json)

```json
{
  "hooks": {
    "PreToolUse": [
      {
        "matcher": "Write|Edit",
        "hooks": [
          "hooks/suppress-custom-retry.sh $FILE_PATH",
          "hooks/suppress-v2-files.sh $FILE_PATH",
          "hooks/suppress-hardcoded-strings.sh $FILE_PATH",
          "hooks/suppress-print-statements.sh $FILE_PATH",
          "hooks/suppress-isolated-classes.sh $FILE_PATH",
          "hooks/suppress-direct-http.sh $FILE_PATH"
        ]
      }
    ]
  }
}
```

### Pre-commit (local hooks)

```yaml
- repo: local
  hooks:
    - id: no-v2-files
      name: Block v2/new/old file creation
      entry: hooks/suppress-v2-files.sh
      language: script
      stages: [pre-commit]
```

## Testing Hooks

Each hook can be tested standalone:

```bash
# Test v2 file detection (should print error and exit 1)
echo "test" > /tmp/handler_v2.py
./hooks/suppress-v2-files.sh /tmp/handler_v2.py
echo "Exit code: $?"

# Test custom retry detection (should print warning)
cat > /tmp/retry_test.py << 'EOF'
for attempt in range(5):
    try:
        result = httpx.get(url)
        break
    except Exception:
        time.sleep(2 ** attempt)
EOF
./hooks/suppress-custom-retry.sh /tmp/retry_test.py
```
