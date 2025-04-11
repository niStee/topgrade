#![allow(unused_imports)]

#[cfg(unix)]
use std::os::unix::prelude::MetadataExt;
use std::path::{Path, PathBuf};
use std::process::Command;

use anyhow::Result;
use directories::BaseDirs;
use log::debug;
#[cfg(unix)]
use nix::unistd::Uid;
use semver::Version;

use crate::executor::{CommandExt, RunType};
use crate::terminal::print_separator;
use crate::utils::{require, PathExt};
use crate::{error::SkipStep, execution_context::ExecutionContext};

/// Checks if the path is likely a Windows path accessed through WSL
#[cfg(target_os = "linux")]
fn is_wsl_windows_path(path: &Path) -> bool {
    path.to_string_lossy().contains("/mnt/")
}

/// Checks if node exists in the same directory as the given command
#[cfg(target_os = "linux")]
fn check_node_available(command: &Path) -> Result<()> {
    // Check if node exists in the same directory as the command
    if let Some(parent_dir) = command.parent() {
        let node_path = parent_dir.join("node");
        if !node_path.exists() && is_wsl_windows_path(command) {
            return Err(SkipStep(format!(
                "Found {} command at {}, but node executable is missing. This may be because you're running WSL with Node.js installed on the Windows side. Consider installing Node.js directly in WSL.",
                command.file_name().unwrap_or_default().to_string_lossy(),
                command.display()
            )).into());
        }
    }
    Ok(())
}

#[allow(clippy::upper_case_acronyms)]
struct NPM {
    command: PathBuf,
}

impl NPM {
    fn new(command: PathBuf) -> Self {
        Self { command }
    }

    #[cfg(target_os = "linux")]
    fn root(&self) -> Result<PathBuf> {
        // Check that node is available before proceeding
        check_node_available(&self.command)?;

        let version = self.version()?;
        let args = if version < Version::new(8, 11, 0) {
            ["root", "-g"]
        } else {
            ["root", "--location=global"]
        };
        Command::new(&self.command)
            .args(args)
            .check_output()
            .map(|s| PathBuf::from(s.trim()))
    }

    fn version(&self) -> Result<Version> {
        // Check that node is available before proceeding
        #[cfg(target_os = "linux")]
        check_node_available(&self.command)?;

        let version_str = Command::new(&self.command)
            .args(["--version"])
            .check_output()
            .map(|s| s.trim().to_owned());
        Version::parse(&version_str?).map_err(|err| err.into())
    }

    fn upgrade(&self, run_type: RunType, use_sudo: bool) -> Result<()> {
        // Check that node is available before proceeding
        #[cfg(target_os = "linux")]
        check_node_available(&self.command)?;

        print_separator("Node Package Manager");
        let version = self.version()?;
        let args = if version < Version::new(8, 11, 0) {
            ["update", "-g"]
        } else {
            ["update", "--location=global"]
        };
        if use_sudo {
            run_type.execute("sudo").args(args).check_run()?;
        } else {
            run_type.execute(&self.command).args(args).check_run()?;
        }

        Ok(())
    }

    #[cfg(target_os = "linux")]
    pub fn should_use_sudo(&self) -> Result<bool> {
        let npm_root = self.root()?;
        if !npm_root.exists() {
            return Err(SkipStep(format!("NPM root at {} doesn't exist", npm_root.display(),)).into());
        }

        let metadata = std::fs::metadata(&npm_root)?;
        let uid = Uid::effective();

        Ok(metadata.uid() != uid.as_raw() && metadata.uid() == 0)
    }
}

struct Yarn {
    command: PathBuf,
    yarn: Option<PathBuf>,
}

impl Yarn {
    fn new(command: PathBuf) -> Self {
        Self {
            command,
            yarn: require("yarn").ok(),
        }
    }

    #[cfg(target_os = "linux")]
    fn root(&self) -> Result<PathBuf> {
        // Check that node is available before proceeding
        check_node_available(&self.command)?;

        let args = ["global", "dir"];
        Command::new(&self.command)
            .args(args)
            .check_output()
            .map(|s| PathBuf::from(s.trim()))
    }

    fn upgrade(&self, run_type: RunType, use_sudo: bool) -> Result<()> {
        // Check that node is available before proceeding
        #[cfg(target_os = "linux")]
        check_node_available(&self.command)?;

        print_separator("Yarn Package Manager");
        let args = ["global", "upgrade"];

        if use_sudo {
            run_type
                .execute("sudo")
                .arg(self.yarn.as_ref().unwrap_or(&self.command))
                .args(args)
                .check_run()?;
        } else {
            run_type.execute(&self.command).args(args).check_run()?;
        }

        Ok(())
    }

