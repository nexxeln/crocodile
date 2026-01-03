# Crocodile Project Documentation Index

Complete guide to all documentation files for the Crocodile AI agent orchestrator project.

---

## Quick Navigation

| Document | Purpose | Best For |
|----------|---------|----------|
| **RUST_CLI_BEST_PRACTICES.md** | Comprehensive guide to modern Rust CLI patterns | Learning architecture, understanding patterns |
| **CLI_QUICK_REFERENCE.md** | Fast lookup for code snippets and patterns | Copy-paste during implementation |
| **IMPLEMENTATION.md** | Crocodile-specific implementation plan | Understanding project goals and phases |
| **CLAUDE.md** | Agent guidelines for code quality | Code review, quality standards |

---

## Document Descriptions

### 1. RUST_CLI_BEST_PRACTICES.md (27KB, 1158 lines)

**Comprehensive guide to building production-quality Rust CLI applications.**

#### Sections:
- **CLI Parsing with `clap`** - Modern derive API patterns, subcommand structure, global options
- **Project Structure** - Recommended folder layouts for multi-command CLIs
- **Error Handling** - `thiserror` + `anyhow` patterns with real examples
- **Environment Variables** - `dotenvy` setup with sensible defaults
- **Structured Logging** - `tracing` initialization and usage patterns
- **Progress Reporting** - `indicatif` for progress bars and spinners
- **Command Registration & Routing** - Trait-based and enum-based routing patterns
- **Complete Example** - Full Crocodile CLI implementation with all components
- **Testing** - Unit and integration test patterns
- **References** - Links to authoritative sources

#### When to Use:
- Learning how to structure a Rust CLI
- Understanding error handling patterns
- Setting up logging and configuration
- Implementing progress reporting
- Writing tests for CLI commands

#### Key Takeaways:
1. Use `clap` derive API (not builder pattern)
2. Struct-based App with enum-based Commands
3. Separate modules for CLI, commands, engine, errors
4. `thiserror` for custom errors + `anyhow` for ergonomics
5. `dotenvy` for configuration with sensible defaults
6. `tracing` for structured logging
7. `indicatif` for progress feedback

---

### 2. CLI_QUICK_REFERENCE.md (5KB, 527 lines)

**Fast lookup guide with copy-paste code snippets.**

#### Sections:
- **Clap Derive Patterns** - App, GlobalOpts, Commands, Args structures
- **Error Handling Patterns** - Custom errors, usage, quick returns
- **Logging Patterns** - Initialization, messages, structured fields
- **Configuration Patterns** - Loading from environment, .env.example
- **Progress Patterns** - Progress bars, spinners, multi-progress
- **Project Structure** - Minimal and modular layouts
- **Templates** - Main.rs, lib.rs, command execution patterns
- **Testing Patterns** - Unit and integration tests
- **Cargo.toml Essentials** - Dependencies and versions
- **Common Attributes** - Clap attributes reference
- **Common Macros** - Tracing macros reference
- **Environment Variables** - Standard env vars
- **Useful Commands** - Common cargo commands
- **Checklist** - Implementation checklist

#### When to Use:
- During implementation when you need a code snippet
- Quick reference for attribute names and syntax
- Checking which dependencies to add
- Verifying command patterns
- Running tests and builds

#### Format:
- Copy-paste ready code blocks
- Minimal explanation (see RUST_CLI_BEST_PRACTICES.md for details)
- Tables for quick lookups
- Checklists for implementation

---

### 3. IMPLEMENTATION.md (29KB, 646 lines)

**Crocodile-specific implementation plan and architecture.**

#### Sections:
- **Project Overview** - Goals, inspiration, key concepts
- **Architecture** - System design, components, data flow
- **Roles** - Planner, Foreman, Workers, Reviewer
- **Workflow** - Human-in-the-loop process
- **Storage** - JSONL + SQLite design
- **Phases** - Phase 1-4 implementation roadmap
- **API Design** - CrocEngine interface
- **TUI Dashboard** - Monitoring interface
- **Data Schemas** - JSONL formats

#### When to Use:
- Understanding project goals and vision
- Learning about the architecture
- Understanding the workflow and roles
- Planning implementation phases
- Designing data structures

#### Key Concepts:
- Human-in-the-loop AI orchestration
- Explicit planning phase with human approval
- Double review gate (AI + human)
- JSONL for append-only event log
- SQLite for efficient querying
- TUI dashboard for monitoring

