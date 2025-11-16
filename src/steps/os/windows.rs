use std::path::Path;
use std::{ffi::OsStr, process::Command};

use color_eyre::eyre::{eyre, Result};
use etcetera::base_strategy::BaseStrategy;
use rust_i18n::t;
use tracing::debug;

use crate::command::CommandExt;
use crate::config::UpdatesAutoReboot;
use crate::execution_context::ExecutionContext;
use crate::step::Step;
use crate::terminal::{print_separator, print_warning};
use crate::utils::{require, which};
use crate::{error::SkipStep, steps::git::RepoStep};

pub fn run_chocolatey(ctx: &ExecutionContext) -> Result<()> {
    let choco = require("choco")?;
    let yes = ctx.config().yes(Step::Chocolatey);

    print_separator("Chocolatey");

    let sudo = ctx.require_sudo()?;

    let mut command = sudo.execute(ctx, &choco)?;
    command.args(["upgrade", "all"]);

    if yes {
        command.arg("--yes");
    }

    command.status_checked()
}

pub fn run_winget(ctx: &ExecutionContext) -> Result<()> {
    let winget = require("winget")?;

    print_separator("winget");

    ctx.execute(&winget).args(["source", "update"]).status_checked()?;

    let mut command = if ctx.config().winget_use_sudo() {
        let sudo = ctx.require_sudo()?;
        sudo.execute(ctx, &winget)?
    } else {
        ctx.execute(winget)
    };

    let mut args = vec!["upgrade", "--all"];
    if ctx.config().winget_silent_install() {
        args.push("--silent");
    }

    command.args(args).status_checked()?;

    Ok(())
}

pub fn run_scoop(ctx: &ExecutionContext) -> Result<()> {
    let scoop = require("scoop")?;

    print_separator("Scoop");

    ctx.execute(&scoop).args(["update"]).status_checked()?;
    ctx.execute(&scoop).args(["update", "*"]).status_checked()?;

    if ctx.config().cleanup() {
        ctx.execute(&scoop).args(["cleanup", "*"]).status_checked()?;
        ctx.execute(&scoop).args(["cache", "rm", "-a"]).status_checked()?
    }
    Ok(())
}

/// SDIO (Snappy Driver Installer Origin) driver update step
/// Mirrors the semantics of the Linux firmware step: default is a safe check-only mode, optional install when enabled.
pub fn run_sdio(ctx: &ExecutionContext) -> Result<()> {
    // Must be explicitly confirmed
    if !ctx.config().yes(Step::Sdio) {
        return Err(SkipStep(t!("SDIO requires confirmation to run").to_string()).into());
    }

    print_separator("SDIO");

    // Warn about elevation (drivers can affect system stability)
    if crate::utils::is_elevated() {
        print_warning("Running SDIO elevated – ensure trust of binary and script (if overridden)");
    }

    // Locate SDIO binary (preferred: exact SDIO, fallback: SDIO_R*.exe)
    let sdio = match require("SDIO") {
        Ok(path) => path,
        Err(_) => {
            // Fallback path scan
            let mut exact: Option<std::path::PathBuf> = None;
            let mut versioned: Vec<std::path::PathBuf> = Vec::new();
            if let Some(path_os) = std::env::var_os("PATH") {
                for dir in std::env::split_paths(&path_os) {
                    if let Ok(read) = std::fs::read_dir(&dir) {
                        for entry in read.flatten() {
                            let p = entry.path();
                            if let Some(name) = p.file_name().and_then(|s| s.to_str()) {
                                let lower = name.to_ascii_lowercase();
                                if lower == "sdio.exe" {
                                    exact = Some(p.clone());
                                } else if lower.starts_with("sdio_r") && lower.ends_with(".exe") {
                                    versioned.push(p);
                                }
                            }
                        }
                    }
                }
            }
            if let Some(p) = exact {
                p
            } else if !versioned.is_empty() {
                versioned.sort_by(|a, b| b.file_name().cmp(&a.file_name()));
                versioned.remove(0)
            } else {
                return Err(SkipStep("SDIO.exe not found in PATH; set windows.sdio_binary".into()).into());
            }
        }
    };

    // Allow explicit binary override
    let sdio = if let Some(explicit) = ctx.config().sdio_binary() {
        std::path::PathBuf::from(explicit)
    } else {
        sdio
    };

    // Script override vs embedded minimal script
    let (script_path, _temp_file): (std::path::PathBuf, Option<tempfile::NamedTempFile>) = if let Some(custom) =
        ctx.config().sdio_script()
    {
        let custom_path = std::path::PathBuf::from(custom);
        if !custom_path.exists() || !custom_path.is_file() {
            return Err(SkipStep(format!("SDIO script invalid: {}", custom)).into());
        }
        (custom_path, None)
    } else {
        let upgrade = ctx.config().sdio_upgrade();
        let content = if upgrade {
            // Install script (restore point + install)
            "logging on\nverbose 384\ninit\nselect missing better\nrestorepoint Topgrade driver updates\nenableinstall on\ninstall\nend\n"
        } else {
            // Check-only script
            "logging on\nverbose 384\ninit\nselect missing better\nenableinstall off\nend\n"
        };
        let mut tmp = tempfile::Builder::new()
            .prefix("topgrade_sdio_")
            .suffix(".txt")
            .tempfile()
            .map_err(|e| eyre!("Failed to create temp SDIO script: {e}"))?;
        use std::io::Write;
        tmp.write_all(content.as_bytes())
            .map_err(|e| eyre!("Failed to write temp SDIO script: {e}"))?;
        let path = tmp.path().to_path_buf();
        (path, Some(tmp))
    };

    // Execute – single pass (script contains install logic if enabled)
    if ctx.config().sdio_upgrade() {
        println!("{}", t!("Checking and installing driver updates..."));
    } else {
        println!("{}", t!("Checking for driver updates..."));
    }

    ctx.execute(&sdio)
        .arg(format!("-script:{}", script_path.display()))
        .status_checked()?;

    if ctx.config().sdio_upgrade() {
        println!("{}", t!("SDIO driver updates completed"));
    } else {
        println!("{}", t!("SDIO driver check completed"));
    }

    // temp_file drops automatically; no explicit cleanup needed
    Ok(())
}

