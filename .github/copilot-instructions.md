# Copilot instructions for this repo (Topgrade)# Copilot instructions for this repo (Topgrade)# Copilot instructions for this repo (Topgrade)



Use these notes to work effectively in this codebase; follow existing patterns and keep edits minimal.



## Big pictureUse these notes to work effectively in this codebase; follow existing patterns and keep edits minimal.Use these notes to work effectively in this codebase; follow existing patterns and keep edits minimal.



- Cross-platform CLI that runs many upgrade "steps" (package managers, tools).



- Entry: `src/main.rs` → loads config (`src/config.rs`), sets logging/i18n, builds an `ExecutionContext`, then runs `Step`s via `Runner`.## Big picture## Big picture



- Steps: `src/step.rs` defines the `Step` enum and `default_steps()`; each variant calls helpers in `src/steps/**` (e.g. `generic.rs`, `os/windows.rs`).



- Execution: always use `ExecutionContext::execute()` → `Executor` (Dry/Wet). Dry prints commands for `--dry-run`; Wet executes. Prefer `CommandExt` helpers: `.status_checked()`, `.output_checked_utf8()`.- Cross-platform CLI that runs many upgrade "steps" (package managers, tools).- Cross-platform CLI that runs many upgrade "steps" (package managers, tools).



- Reporting/retry: `Runner` records results in `Report` and prompts on failure unless `--no-retry` or step listed in `ignore_failures`.



## Config and conventions- Entry: `src/main.rs` → loads config (`src/config.rs`), sets logging/i18n, builds an `ExecutionContext`, then runs `Step`s via `Runner`.- Entry: `src/main.rs` → loads config (`src/config.rs`), sets logging/i18n, builds an `ExecutionContext`, then runs `Step`s via `Runner`.



- Config precedence:



  1) CLI (`CommandLineArgs` in `src/config.rs`),- Steps: `src/step.rs` defines the `Step` enum and `default_steps()`; each variant calls helpers in `src/steps/**` (e.g. `generic.rs`, `os/windows.rs`).- Steps: `src/step.rs` defines the `Step` enum and `default_steps()`; each variant calls helpers in `src/steps/**` (e.g. `generic.rs`, `os/windows.rs`).



  2) File: `CONFIG_DIR/topgrade.toml` or `CONFIG_DIR/topgrade/topgrade.toml`,



  3) Directory includes: `CONFIG_DIR/topgrade.d/*.toml` and inline `[include]` blocks (processed in-order).- Execution: always use `ExecutionContext::execute()` → `Executor` (Dry/Wet). Dry prints commands for `--dry-run`; Wet executes. Prefer `CommandExt` helpers: `.status_checked()`, `.output_checked_utf8()`.- Execution: always use `ExecutionContext::execute()` → `Executor` (Dry/Wet). Dry prints commands for `--dry-run`; Wet executes. Prefer `CommandExt` helpers: `.status_checked()`, `.output_checked_utf8()`.



  First run creates a default config from `config.example.toml`.



- Use `Config` getters (e.g. `yes(step)`, `should_run(step)`, `cleanup()`, `git_concurrency_limit()`, etc.). Don't parse TOML/env directly.- Reporting/retry: `Runner` records results in `Report` and prompts on failure unless `--no-retry` or step listed in `ignore_failures`.- Reporting/retry: `Runner` records results in `Report` and prompts on failure unless `--no-retry` or step listed in `ignore_failures`.



- Use i18n for user-facing strings: `rust_i18n::t!(...)` (initialized in `main`, locales in `locales/app.yml`).



- Common utilities:## Config and conventions



  - `utils::require("bin")` / `require_one([...])`: ensure tools exist or return `SkipStep`.



  - `ctx.require_sudo()` and `SudoExecuteOpts` for elevation.- Config precedence:## Config and conventions



  - `terminal::print_separator`, `print_warning` for consistent UX.



  - For "not applicable", return `Err(SkipStep(...).into())` instead of failing hard.  1) CLI (`CommandLineArgs` in `src/config.rs`),- Config precedence:



- Logging: `tracing`; respect `--log-filter` and `-v`. Prefer `debug!` for diagnostics.



## Temporary artifact files and local-only documentation  2) File: `CONFIG_DIR/topgrade.toml` or `CONFIG_DIR/topgrade/topgrade.toml`,  1) CLI (`CommandLineArgs` in `src/config.rs`),



**DO NOT COMMIT** temporary analysis files or local validation artifacts to the repository. Keep them locally only.



### What to keep locally only (use `.gitignore` or manual cleanup):  3) Directory includes: `CONFIG_DIR/topgrade.d/*.toml` and inline `[include]` blocks (processed in-order).  2) File: `CONFIG_DIR/topgrade.toml` or `CONFIG_DIR/topgrade/topgrade.toml`,



- **Temporary markdown files** (`.md`): Analysis documents, meeting notes, scratch files, decision matrices, workflow analysis files

  - Examples: `PR_ANALYSIS.md`, `DECISION_MATRIX.md`, `*_ANALYSIS.md`, `START_HERE_*.md`

  First run creates a default config from `config.example.toml`.  3) Directory includes: `CONFIG_DIR/topgrade.d/*.toml` and inline `[include]` blocks (processed in-order).

- **Validation/test reports** (`.txt`, `.log`): Local validation summaries, test results, verification reports

  - Examples: `VALIDATION_SUMMARY.txt`, `test_results.log`, `PR_*_REVIEW.md`



- **Temporary workspace files**: IDE-specific files, cache directories, build artifacts, stash files- Use `Config` getters (e.g. `yes(step)`, `should_run(step)`, `cleanup()`, `git_concurrency_limit()`, etc.). Don't parse TOML/env directly.  First run creates a default config from `config.example.toml`.



### What to commit:



- **Official documentation** (`.md`): README, CONTRIBUTING.md, BREAKING_CHANGES.md, guides- Use i18n for user-facing strings: `rust_i18n::t!(...)` (initialized in `main`, locales in `locales/app.yml`).- Use `Config` getters (e.g. `yes(step)`, `should_run(step)`, `cleanup()`, `git_concurrency_limit()`, etc.). Don't parse TOML/env directly.

- **Configuration files** (`.yaml`, `.toml`, `.json`, `.yml`): Actual project configuration

- **Source code**: All `.rs`, `.sh`, and application files



### Why this matters:- Common utilities:- Use i18n for user-facing strings: `rust_i18n::t!(...)` (initialized in `main`, locales in `locales/app.yml`).



- Keeps commit history clean and focused on actual code changes

- Prevents confusion between permanent documentation and temporary analysis

- Allows cleanup without committing/reverting artifacts  - `utils::require("bin")` / `require_one([...])`: ensure tools exist or return `SkipStep`.- Common utilities:

- Makes PRs easier to review (less noise, clearer intent)

- Reduces merge conflicts from abandoned analysis files



### Best practice workflow:  - `ctx.require_sudo()` and `SudoExecuteOpts` for elevation.  - `utils::require("bin")` / `require_one([...])`: ensure tools exist or return `SkipStep`.



1. **Create** temporary analysis files with descriptive names (e.g., `PR_1234_ANALYSIS.md`)

2. **Use** these files locally during development/validation/review

3. **Before pushing**: Run `git clean -fd` to remove untracked temporary files  - `terminal::print_separator`, `print_warning` for consistent UX.  - `ctx.require_sudo()` and `SudoExecuteOpts` for elevation.

4. **Commit** only permanent `.md` files (official documentation)

5. **Clean up** locally-only `.txt` and analysis files



Example from real work:  - For "not applicable", return `Err(SkipStep(...).into())` instead of failing hard.  - `terminal::print_separator`, `print_warning` for consistent UX.