---

### 4. CLAUDE.md (11KB, 217 lines)

**Agent guidelines for maintaining high-quality Rust code.**

#### Sections:
- **Core Principles** - Code optimization requirements
- **Preferred Tools** - Recommended crates and patterns
- **Code Style** - Formatting, naming, conventions
- **Documentation** - Doc comments, examples
- **Type System** - Leveraging Rust's type system
- **Error Handling** - Result types, custom errors
- **Function Design** - Single responsibility, parameters
- **Testing** - Unit tests, mocking, patterns
- **Imports** - Organization, avoiding wildcards
- **Memory & Performance** - Allocations, efficiency
- **Concurrency** - Send, Sync, tokio, rayon
- **Security** - Secrets, environment variables
- **Version Control** - Commit messages, no debug code
- **Tools** - rustfmt, clippy, cargo
- **Before Committing** - Checklist

#### When to Use:
- Code review and quality assurance
- Before committing changes
- When writing new functions or modules
- Ensuring consistency across codebase
- Optimizing code for performance

#### Key Standards:
- All code must be fully optimized
- No `.unwrap()` in production code
- Use `thiserror` for custom errors
- Use `tracing` for logging
- Use `tokio` for async
- Use `rayon` for parallelism
- No hardcoded credentials
- Comprehensive doc comments

---

## How to Use These Documents

### For Learning the Architecture
1. Start with **IMPLEMENTATION.md** to understand the project
2. Read **RUST_CLI_BEST_PRACTICES.md** sections on project structure
3. Review the "Complete Example: Crocodile CLI" section

### For Implementation
1. Use **CLI_QUICK_REFERENCE.md** for code snippets
2. Refer to **RUST_CLI_BEST_PRACTICES.md** for detailed explanations
3. Check **CLAUDE.md** before committing code
4. Follow the implementation checklist in **CLI_QUICK_REFERENCE.md**

### For Code Review
1. Check against **CLAUDE.md** guidelines
2. Verify patterns match **RUST_CLI_BEST_PRACTICES.md**
3. Ensure code follows **CLI_QUICK_REFERENCE.md** patterns

### For Debugging
1. Check error handling patterns in **RUST_CLI_BEST_PRACTICES.md**
2. Review logging setup in **CLI_QUICK_REFERENCE.md**
3. Verify configuration loading in **RUST_CLI_BEST_PRACTICES.md**

---

## Key Patterns Summary

### CLI Structure
```
App (struct)
├── GlobalOpts (flatten)
└── Command (enum)
    ├── Init(InitArgs)
    ├── Plan(PlanArgs)
    └── Prime(PrimeArgs)
```

### Module Organization
```
src/
├── main.rs (minimal)
├── lib.rs (exports)
├── cli/ (argument parsing)
├── commands/ (implementations)
├── engine/ (business logic)
├── error.rs (custom errors)
├── logging.rs (tracing setup)
└── config.rs (configuration)
```

### Error Handling
```
thiserror (define) → anyhow (use) → color_eyre (display)
```

### Configuration
```
.env file → dotenvy → env vars → sensible defaults
```

### Logging
```
tracing (emit) → tracing-subscriber (collect) → RUST_LOG (filter)
```

### Progress
```
indicatif (progress bars) ← tracing-indicatif (automatic)
```

---

## Implementation Checklist

### Phase 1: Core Infrastructure
- [ ] Set up project structure (cli/, commands/, engine/)
- [ ] Implement `clap` argument parsing
- [ ] Define custom error types with `thiserror`
- [ ] Set up `tracing` logging
- [ ] Implement `dotenvy` configuration loading
- [ ] Create `croc init` command
- [ ] Create `croc plan` command
- [ ] Create `croc prime` command
- [ ] Write unit tests for each command
- [ ] Write integration tests
- [ ] Add doc comments to all public items

### Phase 2: Engine Implementation
- [ ] Implement CrocEngine struct
- [ ] Design JSONL storage format
- [ ] Implement SQLite integration
- [ ] Create assignment tracking
- [ ] Implement context management

### Phase 3: TUI Dashboard
- [ ] Design dashboard layout
- [ ] Implement with `ratatui`
- [ ] Add real-time updates
- [ ] Implement command execution