pub fn update_wsl(ctx: &ExecutionContext) -> Result<()> {
    if !is_wsl_installed()? {
        return Err(SkipStep(t!("WSL not installed").to_string()).into());
    }

    let wsl = require("wsl")?;

    print_separator(t!("Update WSL"));

    let mut wsl_command = ctx.execute(wsl);
    wsl_command.args(["--update"]);

    if ctx.config().wsl_update_pre_release() {
        wsl_command.args(["--pre-release"]);
    }

    if ctx.config().wsl_update_use_web_download() {
        wsl_command.args(["--web-download"]);
    }
    wsl_command.status_checked()?;
    Ok(())
}

/// Detect if WSL is installed or not.
///
/// For WSL, we cannot simply check if command `wsl` is installed as on newer
/// versions of Windows (since windows 10 version 2004), this command is
/// installed by default.
///
/// If the command is installed and the user hasn't installed any Linux distros
/// on it, command `wsl -l` would print a help message and exit with failure, we
/// use this to check whether WSL is install or not.
fn is_wsl_installed() -> Result<bool> {
    if let Some(wsl) = which("wsl") {
        // Don't use `output_checked` as an execution failure log is not wanted
        #[allow(clippy::disallowed_methods)]
        let output = Command::new(wsl).arg("-l").output()?;
        let status = output.status;

        if status.success() {
            return Ok(true);
        }
    }

    Ok(false)
}

fn get_wsl_distributions(wsl: &Path) -> Result<Vec<String>> {
    let output = Command::new(wsl).args(["--list", "-q"]).output_checked_utf8()?.stdout;
    Ok(output
        .lines()
        .map(|x| x.replace(['\u{0}', '\r'], "").trim().to_owned())
        .filter(|s| !s.is_empty())
        .collect())
}

fn upgrade_wsl_distribution(wsl: &Path, dist: &str, ctx: &ExecutionContext) -> Result<()> {
    let topgrade = Command::new(wsl)
        .args(["-d", dist, "bash", "-lc", "which topgrade"])
        .output_checked_utf8()
        .map_err(|_| SkipStep(t!("Could not find Topgrade installed in WSL").to_string()))?
        .stdout // The normal output from `which topgrade` appends a newline, so we trim it here.
        .trim_end()
        .to_owned();

    let mut command = ctx.execute(wsl);

    // The `arg` method automatically quotes its arguments.
    // This means we can't append additional arguments to `topgrade` in WSL
    // by calling `arg` successively.
    //
    // For example:
    //
    // ```rust
    // command
    //  .args(["-d", dist, "bash", "-lc"])
    //  .arg(format!("TOPGRADE_PREFIX={dist} exec {topgrade}"));
    // ```
    //
    // creates a command string like:
    // > `C:\WINDOWS\system32\wsl.EXE -d Ubuntu bash -lc 'TOPGRADE_PREFIX=Ubuntu exec /bin/topgrade'`
    //
    // Adding the following:
    //
    // ```rust
    // command.arg("-v");
    // ```
    //
    // appends the next argument like so:
    // > `C:\WINDOWS\system32\wsl.EXE -d Ubuntu bash -lc 'TOPGRADE_PREFIX=Ubuntu exec /bin/topgrade' -v`
    // which means `-v` isn't passed to `topgrade`.
    let mut args = String::new();
    if ctx.config().verbose() {
        args.push_str("-v");
    }

    command
        .args(["-d", dist, "bash", "-lc"])
        .arg(format!("TOPGRADE_PREFIX={dist} exec {topgrade} {args}"));

    if ctx.config().yes(Step::Wsl) {
        command.arg("-y");
    }

    command.status_checked()
}

