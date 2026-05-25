# Testing Contract

## Test Coverage

- All public API methods must have unit tests
- All schemas must have valid and invalid test cases
- Warfare domain must have balance validation tests
- Target coverage: 80%+ code coverage

## Test Execution

- All tests must pass before considering work complete
- Run `dotnet test src/DINOForge.sln --verbosity normal` before committing
- Integration tests must validate against realistic pack/ECS scenarios

## Test Types

- **Unit Tests**: Individual methods and components (xUnit)
- **Schema Tests**: YAML/JSON validation against all schemas
- **Pack Validation Tests**: Manifest structure, ID uniqueness, reference integrity
- **Balance Tests**: Warfare domain stat ranges and doctrine consistency
- **Integration Tests**: SDK loader, registry binding, ECS bridge functionality

## Test Organization

- Unit tests live in `src/Tests/` alongside their source projects
- Fixtures and shared test data in `src/Tests/Fixtures/`
- Use FluentAssertions for readable assertions
- Use Moq for mocking external dependencies
