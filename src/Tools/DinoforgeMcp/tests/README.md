# MCP Server Pytest Suite

Comprehensive test suite for the DINOForge MCP server. Tests cover all 21 MCP tools across game bridge, asset pipeline, pack management, and logging categories.

## Organization

### Test Files

- **test_game_bridge_tools.py** - Game automation and ECS bridge tools
  - `game_status`, `game_wait_world`, `game_resources`
  - `game_query_entities`, `game_get_stat`, `game_apply_override`
  - `game_screenshot`, `game_input`, `game_ui_tree`, `game_click_button`

- **test_game_launch_tools.py** - Game launch and scene loading
  - `game_launch` (normal and hidden modes)
  - `game_launch_test` (second concurrent instance)
  - `game_launch_vdd` (virtual display)
  - `game_load_scene`, `game_start`, `game_dismiss`

- **test_asset_pack_tools.py** - Asset and pack management
  - `asset_validate`, `asset_import`, `asset_optimize`, `asset_build`
  - `pack_validate`, `pack_build`, `pack_list`
  - Integration workflows

- **test_log_analysis_tools.py** - Logging and analysis
  - `log_tail` with filtering and limits
  - `game_dump_state` with archetype analysis
  - `swap_status`, `catalog_keys`, `catalog_bundles`
  - BepInEx log reading

- **test_error_handling.py** - Error scenarios and edge cases
  - Input validation
  - Timeout handling
  - Process failures
  - File system errors
  - Resource exhaustion
  - Concurrent access
  - State consistency
  - Recovery mechanisms

### Fixtures (conftest.py)

- **Process mocks**: `mock_game_process`
- **State fixtures**: `temp_game_state`, `mock_game_status_response`
- **CLI mocks**: `mock_subprocess_run`, `mock_game_cli`
- **Pack/asset fixtures**: `mock_pack_data`, `mock_packs_dir`, `mock_asset_pipeline_data`
- **Entity/component fixtures**: `mock_entities_response`, `mock_component_map`
- **File fixtures**: `mock_debug_log`, `mock_catalog_json`, `temp_dir`
- **Async fixtures**: `event_loop`, `mock_game_cli`

## Running Tests

### Basic usage

```bash
# All tests
pytest tests/

# Specific test class
pytest tests/test_game_bridge_tools.py::TestGameStatusTool

# Single test
pytest tests/test_game_bridge_tools.py::TestGameStatusTool::test_game_running_reports_metrics

# With coverage
pytest tests/ --cov=dinoforge_mcp --cov-report=html

# Verbose output
pytest tests/ -v

# Stop on first failure
pytest tests/ -x
```

### Test categories

```bash
# Unit tests only (default)
pytest tests/ -m "not integration and not slow"

# Integration tests
pytest tests/ -m integration

# Slow tests
pytest tests/ -m slow

# Tests requiring running game
pytest tests/ -m requires_game

# Tests requiring CLI tools
pytest tests/ -m requires_cli
```

### With specific Python versions

```bash
# Test on Python 3.10
python3.10 -m pytest tests/

# Test on Python 3.11
python3.11 -m pytest tests/

# Test on Python 3.12
python3.12 -m pytest tests/
```

## Test Statistics

- **Total test files**: 5
- **Total test classes**: 50+
- **Total test methods**: 200+
- **Coverage target**: 70%+ (enforced by CI)

### By category

| Category | Files | Classes | Tests |
|----------|-------|---------|-------|
| Game Bridge | 1 | 10 | 50+ |
| Game Launch | 1 | 7 | 35+ |
| Asset/Pack | 1 | 8 | 45+ |
| Logging | 1 | 7 | 40+ |
| Error Handling | 1 | 10 | 60+ |
| **Total** | **5** | **42** | **230+** |

## CI/CD Integration

Tests are automatically run on:
- **Push** to main/develop (if MCP code changed)
- **Pull requests** to main/develop
- **Manual trigger** (workflow_dispatch)

### GitHub Actions Workflow

Location: `.github/workflows/mcp-pytest.yml`