```bash

# During validation session, create analysis file

$ echo "Validation notes..." > PR_1309_VALIDATION_NOTES.txt

- Logging: `tracing`; respect `--log-filter` and `-v`. Prefer `debug!` for diagnostics.  - For "not applicable", return `Err(SkipStep(...).into())` instead of failing hard.

# Use it to track your findings

# (do your work here)



# Before pushing, clean up## Temporary artifact files and local-only documentation- Logging: `tracing`; respect `--log-filter` and `-v`. Prefer `debug!` for diagnostics.

$ git clean -fd

$ rm PR_1309_VALIDATION_NOTES.txt



# Now push your actual code/doc changes**DO NOT COMMIT** temporary analysis files or local validation artifacts to the repository. Keep them locally only.## Temporary artifact files and local-only documentation

$ git push

```



## Error handling### What to keep locally only (use `.gitignore` or manual cleanup):Do not commit temporary analysis files or local validation artifacts to the repository:



- Use `color_eyre::Result<()>` as the return type for fallible operations. The `?` operator automatically converts errors to `color_eyre::Report`.



- Three main error scenarios:- **Temporary markdown files** (`.md`): Analysis documents, meeting notes, scratch files, decision matrices, workflow analysis files### What to keep locally only (use `.gitignore` or manual cleanup):



  - **Tool not found/not applicable**: Return `Err(SkipStep("reason").into())`. The `Runner` interprets this as a skipped step, not a failure.  - Examples: `PR_ANALYSIS.md`, `DECISION_MATRIX.md`, `*_ANALYSIS.md`, `START_HERE_*.md`



  - **Command failed**: Use `TopgradeError::ProcessFailed(cmd, status)` or `.status_checked()?` on commands via `CommandExt`.- **Temporary markdown files** (`.md`): Analysis documents, meeting notes, scratch files, decision matrices, workflow analysis files. Examples: `PR_ANALYSIS.md`, `DECISION_MATRIX.md`, `*_ANALYSIS.md`



  - **Configuration/environment issue**: Use `TopgradeError` variants or custom error messages. Always provide context with `context("action")` if using eyre.- **Validation/test reports** (`.txt`, `.log`): Local validation summaries, test results, verification reports



- Example pattern in a step:  - Examples: `VALIDATION_SUMMARY.txt`, `test_results.log`, `PR_*_REVIEW.md`- **Validation/test reports** (`.txt`, `.log`): Local validation summaries, test results, verification reports. Examples: `VALIDATION_SUMMARY.txt`, `test_results.log`

  ```rust

  pub fn run_xxx(ctx: &ExecutionContext) -> Result<()> {

      let xxx = require("xxx")?;  // Returns SkipStep if not found

      print_separator("xxx");- **Temporary workspace files**: IDE-specific files, cache directories, build artifacts, stash files- **Temporary workspace files**: IDE-specific files, cache directories, build artifacts

      ctx.execute(xxx).args([...]).status_checked()?  // Fails if exit status != 0

  }

  ```

### What to commit:### What to commit:

- Prefer early returns with `?` over nested match/if-let for cleaner code.



## Adding or modifying a step (example workflow)

- **Official documentation** (`.md`): README, CONTRIBUTING.md, BREAKING_CHANGES.md, guides- **Documentation** (`.md`): Official docs, guides, CONTRIBUTING.md, BREAKING CHANGES, README

- Add a variant to `Step` in `src/step.rs` (snake_case names are auto-wired by `clap/strum`).

- **Configuration files** (`.yaml`, `.toml`, `.json`, `.yml`): Actual project configuration- **Configuration files** (`.yaml`, `.toml`, `.json`, `.yml`): Actual project configuration

- Implement logic in `src/steps/<module>.rs` (or extend `generic.rs`):

- **Source code**: All `.rs`, `.sh`, and application files- **Source code**: All `.rs`, `.sh`, and application files

  - Guard with `require`/`require_one` for binaries.



  - Use `ctx.execute(...)` + `.args([..])` + `.status_checked()?`.

### Why this matters:### Why this matters:

  - Read options via `ctx.config()` (e.g. `yes(Step::YourStep)`).



  - Return `SkipStep` when the tool isn't present or not applicable.

- Keeps commit history clean and focused on actual code changes- Keeps commit history clean and focused on actual code changes

- Wire the variant in `Step::run()` and add to `default_steps()` behind appropriate `#[cfg(...)]`.

