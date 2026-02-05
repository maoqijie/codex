use std::ffi::OsString;
use std::io;
use std::path::Path;
use std::path::PathBuf;

pub use runfiles;

/// Bazel sets this when runfiles directories are disabled, which we do on all platforms for consistency.
const RUNFILES_MANIFEST_ONLY_ENV: &str = "RUNFILES_MANIFEST_ONLY";

static CARGO_BUILD_LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());

#[derive(Debug, thiserror::Error)]
pub enum CargoBinError {
    #[error("failed to read current exe")]
    CurrentExe {
        #[source]
        source: std::io::Error,
    },
    #[error("failed to read current directory")]
    CurrentDir {
        #[source]
        source: std::io::Error,
    },
    #[error("CARGO_BIN_EXE env var {key} resolved to {path:?}, but it does not exist")]
    ResolvedPathDoesNotExist { key: String, path: PathBuf },
    #[error("could not locate binary {name:?}; tried env vars {env_keys:?}; {fallback}")]
    NotFound {
        name: String,
        env_keys: Vec<String>,
        fallback: String,
    },
}

/// Returns an absolute path to a binary target built for the current test run.
///
/// In `cargo test`, `CARGO_BIN_EXE_*` env vars are absolute.
/// In `bazel test`, `CARGO_BIN_EXE_*` env vars are rlocationpaths, intended to be consumed by `rlocation`.
/// This helper allows callers to transparently support both.
#[allow(deprecated)]
pub fn cargo_bin(name: &str) -> Result<PathBuf, CargoBinError> {
    let env_keys = cargo_bin_env_keys(name);
    for key in &env_keys {
        if let Some(value) = std::env::var_os(key) {
            return resolve_bin_from_env(key, value);
        }
    }

    let mut build_attempt = None;
    for attempt in 0..2 {
        match assert_cmd::Command::cargo_bin(name) {
            Ok(cmd) => {
                let path = resolve_assert_cmd_path(&cmd)?;
                if path.exists() {
                    return Ok(path);
                }

                if attempt == 0 && !runfiles_available() {
                    build_attempt = Some(try_build_cargo_bin(name));
                    continue;
                }

                let build_note = match build_attempt {
                    Some(Ok(())) => " (cargo build fallback succeeded)".to_owned(),
                    Some(Err(ref err)) => format!(" (cargo build fallback failed: {err})"),
                    None => String::new(),
                };

                return Err(CargoBinError::ResolvedPathDoesNotExist {
                    key: format!("assert_cmd::Command::cargo_bin{build_note}"),
                    path,
                });
            }
            Err(err) => {
                if attempt == 0 && !runfiles_available() {
                    build_attempt = Some(try_build_cargo_bin(name));
                    continue;
                }

                let build_note = match build_attempt {
                    Some(Ok(())) => "; cargo build fallback succeeded".to_owned(),
                    Some(Err(ref err)) => format!("; cargo build fallback failed: {err}"),
                    None => String::new(),
                };

                return Err(CargoBinError::NotFound {
                    name: name.to_owned(),
                    env_keys,
                    fallback: format!("assert_cmd fallback failed: {err}{build_note}"),
                });
            }
        }
    }

    unreachable!("cargo_bin should return on attempts")
}

fn cargo_bin_env_keys(name: &str) -> Vec<String> {
    let mut keys = Vec::with_capacity(2);
    keys.push(format!("CARGO_BIN_EXE_{name}"));

    // Cargo replaces dashes in target names when exporting env vars.
    let underscore_name = name.replace('-', "_");
    if underscore_name != name {
        keys.push(format!("CARGO_BIN_EXE_{underscore_name}"));
    }

    keys
}

fn resolve_assert_cmd_path(cmd: &assert_cmd::Command) -> Result<PathBuf, CargoBinError> {
    fn resolve_candidate(candidate: PathBuf) -> Option<PathBuf> {
        if candidate.exists() {
            return Some(candidate);
        }
        if cfg!(windows) && candidate.extension().is_none() {
            let exe = candidate.with_extension("exe");
            if exe.exists() {
                return Some(exe);
            }
        }
        None
    }

    let path = PathBuf::from(cmd.get_program());
    if path.is_absolute() {
        return Ok(path);
    }

    let current_dir =
        std::env::current_dir().map_err(|source| CargoBinError::CurrentDir { source })?;
    let current_dir_candidate = current_dir.join(&path);
    if let Some(resolved) = resolve_candidate(current_dir_candidate.clone()) {
        return Ok(resolved);
    }

    if let Ok(repo_root) = repo_root() {
        let workspace_candidate = repo_root.join("codex-rs").join(&path);
        if let Some(resolved) = resolve_candidate(workspace_candidate) {
            return Ok(resolved);
        }

        let repo_root_candidate = repo_root.join(&path);
        if let Some(resolved) = resolve_candidate(repo_root_candidate) {
            return Ok(resolved);
        }
    }

    Ok(current_dir_candidate)
}