### Phase 4: Advanced Features
- [ ] Add nested subcommands
- [ ] Implement configuration files
- [ ] Add shell completions
- [ ] Performance optimization

---

## Dependencies Reference

### Core CLI
- `clap` - Argument parsing (derive API)
- `anyhow` - Error handling
- `thiserror` - Custom error types
- `color-eyre` - Pretty error output

### Configuration
- `dotenvy` - Load .env files
- `serde` - Serialization
- `serde_json` - JSON handling

### Logging & Progress
- `tracing` - Structured logging
- `tracing-subscriber` - Logging setup
- `tracing-indicatif` - Progress integration
- `indicatif` - Progress bars

### Async Runtime
- `tokio` - Async runtime

### Utilities
- `dirs` - Home directory
- `camino` - UTF-8 paths

---

## References

### Official Documentation
- [Clap v4 Derive API](https://docs.rs/clap/latest/clap/)
- [Tracing Framework](https://docs.rs/tracing/latest/tracing/)
- [Indicatif Progress Bars](https://docs.rs/indicatif/latest/indicatif/)
- [Anyhow Error Handling](https://docs.rs/anyhow/latest/anyhow/)
- [Thiserror Custom Errors](https://docs.rs/thiserror/latest/thiserror/)

### Authoritative Guides
- [Rain's Rust CLI Recommendations](https://rust-cli-recommendations.sunshowers.io/)
- [Kevin K's CLI Structure Series](https://kbknapp.dev/cli-structure-01/)
- [Comprehensive Rust - Error Handling](https://comprehensive-rust.pages.dev/error-handling/thiserror-and-anyhow.html)

### Real-World Examples
- cargo-shuttle - Multi-command CLI
- delta - Diff tool with modular structure
- probe-rs - Embedded debugging tool

---

## Document Maintenance

### When to Update
- New Rust edition released
- Major dependency version changes
- New best practices discovered
- Implementation changes

### How to Update
1. Update relevant sections
2. Update examples and code snippets
3. Update references if needed
4. Update checklist if applicable
5. Commit with clear message

---

## Questions & Troubleshooting

### "How do I add a new command?"
1. Add variant to `Command` enum in `src/cli/mod.rs`
2. Create `src/commands/newcmd.rs` with `pub async fn exec()`
3. Add match arm in `App::execute()`
4. Write tests in `src/commands/newcmd.rs`
5. See **CLI_QUICK_REFERENCE.md** for code snippets

### "How do I add a global option?"
1. Add field to `GlobalOpts` struct in `src/cli/mod.rs`
2. Use `#[clap(global = true)]` attribute
3. Access via `app.global.option_name`
4. See **RUST_CLI_BEST_PRACTICES.md** for examples

### "How do I log something?"
1. Use `tracing::info!()`, `debug!()`, `warn!()`, `error!()`
2. Add structured fields: `info!(key = value, "message")`
3. Control with `RUST_LOG` env var
4. See **CLI_QUICK_REFERENCE.md** for examples

### "How do I handle errors?"
1. Define custom error in `src/error.rs` with `thiserror`
2. Return `anyhow::Result<T>` from functions
3. Use `.context()` to add information
4. See **RUST_CLI_BEST_PRACTICES.md** for patterns

---

## Document Statistics

| Document | Size | Lines | Purpose |
|----------|------|-------|---------|
| RUST_CLI_BEST_PRACTICES.md | 27KB | 1158 | Comprehensive guide |
| CLI_QUICK_REFERENCE.md | 9.2KB | 527 | Quick lookup |
| IMPLEMENTATION.md | 29KB | 646 | Project plan |
| CLAUDE.md | 11KB | 217 | Code guidelines |
| **Total** | **76KB** | **2548** | **Complete documentation** |

---

## Next Steps

1. **Read** IMPLEMENTATION.md to understand the project
2. **Review** RUST_CLI_BEST_PRACTICES.md for architecture
3. **Use** CLI_QUICK_REFERENCE.md during implementation
4. **Follow** CLAUDE.md for code quality
5. **Implement** Phase 1 using the provided patterns
6. **Test** each command thoroughly
7. **Document** your code with doc comments
8. **Review** against guidelines before committing

---

**Last Updated**: January 3, 2026
**Status**: Complete - Ready for implementation
**Next Phase**: Phase 1 Core Infrastructure
