# OpenCode.md - Ordis Project Guidelines

## Build Commands
- Build: `cargo build`
- Run: `cargo run`
- Test all: `cargo test`
- Test single: `cargo test test_name`
- Lint: `cargo clippy`
- Format: `cargo fmt`

## Code Style
- Use 4-space indentation
- Follow Rust naming conventions: snake_case for variables/functions, CamelCase for types/structs
- Organize imports alphabetically, group by std/external/internal
- Prefer Result<T, Error> for error handling with ? operator
- Use descriptive error messages in Err() variants
- Document public functions with /// comments
- Use strong typing with enums for state representation
- Prefer immutable variables (let vs let mut)
- Use async/await for asynchronous operations
- Follow the existing module structure for new features