- Prevents confusion between permanent documentation and temporary analysis- Prevents confusion between permanent documentation and temporary analysis

## Common utility patterns

- Allows cleanup without committing/reverting artifacts- Allows cleanup without committing/reverting artifacts

- **PathExt trait** provides useful methods on any `AsRef<Path>`:

- Makes PRs easier to review (less noise, clearer intent)- Makes PRs easier to review (no noise from temporary files)

  - `.if_exists()`: Returns `Option<Self>`, useful for optional paths.

- Reduces merge conflicts from abandoned analysis files

  - `.require()`: Returns `Result<Self>` with `SkipStep` error if path doesn't exist.

### Best practice workflow:

  - `.is_descendant_of(ancestor)`: Checks if a path is under another.

### Best practice workflow:

- **CommandExt trait** extends `Command` with helpers:

1. Create temporary analysis files with descriptive names (e.g., `PR_1234_ANALYSIS.md`)

  - `.status_checked()?`: Executes and returns error if exit status != 0.

1. **Create** temporary analysis files with descriptive names (e.g., `PR_1234_ANALYSIS.md`)2. Use these files locally during development/validation

  - `.output_checked_utf8()?`: Like above but returns UTF-8 output as `String`.

2. **Use** these files locally during development/validation/review3. Before committing/pushing, remove temporary files: `git clean -fd`

  - Both automatically set up error context; prefer these over raw `.status()` calls.

3. **Before pushing**: Run `git clean -fd` to remove untracked temporary files4. Commit only `.md` files that are permanent documentation

- **`require()` and `require_one()` functions**:

4. **Commit** only permanent `.md` files (official documentation)5. Clean up locally-only `.txt` and analysis files

  - `require("binary_name")?` searches PATH and returns `SkipStep` if not found.

5. **Clean up** locally-only `.txt` and analysis files

  - `require_one(&["alt1", "alt2"])?` tries multiple binaries, returns first found or `SkipStep`.

## Error handling

  - Always use these before executing a tool to gracefully skip steps when tools are missing.

Example from real work:

- **Path operations**:

```- Use `color_eyre::Result<()>` as the return type for fallible operations. The `?` operator automatically converts errors to `color_eyre::Report`.

  - Use `home::home_dir()` via `HOME_DIR` LazyLock for user's home (handles Windows/Unix).

# During validation session, create analysis file

  - On Unix: use `XDG_DIRS` for XDG config paths; on Windows: use `WINDOWS_DIRS` for AppData.

$ echo "Validation notes..." > PR_1309_VALIDATION_NOTES.txt- Three main error scenarios:

  - Use `shellexpand::tilde_expand()` to handle `~` in user config paths.



## Development workflow

# Use it to track your findings  - **Tool not found/not applicable**: Return `Err(SkipStep("reason").into())`. The `Runner` interprets this as a skipped step, not a failure.

- Windows specifics live in `src/steps/os/windows.rs` (winget/choco/scoop, WSL). See `upgrade_wsl_distribution` for careful quoting through `wsl.exe`.

# (do your work here)

- Git repos step (`src/steps/git.rs`) discovers built-in + user repos, supports glob patterns, pulls concurrently (Tokio). Honor `git.max_concurrency` and `git.arguments`.

  - **Command failed**: Use `TopgradeError::ProcessFailed(cmd, status)` or `.status_checked()?` on commands via `CommandExt`.

- Always route process execution via `ExecutionContext` to preserve dry-run and unified error handling.

# Before pushing, clean up

## Security & Logging best practices

$ git clean -fd  - **Configuration/environment issue**: Use `TopgradeError` variants or custom error messages. Always provide context with `context("action")` if using eyre.

- **Command quoting and escaping**: Always be careful with shell expansion, especially when forwarding commands through `wsl.exe` or SSH. Use argument arrays instead of shell strings where possible.

$ rm PR_1309_VALIDATION_NOTES.txt

  - Windows/WSL tip: See `upgrade_wsl_distribution()` for careful quoting pattern when calling `wsl.exe`.

- Example pattern in a step:

  - Never embed user input directly in command strings; use `args()` instead.

# Now push your actual code/doc changes  ```rust