    #[cfg(target_os = "linux")]
    pub fn should_use_sudo(&self) -> Result<bool> {
        let yarn_root = self.root()?;
        if !yarn_root.exists() {
            return Err(SkipStep(format!("NPM root at {} doesn't exist", yarn_root.display(),)).into());
        }

        let metadata = std::fs::metadata(&yarn_root)?;
        let uid = Uid::effective();

        Ok(metadata.uid() != uid.as_raw() && metadata.uid() == 0)
    }
}

#[cfg(target_os = "linux")]
fn should_use_sudo(npm: &NPM, ctx: &ExecutionContext) -> Result<bool> {
    if npm.should_use_sudo()? {
        if ctx.config().npm_use_sudo() {
            Ok(true)
        } else {
            Err(SkipStep("NPM root is owned by another user which is not the current user. Set use_sudo = true under the NPM section in your configuration to run NPM as sudo".to_string())
                .into())
        }
    } else {
        Ok(false)
    }
}

#[cfg(target_os = "linux")]
fn should_use_sudo_yarn(yarn: &Yarn, ctx: &ExecutionContext) -> Result<bool> {
    if yarn.should_use_sudo()? {
        if ctx.config().yarn_use_sudo() {
            Ok(true)
        } else {
            Err(SkipStep("NPM root is owned by another user which is not the current user. Set use_sudo = true under the NPM section in your configuration to run NPM as sudo".to_string())
                .into())
        }
    } else {
        Ok(false)
    }
}

/// Wrapper to handle finding Node package manager and dealing with WSL edge cases
#[cfg(target_os = "linux")]
fn find_package_manager(npm_command: &str, fallback_command: &str) -> Result<PathBuf> {
    // First try to find a Linux-native binary
    let command = require(npm_command);

    if let Ok(path) = &command {
        if !is_wsl_windows_path(path) {
            return Ok(path.clone());
        }
        // Found Windows binary, but let's check if we have a Linux one first
        debug!(
            "Found Windows {} at {}, checking for Linux version",
            npm_command,
            path.display()
        );
    }

    // Try looking for the command in Linux paths only
    let linux_paths = std::env::var("PATH")
        .unwrap_or_default()
        .split(':')
        .filter(|p| !p.contains("/mnt/"))
        .collect::<Vec<_>>()
        .join(":");

    let linux_command = Command::new("sh")
        .args([
            "-c",
            &format!("PATH=\"{}\" which {} 2>/dev/null", linux_paths, npm_command),
        ])
        .output()
        .ok()
        .and_then(|output| {
            if output.status.success() {
                let path_str = String::from_utf8_lossy(&output.stdout).trim().to_string();
                if !path_str.is_empty() {
                    Some(PathBuf::from(path_str))
                } else {
                    None
                }
            } else {
                None
            }
        });

    if let Some(path) = linux_command {
        debug!("Found Linux {} at {}", npm_command, path.display());
        return Ok(path);
    }

    // Fallback to the original result or try the fallback command
    command.or_else(|_| require(fallback_command))
}

#[cfg(not(target_os = "linux"))]
fn find_package_manager(npm_command: &str, fallback_command: &str) -> Result<PathBuf> {
    require(npm_command).or_else(|_| require(fallback_command))
}

pub fn run_npm_upgrade(ctx: &ExecutionContext) -> Result<()> {
    let npm_path = find_package_manager("pnpm", "npm")?;
    let npm = NPM::new(npm_path);

    #[cfg(target_os = "linux")]
    {
        npm.upgrade(ctx.run_type(), should_use_sudo(&npm, ctx)?)
    }

    #[cfg(not(target_os = "linux"))]
    {
        npm.upgrade(ctx.run_type(), false)
    }
}

pub fn run_yarn_upgrade(ctx: &ExecutionContext) -> Result<()> {
    let yarn_path = find_package_manager("yarn", "yarn")?;
    let yarn = Yarn::new(yarn_path);

    #[cfg(target_os = "linux")]
    {
        yarn.upgrade(ctx.run_type(), should_use_sudo_yarn(&yarn, ctx)?)
    }

    #[cfg(not(target_os = "linux"))]
    {
        yarn.upgrade(ctx.run_type(), false)
    }
}

pub fn deno_upgrade(ctx: &ExecutionContext) -> Result<()> {
    let deno = require("deno")?;
    let deno_dir = ctx.base_dirs().home_dir().join(".deno");

    if !deno.canonicalize()?.is_descendant_of(&deno_dir) {
        let skip_reason = SkipStep("Deno installed outside of .deno directory".to_string());
        return Err(skip_reason.into());
    }

    print_separator("Deno");
    ctx.run_type().execute(&deno).arg("upgrade").check_run()
}