fn try_build_cargo_bin(name: &str) -> io::Result<()> {
    let _lock = CARGO_BUILD_LOCK
        .lock()
        .map_err(|_| io::Error::other("cargo build lock poisoned"))?;

    let repo_root = repo_root()?;
    let manifest_path = repo_root.join("codex-rs").join("Cargo.toml");
    if !manifest_path.exists() {
        let manifest_path_display = manifest_path.display();
        return Err(io::Error::new(
            io::ErrorKind::NotFound,
            format!("workspace Cargo.toml not found at {manifest_path_display}"),
        ));
    }

    let status = std::process::Command::new("cargo")
        .arg("build")
        .arg("--quiet")
        .arg("--manifest-path")
        .arg(&manifest_path)
        .arg("--bin")
        .arg(name)
        .status()?;

    if status.success() {
        Ok(())
    } else {
        Err(io::Error::other(format!(
            "cargo build --bin {name} failed with status {status}"
        )))
    }
}

pub fn runfiles_available() -> bool {
    std::env::var_os(RUNFILES_MANIFEST_ONLY_ENV).is_some()
}

fn resolve_bin_from_env(key: &str, value: OsString) -> Result<PathBuf, CargoBinError> {
    let raw = PathBuf::from(&value);
    if runfiles_available() {
        let runfiles = runfiles::Runfiles::create().map_err(|err| CargoBinError::CurrentExe {
            source: std::io::Error::other(err),
        })?;
        if let Some(resolved) = runfiles::rlocation!(runfiles, &raw)
            && resolved.exists()
        {
            return Ok(resolved);
        }
    } else if raw.is_absolute() && raw.exists() {
        return Ok(raw);
    }

    Err(CargoBinError::ResolvedPathDoesNotExist {
        key: key.to_owned(),
        path: raw,
    })
}

/// Macro that derives the path to a test resource at runtime, the value of
/// which depends on whether Cargo or Bazel is being used to build and run a
/// test. Note the return value may be a relative or absolute path.
/// (Incidentally, this is a macro rather than a function because it reads
/// compile-time environment variables that need to be captured at the call
/// site.)
///
/// This is expected to be used exclusively in test code because Codex CLI is a
/// standalone binary with no packaged resources.
#[macro_export]
macro_rules! find_resource {
    ($resource:expr) => {{
        let resource = std::path::Path::new(&$resource);
        if $crate::runfiles_available() {
            // When this code is built and run with Bazel:
            // - we inject `BAZEL_PACKAGE` as a compile-time environment variable
            //   that points to native.package_name()
            // - at runtime, Bazel will set runfiles-related env vars
            $crate::resolve_bazel_runfile(option_env!("BAZEL_PACKAGE"), resource)
        } else {
            let manifest_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
            Ok(manifest_dir.join(resource))
        }
    }};
}

pub fn resolve_bazel_runfile(
    bazel_package: Option<&str>,
    resource: &Path,
) -> std::io::Result<PathBuf> {
    let runfiles = runfiles::Runfiles::create()
        .map_err(|err| std::io::Error::other(format!("failed to create runfiles: {err}")))?;
    let runfile_path = match bazel_package {
        Some(bazel_package) => PathBuf::from("_main").join(bazel_package).join(resource),
        None => {
            return Err(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                "BAZEL_PACKAGE was not set at compile time",
            ));
        }
    };
    let runfile_path = normalize_runfile_path(&runfile_path);
    if let Some(resolved) = runfiles::rlocation!(runfiles, &runfile_path)
        && resolved.exists()
    {
        return Ok(resolved);
    }
    let runfile_path_display = runfile_path.display();
    Err(std::io::Error::new(
        std::io::ErrorKind::NotFound,
        format!("runfile does not exist at: {runfile_path_display}"),
    ))
}

pub fn resolve_cargo_runfile(resource: &Path) -> std::io::Result<PathBuf> {
    let manifest_dir = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    Ok(manifest_dir.join(resource))
}

pub fn repo_root() -> io::Result<PathBuf> {
    let marker = if runfiles_available() {
        let runfiles = runfiles::Runfiles::create()
            .map_err(|err| io::Error::other(format!("failed to create runfiles: {err}")))?;
        let marker_path = option_env!("CODEX_REPO_ROOT_MARKER")
            .map(PathBuf::from)
            .ok_or_else(|| {
                io::Error::new(
                    io::ErrorKind::NotFound,
                    "CODEX_REPO_ROOT_MARKER was not set at compile time",
                )
            })?;
        runfiles::rlocation!(runfiles, &marker_path).ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::NotFound,
                "repo_root.marker not available in runfiles",
            )
        })?
    } else {
        resolve_cargo_runfile(Path::new("repo_root.marker"))?
    };
    let mut root = marker;
    for _ in 0..4 {
        root = root
            .parent()
            .ok_or_else(|| {
                io::Error::new(
                    io::ErrorKind::NotFound,
                    "repo_root.marker did not have expected parent depth",
                )
            })?
            .to_path_buf();
    }
    Ok(root)
}

fn normalize_runfile_path(path: &Path) -> PathBuf {
    let mut components = Vec::new();
    for component in path.components() {
        match component {
            std::path::Component::CurDir => {}
            std::path::Component::ParentDir => {
                if matches!(components.last(), Some(std::path::Component::Normal(_))) {
                    components.pop();
                } else {
                    components.push(component);
                }
            }
            _ => components.push(component),
        }
    }

    components
        .into_iter()
        .fold(PathBuf::new(), |mut acc, component| {
            acc.push(component.as_os_str());
            acc
        })
}
