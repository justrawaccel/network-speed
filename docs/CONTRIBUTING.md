# Contributing to `network-speed`

Thanks for your interest in improving the project! We welcome bug reports, feature ideas, documentation tweaks, and pull requests. This guide explains how to get set up, what we expect in contributions, and how to collaborate efficiently.

---

## Table of contents

1. [Project goals](#project-goals)
2. [Ways to contribute](#ways-to-contribute)
3. [Before you start](#before-you-start)
4. [Setting up your environment](#setting-up-your-environment)
5. [Project structure](#project-structure)
6. [Development workflow](#development-workflow)
7. [Coding standards](#coding-standards)
8. [Testing & quality gates](#testing--quality-gates)
9. [Documentation](#documentation)
10. [Commit & PR checklist](#commit--pr-checklist)
11. [Community expectations](#community-expectations)
12. [Getting help](#getting-help)

---

## Project goals

`network-speed` is a Windows-focused Rust library that provides high-performance, ergonomic access to real-time network interface statistics. We aim to deliver:

- Safe, well-documented APIs for sync and async monitoring
- Lean dependencies with feature gates (`async`, `serde`, `cli`)
- Production-grade reliability backed by tests and examples
- Clear documentation for end users and integrators

When proposing changes, make sure they align with these goals and maintain or improve the library’s focus on performance and developer experience.

## Ways to contribute

- **Report bugs** via GitHub issues with reproduction steps and environment details.
- **Suggest improvements** for APIs, performance, or developer ergonomics.
- **Fix issues** by submitting pull requests that reference an existing ticket whenever possible.
- **Improve documentation** (README, examples, Rustdoc, guides).
- **Add tests** that increase coverage or capture regressions.

If you are unsure whether an idea fits the roadmap, open an issue or start a discussion before writing code.

## Before you start

1. **Read the [Code of Conduct](./CODE_OF_CONDUCT.md).** By participating you agree to follow it.
2. **Search existing issues/PRs** to avoid duplicates.
3. **Align with maintainers early** for large or breaking changes.

## Setting up your environment

- Install the latest stable Rust toolchain with [`rustup`](https://rustup.rs/).
- Windows 10/11 (x64) is the primary target platform. Most development runs fine on other OSes, but integration tests that hit Windows APIs will only succeed on Windows.
- Install the build essentials for your platform (Microsoft Build Tools or Visual Studio for MSVC targets).
- Clone the repository and fetch dependencies:

  ```bash
  git clone https://github.com/justrawaccel/network-speed.git
  cd network-speed
  cargo fetch
  ```

- Optional tooling:
  - `rustfmt` and `clippy` for formatting and linting (normally installed with the default toolchain).
  - `cargo-edit` for dependency management tweaks.

## Project structure

```
.
├── src/                # Library source
│   ├── monitor/        # Sync/async monitors and interface helpers
│   └── types/          # Config, error, and data types
├── examples/           # Runnable examples (sync & async monitors)
├── tests/              # Integration and doc-aligned tests
├── README.md           # User-facing overview & usage guide
├── CONTRIBUTING.md     # (this guide)
└── CODE_OF_CONDUCT.md  # Community expectations
```

## Development workflow

1. **Fork & branch**

   - Fork the repo and create a feature branch: `git checkout -b feature/my-improvement`.

2. **Implement your change**

   - Follow coding standards and keep changes scoped. Update documentation alongside code.

3. **Run checks locally**

   - `cargo fmt --all`
   - `cargo clippy --all-targets --all-features -- -D warnings`
   - `cargo test` (see [Testing & quality gates](#testing--quality-gates))

4. **Commit with context**

   - Use descriptive commit messages (e.g., `Add async helper for interface refresh`).
   - Reference related issues in the body where relevant.

5. **Open a pull request**

   - Provide a summary, testing evidence, and screenshots/logs if useful.
   - Template (if available) must be filled out completely.

6. **Collaborate**
   - Respond to review feedback quickly and keep conversations respectful.
   - Squash or rebase as requested by maintainers before merging.

## Coding standards

- **Style:** The project follows `rustfmt` defaults. Format all code before committing.
- **Linting:** Ensure `cargo clippy --all-targets --all-features` passes without warnings.
- **Features:** Keep optional features additive and carefully gate platform-specific code.
- **Errors:** Prefer expressive `Result<T, NetworkError>` returns and avoid panics in library code.
- **Testing:** Provide unit/integration tests for new functionality or bug fixes when practical.
- **Documentation:** Add or update Rustdoc comments for public APIs. Keep README and examples in sync with behavior changes.

## Testing & quality gates

Run the following commands locally before submitting a PR:

```bash
cargo fmt --all
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all-features
```

Notes:

- Some integration tests call Windows APIs; they are skipped or no-ops on non-Windows hosts.
- If you add new features behind flags, ensure tests cover both feature-enabled and disabled states when possible.
- Include benchmarks or performance notes for changes that impact runtime characteristics.

## Documentation

- Update `README.md`, examples, and inline docs to reflect behavioral changes.
- Add new examples under `examples/` when introducing notable user-facing features.
- Keep changelog entries (if present) accurate, or include release notes in the PR description.

## Commit & PR checklist

Before requesting a review, double-check:

- [ ] Code is formatted (`cargo fmt`).
- [ ] Clippy passes with warnings treated as errors.
- [ ] Tests succeed locally (note any platform skips).
- [ ] Public APIs include Rustdoc coverage.
- [ ] README/examples/docs are updated.
- [ ] Commits are descriptive; large changes are broken into logical commits.
- [ ] PR description explains motivation, approach, and test coverage.

## Community expectations

We are committed to providing a welcoming and inclusive environment. Treat everyone with respect, acknowledge differing viewpoints, and prefer constructive feedback over dismissive comments. Harassment or discrimination will not be tolerated—see the [Code of Conduct](./CODE_OF_CONDUCT.md) for details.

## Getting help

- **Questions / ideas:** Open a discussion or issue on GitHub.
- **Security concerns:** Email the maintainers at [`security@justrawaccel.dev`](mailto:security@justrawaccel.dev).
- **Private contact:** Reach the maintainer at [`x@justrawaccel.dev`](mailto:x@justrawaccel.dev).

Thank you for helping make `network-speed` better!
