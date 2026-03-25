# Coverage Expansion Task

## Objective
Expand DINOForge test coverage from 1248 tests (1227 main + 21 integration) to 1348+ tests by adding 100+ high-value tests targeting critical gaps.

## Current Status
- **Total tests**: 1227 main + 21 integration = 1248
- **Failing tests**: 4 ScenarioContentLoader tests (YAML parsing issues)
- **Test files**: 112 .cs files in src/Tests/

## Priority Gap Areas (in order)

### 1. SDK Core Services (HIGH PRIORITY - 30+ tests)
These core services have NO dedicated test files.

**ContentRegistrationService (RegistryImportService)**
- File: `src/SDK/ContentRegistrationService.cs`
- Required tests:
  - ValidateContent with valid/invalid YAML for each content type (units, buildings, factions, weapons, projectiles, doctrines, stats, faction_patches)
  - RegisterContent for each type
  - Error handling for missing files, invalid schema, deserialization failures
  - Test each content type registration path

**FileDiscoveryService**
- File: `src/SDK/FileDiscoveryService.cs`
- Required tests:
  - Discover files by pattern
  - Handle missing directories
  - Nested directory discovery
  - File filter edge cases

**YamlLoader**
- File: `src/SDK/YamlLoader.cs`
- Required tests:
  - Load valid YAML files
  - Handle missing files
  - Invalid YAML syntax
  - Null/empty file handling

**UniverseLoader**
- File: `src/SDK/Universe/UniverseLoader.cs`
- Required tests:
  - Load valid universe definitions
  - Handle version mismatches
  - Invalid faction definitions
  - Cycle detection in inheritance

### 2. PackCompiler Services (MEDIUM PRIORITY - 25+ tests)
These asset pipeline services have NO dedicated tests.

**AssetValidationService**
- File: `src/Tools/PackCompiler/Services/AssetValidationService.cs`
- Required tests:
  - Validate imported assets against config
  - Polycount boundary checks
  - Scale range validation
  - Material reference validation
  - LOD sanity checks
  - Valid vs invalid asset configs

**AssetImportService**
- File: `src/Tools/PackCompiler/Services/AssetImportService.cs`
- Required tests:
  - Import GLB/FBX files
  - Handle missing files
  - Extract mesh data
  - Material import
  - LOD variant generation

**AssetOptimizationService**
- File: `src/Tools/PackCompiler/Services/AssetOptimizationService.cs`
- Required tests:
  - Mesh decimation
  - LOD generation with target polycounts
  - Bound calculation
  - Quality preservation for critical features

**PrefabGenerationService**
- File: `src/Tools/PackCompiler/Services/PrefabGenerationService.cs`
- Required tests:
  - Generate prefab from JSON
  - Material assignment
  - Transform hierarchy
  - Reference integrity

**DefinitionUpdateService**
- File: `src/Tools/PackCompiler/Services/DefinitionUpdateService.cs`
- Required tests:
  - Inject visual_asset into YAML definitions
  - Handle missing definition fields
  - Version tracking
  - Batch update scenarios

**AddressablesService**
- File: `src/Tools/PackCompiler/Services/AddressablesService.cs`
- Required tests:
  - Generate catalog entries
  - Address key validation
  - Bundle reference integrity
  - Conflict detection for duplicate addresses

### 3. Bridge/Protocol Edge Cases (MEDIUM PRIORITY - 15+ tests)
ProtocolSerializationTests covers roundtrips, but missing edge cases.

**ScreenshotResult**
- Required tests:
  - Base64 encoding roundtrip with large images
  - Timestamp format validation
  - Null/empty image handling
  - Format preservation (PNG vs JPEG)

**Message Types Edge Cases**
- Null collections in QueryResult.Entities
- Empty GameStatus.LoadedPacks
- Zero-length arrays in batch operations
- Invalid JSON structure recovery

**Error Messages**
- Serialization of exception details
- Stack trace preservation in error results
- Unicode character handling in messages

### 4. Scenario Domain Fixes (MEDIUM PRIORITY - 15+ tests)
Fix 4 failing tests and add coverage.

**ScenarioContentLoader**
- Fix YAML parsing issues in existing 4 tests
- Add tests for:
  - Valid scenario files
  - Multiple scenario files
  - Nested scenario directories
  - Comprehensive scenario definitions
  - Invalid scenario YAML

**ScenarioRegistry**
- CRUD operations
- Query by ID/name
- Conflict detection

**Win Condition Validation**
- Valid condition definitions
- Invalid condition types
- Missing required fields

### 5. Validation & Schemas (MEDIUM PRIORITY - 15+ tests)

**Schema Validation**
- Valid schemas pass
- Invalid schemas fail with clear errors
- Schema discovery from files
- Schema version matching

**Error Messages**
- Clear, actionable error messages
- Suggest fixes for common mistakes
- Localization support

## Test File Organization

Create new test files in `src/Tests/`:
1. `SdkServicesTests.cs` — ContentRegistrationService, FileDiscoveryService, YamlLoader, UniverseLoader
2. `PackCompilerServicesTests.cs` — AssetValidationService, AssetImportService, etc.
3. `BridgeProtocolEdgeCasesTests.cs` — ScreenshotResult, message type edge cases
4. `ScenarioContentLoaderFixTests.cs` — Fix 4 failing tests
5. `ScenarioDomainTests.cs` — Additional scenario coverage
6. `ValidationSchemaTests.cs` — Validation and schema edge cases

## Implementation Rules

1. **Use FluentAssertions** for all assertions
2. **Test fixture pattern** for creating test data (AutoFixture where appropriate)
3. **BDD naming**: `Should_ExpectBehavior_WhenCondition`
4. **One assertion per test** OR group related assertions with `.And`
5. **No hardcoded paths** — use temp directories for file tests
6. **Clean up temp files** in test teardown
7. **Document why** each test matters (XML comments)
8. **Mock external dependencies** (file I/O, serialization)

## Success Criteria

- All new tests pass
- Total test count: 1348+ (100+ new tests)
- All existing tests still pass
- No failing tests in final run
- Coverage improvements visible in gap areas

## Notes

- Scenario domain tests are currently failing due to YAML parsing issue — diagnose and fix first
- Asset pipeline tests require careful test data setup (GLB/FBX files or mocks)
- Bridge tests should verify JSON serialization with round-trip assertions
- All tests must run in CI environment (no game process required)