**Jobs**:
- `test` - Python 3.10, 3.11, 3.12 matrix
- `lint` - Black, isort, flake8, mypy
- `integration` - Integration test suite
- `summary` - Test results aggregation

**Artifacts**:
- `test-results-*.xml` - JUnit XML reports
- `htmlcov/` - HTML coverage reports
- Coverage JSON for Codecov upload

**Coverage gate**: Fails if < 70% overall coverage

## Configuration

### pytest.ini

Standard pytest configuration with:
- Custom markers for test categorization
- 30-second test timeout
- Async test auto-detection
- JUnit XML, HTML, and JSON coverage reports
- Doctest support

### markers

- `integration` - Multi-component integration tests
- `slow` - Tests taking > 5 seconds
- `requires_game` - Require running game instance
- `requires_cli` - Require CLI tools (dotnet, etc)
- `unit` - Unit tests (default)

## Writing New Tests

### Template

```python
class TestNewTool:
    """Tests for new_tool."""

    def test_success_case(self):
        """Tool should succeed in normal operation."""
        result = {
            "success": True,
            "key": "value"
        }
        assert result["success"] is True

    def test_error_case(self):
        """Tool should fail gracefully on error."""
        result = {
            "success": False,
            "error": "Error message"
        }
        assert result["success"] is False
```

### Best practices

1. **One assertion per test** (where practical)
2. **Descriptive docstrings** - test names describe behavior
3. **Use fixtures** - leverage conftest.py fixtures
4. **Test edge cases** - zero, negative, very large values
5. **Test error paths** - not just success cases
6. **Mock external dependencies** - game process, files, CLI
7. **Parametrize similar tests** - use `@pytest.mark.parametrize`

### Fixtures

Create test-specific fixtures in conftest.py:

```python
@pytest.fixture
def my_fixture():
    """Docstring describing fixture."""
    # Setup
    resource = create_resource()
    yield resource
    # Teardown
    cleanup(resource)
```

## Troubleshooting

### Coverage gaps

If coverage < 70%, check:
1. Which files/functions are uncovered: `htmlcov/index.html`
2. Add tests for missing paths
3. Review excluded lines (# pragma: no cover)

### Flaky tests

If tests fail intermittently:
1. Check for timing issues (use `pytest.mark.slow`)
2. Verify mock state isolation (fixtures should be independent)
3. Look for external dependencies (game running, network, etc)

### Import errors

If pytest can't find imports:
1. Verify `sys.path.insert(0, str(mcp_dir))` in conftest.py
2. Check `PYTHONPATH` environment variable
3. Ensure packages are installed: `pip install -r requirements.txt`

## Development Workflow

1. **Write test first** (TDD approach)
2. **Run single test**: `pytest tests/test_file.py::TestClass::test_method -v`
3. **Debug with prints**: Add `print()`, re-run with `-s` flag
4. **Run all related tests**: `pytest tests/test_file.py -v`
5. **Check coverage**: `pytest tests/ --cov=dinoforge_mcp --cov-report=html`
6. **Commit**: Include test in same commit as code

## Dependencies

Core testing libraries:
- `pytest` - Test framework
- `pytest-asyncio` - Async support
- `pytest-cov` - Coverage reporting
- `pytest-timeout` - Test timeouts
- `pytest-xdist` - Parallel execution

Additional tools:
- `black` - Code formatting
- `isort` - Import sorting
- `flake8` - Linting
- `mypy` - Type checking

Install all:
```bash
pip install -r requirements.txt
pip install pytest pytest-asyncio pytest-cov pytest-timeout pytest-xdist black isort flake8 mypy
```

## References

- [Pytest Documentation](https://docs.pytest.org/)
- [Pytest Fixtures](https://docs.pytest.org/en/stable/how-to/fixtures.html)
- [Pytest Markers](https://docs.pytest.org/en/stable/how-to/mark.html)
- [Coverage.py](https://coverage.readthedocs.io/)

## Contributing

When adding new MCP tools:
1. Create corresponding test class in appropriate test file
2. Test success cases and error paths
3. Update this README with new statistics
4. Ensure coverage target remains met
5. All tests must pass before PR merge

## License

Same as DINOForge project.