- **Logging with tracing**:

$ git push  pub fn run_xxx(ctx: &ExecutionContext) -> Result<()> {

  - Use `debug!("message")` for diagnostic info (respects `--log-filter` and `-v`).

```      let xxx = require("xxx")?;  // Returns SkipStep if not found

  - Use `info!()` for important steps, `warn!()` for recoverable issues, `error!()` for failures.

      print_separator("xxx");

  - Avoid logging sensitive information (paths, credentials). Redact before logging if necessary.

## Error handling      ctx.execute(xxx).args([...]).status_checked()?  // Fails if exit status != 0

  - When debugging, use `RUST_LOG=debug` or `--log-filter debug` to see trace output.

  }

- **Sudo and elevation**:

- Use `color_eyre::Result<()>` as the return type for fallible operations. The `?` operator automatically converts errors to `color_eyre::Report`.  ```

  - Call `ctx.require_sudo()?` to check if sudo is available before using it.



  - Use `SudoExecuteOpts` to configure sudo behavior (caching, env vars).

- Three main error scenarios:- Prefer early returns with `?` over nested match/if-let for cleaner code.

  - On Windows, elevation checks should guard Topgrade execution; see `is_elevated()`.



- **Environment variables**:

  - **Tool not found/not applicable**: Return `Err(SkipStep("reason").into())`. The `Runner` interprets this as a skipped step, not a failure.## Adding or modifying a step (example workflow)

  - Respect `SSH_CLIENT` and `SSH_TTY` to detect remote execution.



  - Use `env::var()` for reading, but don't expose values in debug logs.

  - **Command failed**: Use `TopgradeError::ProcessFailed(cmd, status)` or `.status_checked()?` on commands via `CommandExt`.- Add a variant to `Step` in `src/step.rs` (snake_case names are auto-wired by `clap/strum`).

  - Set variables via `env::set_var()` only when necessary for subcommand behavior.



- **Configuration security**:

  - **Configuration/environment issue**: Use `TopgradeError` variants or custom error messages. Always provide context with `context("action")` if using eyre.- Implement logic in `src/steps/<module>.rs` (or extend `generic.rs`):

  - Config files should respect filesystem permissions (especially on Unix).



  - Never log full config; use `Debug` trait selectively.

- Example pattern in a step:  - Guard with `require`/`require_one` for binaries.

  - Redact paths and credentials in error messages where possible.

  ```rust

## Build, run, test

  pub fn run_xxx(ctx: &ExecutionContext) -> Result<()> {  - Use `ctx.execute(...)` + `.args([..])` + `.status_checked()?`.

- **Build**: `cargo build` (MSRV: 1.84.1). Optional feature: `--features self-update`.

      let xxx = require("xxx")?;  // Returns SkipStep if not found

- **Run**: `cargo run -- [args]` (handy: `-n/--dry-run`, `-v/--verbose`, `--only <step>...`, `--disable <step>...`, `--log-filter ...`).

      print_separator("xxx");  - Read options via `ctx.config()` (e.g. `yes(Step::YourStep)`).

- **Tests**: `cargo test` (see `src/config.rs` tests).

      ctx.execute(xxx).args([...]).status_checked()?  // Fails if exit status != 0

- **Coverage**: Add or update unit/integration tests for new behaviour; run `cargo llvm-cov` if you need more insight, but quality scenarios matter more than a numeric threshold.

  }  - Return `SkipStep` when the tool isn't present or not applicable.

- **Git commits/pre-commit**:

  - On Windows PowerShell/CMD: pre-commit hooks may fail due to fork limitations with Git for Windows.  ```

  - **Recommended**: Run `pre-commit run` and `git commit` from WSL or another Unix-like shell.

  - If working across Windows and WSL, ensure hooks are configured to work in both environments.- Wire the variant in `Step::run()` and add to `default_steps()` behind appropriate `#[cfg(...)]`.



- **Pre-commit setup** (in WSL):- Prefer early returns with `?` over nested match/if-let for cleaner code.

  - Create/install a Python environment: `python3 -m venv ~/.venvs/pre-commit && ~/.venvs/pre-commit/bin/pip install pre-commit`

  - Run `pre-commit install` from the repo to ensure hooks execute correctly.## Common utility patterns



- **Release cross-build script**: `build-all.sh` (uses `cross`).## Adding or modifying a step (example workflow)



## Workflow & PR expectations- **PathExt trait** provides useful methods on any `AsRef<Path>`:



### Fork-based development workflow (best practice)- Add a variant to `Step` in `src/step.rs` (snake_case names are auto-wired by `clap/strum`).



This project uses a **fork-based workflow** for contributions:  - `.if_exists()`: Returns `Option<Self>`, useful for optional paths.



1. **Create a development branch** in your local repository:- Implement logic in `src/steps/<module>.rs` (or extend `generic.rs`):

   ```bash

   git checkout -b feature/descriptive-name  # or fix/, chore/, docs/, ci/, etc.  - `.require()`: Returns `Result<Self>` with `SkipStep` error if path doesn't exist.

   ```

  - Guard with `require`/`require_one` for binaries.

2. **Make your changes** and commit locally:

   - Follow [Karma Runner commit message format](http://karma-runner.github.io/6.2/dev/git-commit-msg.html)  - `.is_descendant_of(ancestor)`: Checks if a path is under another.

   - Example: `feat: add new step`, `fix: resolve bug`, `ci: improve security`, `docs: clarify instructions`

  - Use `ctx.execute(...)` + `.args([..])` + `.status_checked()?`.

3. **Run pre-commit validation** (from WSL if on Windows):

   ```bash- **CommandExt trait** extends `Command` with helpers:

   pre-commit run --all-files

   ```  - Read options via `ctx.config()` (e.g. `yes(Step::YourStep)`).



4. **Run tests and checks locally**:  - `.status_checked()?`: Executes and returns error if exit status != 0.

   ```bash

   cargo fmt  - Return `SkipStep` when the tool isn't present or not applicable.

   cargo clippy

   cargo test  - `.output_checked_utf8()?`: Like above but returns UTF-8 output as `String`.

   ```

- Wire the variant in `Step::run()` and add to `default_steps()` behind appropriate `#[cfg(...)]`.

5. **Push to your fork** (not to upstream):

   ```bash  - Both automatically set up error context; prefer these over raw `.status()` calls.

   git push origin feature/descriptive-name  # or git push <fork-remote> <branch>

   ```## Common utility patterns



6. **Open a PR against the upstream repository**:- **`require()` and `require_one()` functions**:

   - Base branch: `main` (topgrade-rs/topgrade)

   - Head branch: `your-fork:feature/descriptive-name`- **PathExt trait** provides useful methods on any `AsRef<Path>`:

   - Use the [pull request template](./PULL_REQUEST_TEMPLATE.md)

  - `require("binary_name")?` searches PATH and returns `SkipStep` if not found.

### PR expectations

  - `.if_exists()`: Returns `Option<Self>`, useful for optional paths.

- Follow the project [Code of Conduct](../CODE_OF_CONDUCT.md); maintain a respectful, inclusive tone in discussions and reviews.

  - `require_one(&["alt1", "alt2"])?` tries multiple binaries, returns first found or `SkipStep`.

- Read and adhere to the steps in [CONTRIBUTING.md](../CONTRIBUTING.md) before opening a PR (run `cargo fmt`, `cargo clippy`, `cargo test`, update docs/configs, document breaking changes, etc.).

  - `.require()`: Returns `Result<Self>` with `SkipStep` error if path doesn't exist.

- Pull requests must follow [pull request template](./PULL_REQUEST_TEMPLATE.md); copy the sections into the PR body, complete the summary, and tick the relevant checklist items before requesting review.

  - Always use these before executing a tool to gracefully skip steps when tools are missing.

- Mention any limitations (e.g., pre-commit issues on Windows) in the PR body so reviewers know what was skipped and why.

  - `.is_descendant_of(ancestor)`: Checks if a path is under another.

## Key references

- **Path operations**:

- Core: `src/main.rs`, `src/runner.rs`, `src/execution_context.rs`, `src/executor.rs`.

- Config: `src/config.rs`, `config.example.toml`, `src/utils.rs` (merge strategies, logging setup).- **CommandExt trait** extends `Command` with helpers:

- Steps: `src/step.rs`, `src/steps/generic.rs`, `src/steps/os/*`, `src/steps/node.rs`, `src/steps/git.rs`, `src/steps/remote/*`.

- i18n: `locales/app.yml`.  - Use `home::home_dir()` via `HOME_DIR` LazyLock for user's home (handles Windows/Unix).



If a section is unclear or you need a template for a specific tool/distro, ask and we'll refine this doc.  - `.status_checked()?`: Executes and returns error if exit status != 0.



---  - On Unix: use `XDG_DIRS` for XDG config paths; on Windows: use `WINDOWS_DIRS` for AppData.



**Note**: This file is for local reference only. Do not commit it to the repository. It's personal guidance for working with this codebase.  - `.output_checked_utf8()?`: Like above but returns UTF-8 output as `String`.


  - Use `shellexpand::tilde_expand()` to handle `~` in user config paths.

## Development workflow

- Windows specifics live in `src/steps/os/windows.rs` (winget/choco/scoop, WSL). See `upgrade_wsl_distribution` for careful quoting through `wsl.exe`.

### Before opening a PR

- Git repos step (`src/steps/git.rs`) discovers built-in + user repos, supports glob patterns, pulls concurrently (Tokio). Honor `git.max_concurrency` and `git.arguments`.

1. **Run pre-commit validation** (from WSL if on Windows):

   ```bash- Always route process execution via `ExecutionContext` to preserve dry-run and unified error handling.

   pre-commit run --all-files

   ```## Security & Logging best practices



2. **Run tests and checks locally**:- **Command quoting and escaping**: Always be careful with shell expansion, especially when forwarding commands through `wsl.exe` or SSH. Use argument arrays instead of shell strings where possible.

   ```bash

   cargo fmt  - Windows/WSL tip: See `upgrade_wsl_distribution()` for careful quoting pattern when calling `wsl.exe`.

   cargo clippy  

   cargo test  - Never embed user input directly in command strings; use `args()` instead.

   ```

- **Logging with tracing**:

3. **Clean up temporary files**:

   ```bash  - Use `debug!("message")` for diagnostic info (respects `--log-filter` and `-v`).

   git clean -fd  # Removes untracked files

   ```  - Use `info!()` for important steps, `warn!()` for recoverable issues, `error!()` for failures.



### Commit message format  - Avoid logging sensitive information (paths, credentials). Redact before logging if necessary.



- Follow [Karma Runner commit message format](http://karma-runner.github.io/6.2/dev/git-commit-msg.html)  - When debugging, use `RUST_LOG=debug` or `--log-filter debug` to see trace output.

- Examples: `feat: add new step`, `fix: resolve bug`, `ci: improve security`, `docs: clarify instructions`

- **Sudo and elevation**:

### PR expectations

  - Call `ctx.require_sudo()?` to check if sudo is available before using it.

- Follow the project [Code of Conduct](../CODE_OF_CONDUCT.md)

- Read and follow steps in [CONTRIBUTING.md](../CONTRIBUTING.md)  - Use `SudoExecuteOpts` to configure sudo behavior (caching, env vars).

- Run `cargo fmt`, `cargo clippy`, `cargo test` locally first

- Use the [pull request template](./PULL_REQUEST_TEMPLATE.md)  - On Windows, elevation checks should guard Topgrade execution; see `is_elevated()`.

- Mention limitations or skipped items in PR body

- **Environment variables**:

## Key references

  - Respect `SSH_CLIENT` and `SSH_TTY` to detect remote execution.

- Core: `src/main.rs`, `src/runner.rs`, `src/execution_context.rs`, `src/executor.rs`

- Config: `src/config.rs`, `config.example.toml`, `src/utils.rs`  - Use `env::var()` for reading, but don't expose values in debug logs.

- Steps: `src/step.rs`, `src/steps/generic.rs`, `src/steps/os/*`

- i18n: `locales/app.yml`  - Set variables via `env::set_var()` only when necessary for subcommand behavior.



---- **Configuration security**:



**Note**: This file is for local reference only. Do not commit it to the repository. It's personal guidance for working with this codebase.  - Config files should respect filesystem permissions (especially on Unix).


  - Never log full config; use `Debug` trait selectively.

  - Redact paths and credentials in error messages where possible.

## Build, run, test

- **Build**: `cargo build` (MSRV: 1.84.1). Optional feature: `--features self-update`.

- **Run**: `cargo run -- [args]` (handy: `-n/--dry-run`, `-v/--verbose`, `--only <step>...`, `--disable <step>...`, `--log-filter ...`).

- **Tests**: `cargo test` (see `src/config.rs` tests).

- **Coverage**: Add or update unit/integration tests for new behaviour; run `cargo llvm-cov` if you need more insight, but quality scenarios matter more than a numeric threshold.

- **Git commits/pre-commit**: 
  - On Windows PowerShell/CMD: pre-commit hooks may fail due to fork limitations with Git for Windows.
  - **Recommended**: Run `pre-commit run` and `git commit` from WSL or another Unix-like shell.
  - If working across Windows and WSL, ensure hooks are configured to work in both environments.

- **Pre-commit setup** (in WSL):
  - Create/install a Python environment: `python3 -m venv ~/.venvs/pre-commit && ~/.venvs/pre-commit/bin/pip install pre-commit`
  - Run `pre-commit install` from the repo to ensure hooks execute correctly.

- **Release cross-build script**: `build-all.sh` (uses `cross`).



## Workflow & PR expectations

### Fork-based development workflow (best practice)

This project uses a **fork-based workflow** for contributions:

1. **Create a development branch** in your local repository:
   ```bash
   git checkout -b feature/descriptive-name  # or fix/, chore/, docs/, ci/, etc.
   ```

2. **Make your changes** and commit locally:
   - Follow [Karma Runner commit message format](http://karma-runner.github.io/6.2/dev/git-commit-msg.html)
   - Example: `feat: add new step`, `fix: resolve bug`, `ci: improve security`, `docs: clarify instructions`

3. **Run pre-commit validation** (from WSL if on Windows):
   ```bash
   pre-commit run --all-files
   ```

4. **Run tests and checks locally**:
   ```bash
   cargo fmt
   cargo clippy
   cargo test
   ```

5. **Push to your fork** (not to upstream):
   ```bash
   git push origin feature/descriptive-name  # or git push <fork-remote> <branch>
   ```

6. **Open a PR against the upstream repository**:
   - Base branch: `main` (topgrade-rs/topgrade)
   - Head branch: `your-fork:feature/descriptive-name`
   - Use the [pull request template](./PULL_REQUEST_TEMPLATE.md)

### PR expectations

- Follow the project [Code of Conduct](../CODE_OF_CONDUCT.md); maintain a respectful, inclusive tone in discussions and reviews.

- Read and adhere to the steps in [CONTRIBUTING.md](../CONTRIBUTING.md) before opening a PR (run `cargo fmt`, `cargo clippy`, `cargo test`, update docs/configs, document breaking changes, etc.).

- Pull requests must follow [pull request template](./PULL_REQUEST_TEMPLATE.md); copy the sections into the PR body, complete the summary, and tick the relevant checklist items before requesting review.

- Mention any limitations (e.g., pre-commit issues on Windows) in the PR body so reviewers know what was skipped and why.

## Key references

- Core: `src/main.rs`, `src/runner.rs`, `src/execution_context.rs`, `src/executor.rs`.
- Config: `src/config.rs`, `config.example.toml`, `src/utils.rs` (merge strategies, logging setup).
- Steps: `src/step.rs`, `src/steps/generic.rs`, `src/steps/os/*`, `src/steps/node.rs`, `src/steps/git.rs`, `src/steps/remote/*`.
- i18n: `locales/app.yml`.

If a section is unclear or you need a template for a specific tool/distro, ask and we’ll refine this doc.