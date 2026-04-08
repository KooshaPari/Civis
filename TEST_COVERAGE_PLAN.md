# Test Coverage Gap Filling Plan

## Priority 1: EconomyBalanceCalculator (44.4% branch coverage - CRITICAL)

### File: src/Domains/Economy/Balance/EconomyBalanceCalculator.cs
### Test File: src/Tests/EconomyCoverageGapTests.cs (NEW)

#### Uncovered Branch Areas:
1. **Line 100-103**: prodRate <= 0 warning condition
   - Test: Zero production rate generates warning
   - Test: Negative production rate (edge case clamping)
   
2. **Line 104-107**: balance < 0 deficit warning condition
   - Test: Negative balance generates deficit warning
   - Test: Zero balance (boundary)
   
3. **Line 214**: EffectiveRate > 0 check in trade efficiency
   - Test: Zero exchange rate handling
   - Test: Negative exchange rate edge case
   
4. **Line 217-221**: sourceBalance > 0 surplus check
   - Test: Zero source balance (no surplus)
   - Test: Negative source balance
   
5. **Line 246**: average <= 0 in overall balance
   - Test: All zero sustainability scores
   - Test: Mixed positive/negative scores

#### Test Cases to Create (6 total):
1. `GenerateReport_ZeroProductionRates_GeneratesWarning`
2. `GenerateReport_NegativeRatesInProduction_HandlesGracefully`
3. `GenerateReport_DeficitDetection_GeneratesCorrectWarning`
4. `GenerateReport_ZeroExchangeRate_NoTradeEfficiency`
5. `GenerateReport_NegativeSustainabilityScores_CalculatesOverallBalance`
6. `GenerateReport_MaxIntResourceRates_NoOverflow`

---

## Priority 2: HUDInjectionSystem + HUDElementDefinition (50% branch - CRITICAL)

### File: src/Domains/UI/HUDInjectionSystem.cs + HUDElementDefinition
### Test File: src/Tests/UIDomainCoverageGapTests.cs (NEW)

#### Uncovered Branch Areas:
1. **Line 31-44**: Initialize() / not initialized check
   - Test: RegisterElement before Initialize throws
   - Test: Initialize called multiple times clears properly
   
2. **Line 56-60**: UnregisterElement removal logic
   - Test: Element not found returns false
   - Test: Case-insensitive ID matching in UnregisterElement
   
3. **Line 84-89**: Shutdown() behavior
   - Test: Shutdown properly clears elements
   - Test: Operations after shutdown fail appropriately

#### Test Cases to Create (5 total):
1. `RegisterElement_BeforeInitialization_ThrowsInvalidOperationException`
2. `RegisterElement_NullElement_ThrowsArgumentNullException`
3. `UnregisterElement_NotFound_ReturnsFalse`
4. `UnregisterElement_CaseInsensitiveId_RemovesCorrectly`
5. `Shutdown_ClearsElementsAndDisablesSystem`

---

## Priority 3: FactionArchetype Validation (50% branch - CRITICAL)

### File: src/Domains/Warfare/Archetypes/FactionArchetype.cs
### Test File: src/Tests/WarfareCoverageGapTests.cs (NEW)

#### Uncovered Branch Areas:
1. **Line 40-45**: Constructor null checks
   - Test: Null ID throws
   - Test: Null DisplayName throws
   - Test: Null Description throws
   - Test: Empty/whitespace values (boundary)

2. **Line 45**: BaseModifiers initialization
   - Test: Case-insensitive modifier lookups
   - Test: Immutable dictionary behavior

#### Test Cases to Create (5 total):
1. `FactionArchetype_NullId_ThrowsArgumentNullException`
2. `FactionArchetype_NullDisplayName_ThrowsArgumentNullException`
3. `FactionArchetype_NullDescription_ThrowsArgumentNullException`
4. `FactionArchetype_CaseInsensitiveModifiers_Works`
5. `FactionArchetype_BaseModifiers_IsImmutable`

---

## Priority 4: SchemaResolverService / Validation (50% branch)

### File: src/SDK/Validation/NJsonSchemaValidator.cs (closest to SchemaResolver)
### Test File: src/Tests/SDKValidationCoverageGapTests.cs (NEW)

#### Uncovered Branch Areas:
1. Schema not found error handling
2. Invalid schema reference (circular, missing)
3. Fallback schema resolution
4. Validation error accumulation

#### Test Cases to Create (4 total):
1. `ValidateContentPack_SchemaNotFound_ReturnsError`
2. `ValidateContentPack_InvalidReference_ReturnsError`
3. `ValidateContentPack_MultipleErrors_AccumulatesAll`
4. `ValidateContentPack_EmptyContent_HandlesGracefully`

---

## Test Implementation Details

### Dependencies & Mocking Strategy:
- **EconomyBalanceCalculator**: Mock ProductionCalculator and TradeEngine using Moq
- **HUDInjectionSystem**: No dependencies, direct instantiation
- **FactionArchetype**: No dependencies, direct instantiation
- **NJsonSchemaValidator**: Use real schema files from schemas/ or mock minimal schema

### Assertion Patterns:
- Use FluentAssertions throughout: `.Should().Be()`, `.Should().Throw<>()`
- Test both happy path AND error paths
- Validate boundary conditions (0, negative, max int)

### Test Organization:
- Each test file: class per component under test
- Naming: `[ComponentName]_[Condition]_[Expected]`
- Use xUnit `[Fact]` and `[Theory]` attributes
- Mark game-dependent tests with `[Trait("Category", "Integration")]` if needed

---

## Files to Create/Modify:

1. **NEW**: `src/Tests/EconomyCoverageGapTests.cs` (6 tests)
2. **NEW**: `src/Tests/UIDomainCoverageGapTests.cs` (5 tests)
3. **NEW**: `src/Tests/WarfareCoverageGapTests.cs` (5 tests)
4. **NEW**: `src/Tests/SDKValidationCoverageGapTests.cs` (4 tests)

**Total New Tests: 20**
**Expected Coverage Improvement: +15-20% across target areas**

---

## Execution Steps:

1. Create EconomyCoverageGapTests.cs with ProductionCalculator/TradeEngine mocks
2. Create UIDomainCoverageGapTests.cs with direct instantiation tests
3. Create WarfareCoverageGapTests.cs with null/boundary tests
4. Create SDKValidationCoverageGapTests.cs with schema validation tests
5. Run: `dotnet test src/Tests/DINOForge.Tests.sln --verbosity normal`
6. Verify all tests pass
7. Report coverage improvements

