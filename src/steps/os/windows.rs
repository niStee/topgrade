use std::path::Path;
use std::{ffi::OsStr, process::Command};

use color_eyre::eyre::Result;
use etcetera::base_strategy::BaseStrategy;
use rust_i18n::t;
use tracing::debug;

use crate::command::CommandExt;
use crate::config::UpdatesAutoReboot;
use crate::execution_context::ExecutionContext;
use crate::step::Step;
use crate::terminal::{print_separator, print_warning};
use crate::utils::is_elevated;
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

// Embedded default SDIO script for checking driver updates (safe, no installs)
const SDIO_CHECK_SCRIPT: &str = r"logging on
verbose 384
enableinstall off
init
select missing better
end
";

// Embedded default SDIO script for installing driver updates
const SDIO_INSTALL_SCRIPT: &str = r"logging on
verbose 384
enableinstall on
init
select missing better
restorepoint Topgrade driver updates
install
end
";

/// Find SDIO binary in PATH or use configured path
fn find_sdio_binary(config_binary: Option<&str>) -> Result<std::path::PathBuf> {
    if let Some(binary_path) = config_binary {
        return Ok(std::path::PathBuf::from(binary_path));
    }

    // Try exact "SDIO" first
    if let Ok(path) = require("SDIO") {
        return Ok(path);
    }

    // Fallback: scan PATH for SDIO.exe or SDIO_R*.exe (prefer exact, then latest version)
    let mut exact_match: Option<std::path::PathBuf> = None;
    let mut versioned_candidates: Vec<std::path::PathBuf> = Vec::new();

    if let Some(path_os) = std::env::var_os("PATH") {
        for p in std::env::split_paths(&path_os) {
            if let Ok(rd) = std::fs::read_dir(&p) {
                for entry in rd.flatten() {
                    let path = entry.path();
                    if let Some(name) = path.file_name().and_then(|s| s.to_str()) {
                        let name_lower = name.to_ascii_lowercase();
                        if name_lower == "sdio.exe" {
                            exact_match = Some(path.clone());
                        } else if name_lower.starts_with("sdio_r") && name_lower.ends_with(".exe") {
                            versioned_candidates.push(path);
                        }
                    }
                }
            }
        }
    }

    if let Some(path) = exact_match {
        Ok(path)
    } else if !versioned_candidates.is_empty() {
        versioned_candidates.sort_by(|a, b| b.file_name().cmp(&a.file_name()));
        Ok(versioned_candidates.into_iter().next().unwrap())
    } else {
        let msg = format!(
            "{}; set [windows].sdio_binary to the full SDIO.exe path",
            t!("Cannot find {binary_name} in PATH", binary_name = "SDIO.exe")
        );
        Err(SkipStep(msg).into())
    }
}

pub fn run_sdio(ctx: &ExecutionContext) -> Result<()> {
    // Check if user confirmed this step
    if !ctx.config().yes(Step::Sdio) {
        return Err(SkipStep(t!("SDIO requires confirmation to run").to_string()).into());
    }

    // Warn if running with elevated privileges
    if is_elevated() {
        print_warning("SDIO running with elevated privileges - driver installation may require admin rights");
    }

    print_separator("SDIO");

    // Get SDIO binary
    let sdio = find_sdio_binary(ctx.config().sdio_binary())?;

    // Determine script to use: custom override or embedded default
    let (script_path, cleanup_script) = if let Some(custom_script) = ctx.config().sdio_script() {
        // User provided custom script - validate it exists and is a file
        let script_path_buf = std::path::PathBuf::from(custom_script);
        if !script_path_buf.exists() {
            return Err(SkipStep(format!("SDIO script not found: {}", custom_script)).into());
        }
        if !script_path_buf.is_file() {
            return Err(SkipStep(format!("SDIO script path is not a file: {}", custom_script)).into());
        }
        (script_path_buf, false) // Don't cleanup user's script
    } else {
        // Use embedded default script based on upgrade mode
        let script_content = if ctx.config().sdio_upgrade() {
            SDIO_INSTALL_SCRIPT
        } else {
            SDIO_CHECK_SCRIPT
        };

        // Write to temp file
        let temp_dir = std::env::temp_dir();
        let script_path = temp_dir.join(format!("topgrade_sdio_{}.txt", std::process::id()));

        std::fs::write(&script_path, script_content.as_bytes())
            .map_err(|e| SkipStep(format!("Failed to write temp SDIO script: {}", e)))?;

        (script_path, true) // Cleanup our temp file
    };

    // Execute SDIO with the script (ensure cleanup happens even if execution fails)
    let result = (|| -> Result<()> {
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
        Ok(())
    })();

    // Cleanup temp file if we created one
    if cleanup_script {
        let _ = std::fs::remove_file(&script_path);
    }

    result
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