pub fn run_wsl_topgrade(ctx: &ExecutionContext) -> Result<()> {
    if !is_wsl_installed()? {
        return Err(SkipStep(t!("WSL not installed").to_string()).into());
    }

    let wsl = require("wsl")?;
    let wsl_distributions = get_wsl_distributions(&wsl)?;
    let mut ran = false;

    debug!("WSL distributions: {:?}", wsl_distributions);

    for distribution in wsl_distributions {
        let result = upgrade_wsl_distribution(&wsl, &distribution, ctx);
        debug!("Upgrading {:?}: {:?}", distribution, result);
        if let Err(e) = result {
            if e.is::<SkipStep>() {
                continue;
            }
        }
        ran = true
    }

    if ran {
        Ok(())
    } else {
        Err(SkipStep(t!("Could not find Topgrade in any WSL distribution").to_string()).into())
    }
}

pub fn windows_update(ctx: &ExecutionContext) -> Result<()> {
    let powershell = ctx.require_powershell()?;

    print_separator(t!("Windows Update"));

    if !powershell.has_module("PSWindowsUpdate") {
        print_warning(t!(
            "The PSWindowsUpdate PowerShell module isn't installed so Topgrade can't run Windows Update.\nInstall PSWindowsUpdate by running `Install-Module PSWindowsUpdate` in PowerShell."
        ));

        return Err(SkipStep(t!("PSWindowsUpdate is not installed").to_string()).into());
    }

    let mut cmd = "Import-Module PSWindowsUpdate; Install-WindowsUpdate -Verbose".to_string();

    if ctx.config().accept_all_windows_updates() {
        cmd.push_str(" -AcceptAll");
    }

    match ctx.config().windows_updates_auto_reboot() {
        UpdatesAutoReboot::Yes => cmd.push_str(" -AutoReboot"),
        UpdatesAutoReboot::No => cmd.push_str(" -IgnoreReboot"),
        UpdatesAutoReboot::Ask => (), // Prompting is the default for Install-WindowsUpdate
    }

    powershell.build_command(ctx, &cmd, true)?.status_checked()
}

pub fn microsoft_store(ctx: &ExecutionContext) -> Result<()> {
    let powershell = ctx.require_powershell()?;

    print_separator(t!("Microsoft Store"));

    println!("{}", t!("Scanning for updates..."));

    // Scan for updates using the MDM UpdateScanMethod
    // This method is also available for non-MDM devices
    let cmd = r#"(Get-CimInstance -Namespace "Root\cimv2\mdm\dmmap" -ClassName "MDM_EnterpriseModernAppManagement_AppManagement01" | Invoke-CimMethod -MethodName UpdateScanMethod).ReturnValue"#;

    powershell
        .build_command(ctx, cmd, true)?
        .output_checked_with_utf8(|output| {
            if !output.status.success() {
                return Err(());
            }
            let ret_val = output.stdout.trim();
            debug!("Command return value: {}", ret_val);
            if ret_val == "0" {
                Ok(())
            } else {
                Err(())
            }
        })?;
    println!(
        "{}",
        t!("Success, Microsoft Store apps are being updated in the background")
    );
    Ok(())
}

pub fn reboot(ctx: &ExecutionContext) -> Result<()> {
    // If this works, it won't return, but if it doesn't work, it may return a useful error
    // message.
    ctx.execute("shutdown.exe").args(["/R", "/T", "0"]).status_checked()
}

pub fn insert_startup_scripts(git_repos: &mut RepoStep) -> Result<()> {
    let startup_dir = crate::WINDOWS_DIRS
        .data_dir()
        .join("Microsoft\\Windows\\Start Menu\\Programs\\Startup");
    for entry in std::fs::read_dir(&startup_dir)?.flatten() {
        let path = entry.path();
        if path.extension().and_then(OsStr::to_str) == Some("lnk") {
            if let Ok(lnk) = parselnk::Lnk::try_from(Path::new(&path)) {
                debug!("Startup link: {:?}", lnk);
                if let Some(path) = lnk.relative_path() {
                    git_repos.insert_if_repo(startup_dir.join(path));
                }
            }
        }
    }

    Ok(())
}
