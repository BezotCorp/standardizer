use crate::tool_configuration;
use reqwest::blocking::Client;
use semver::Version;
use standardizer_common::tools::Tool;
use std::{
    env,
    error::Error,
    fs::{self, File},
    io,
    path::{Path, PathBuf},
    process::{self, Command},
};

pub(crate) struct BiomeInstallation;

impl BiomeInstallation {
    pub(crate) fn minimum_version() -> Result<Version, Box<dyn Error>> {
        let configuration = tool_configuration::load()?;

        let definition = configuration
            .definition(Tool::Biome)
            .ok_or("missing tool definition for biome")?;

        Ok(Version::parse(definition.minimum_version())?)
    }

    pub(crate) fn managed_version() -> Result<Version, Box<dyn Error>> {
        let configuration = tool_configuration::load()?;

        let definition = configuration
            .definition(Tool::Biome)
            .ok_or("missing tool definition for biome")?;

        Ok(Version::parse(definition.managed_version())?)
    }

    pub(crate) fn managed_path() -> Result<PathBuf, Box<dyn Error>> {
        let base = Self::data_directory()?;
        let version = Self::managed_version()?;

        Ok(base
            .join("standardizer")
            .join("tools")
            .join("biome")
            .join(version.to_string())
            .join(Self::executable_name()))
    }

    pub(crate) fn install(force: bool) -> Result<PathBuf, Box<dyn Error>> {
        let destination = Self::managed_path()?;

        let version = Self::managed_version()?;

        if destination.is_file() && !force && Self::is_expected_version(&destination)? {
            return Ok(destination);
        }

        let parent = destination
            .parent()
            .ok_or("invalid managed Biome destination")?;

        fs::create_dir_all(parent)?;

        let temporary = parent.join(Self::temporary_name());

        if temporary.exists() {
            fs::remove_file(&temporary)?;
        }

        let url = Self::download_url(&version)?;

        let platform = Self::platform_name()?;

        eprintln!(
            "standardizer: téléchargement de Biome {} pour {}",
            version, platform,
        );

        let client = Client::builder()
            .user_agent(format!("standardizer/{}", env!("CARGO_PKG_VERSION"),))
            .build()?;

        let mut response = client.get(url).send()?.error_for_status()?;

        let mut file = File::create(&temporary)?;

        let copy_result = io::copy(&mut response, &mut file);

        drop(file);

        if let Err(error) = copy_result {
            let _ = fs::remove_file(&temporary);

            return Err(error.into());
        }

        Self::make_executable(&temporary)?;

        if !Self::is_expected_version(&temporary)? {
            let _ = fs::remove_file(&temporary);

            return Err(format!(
                "downloaded Biome does not report expected version {}",
                version,
            )
            .into());
        }

        if destination.exists() {
            fs::remove_file(&destination)?;
        }

        if let Err(error) = fs::rename(&temporary, &destination) {
            let _ = fs::remove_file(&temporary);

            return Err(error.into());
        }

        eprintln!(
            "standardizer: Biome {} installé dans {}",
            version,
            destination.display(),
        );

        Ok(destination)
    }

    pub(crate) fn uninstall() -> Result<bool, Box<dyn Error>> {
        let path = Self::managed_path()?;

        if !path.exists() {
            return Ok(false);
        }

        fs::remove_file(&path)?;

        if let Some(version_directory) = path.parent() {
            let _ = fs::remove_dir(version_directory);

            if let Some(biome_directory) = version_directory.parent() {
                let _ = fs::remove_dir(biome_directory);
            }
        }

        Ok(true)
    }

    pub(crate) fn version(path: &Path) -> Result<Option<Version>, Box<dyn Error>> {
        let output = match Command::new(path).arg("--version").output() {
            Ok(output) => output,

            Err(error) if error.kind() == io::ErrorKind::NotFound => {
                return Ok(None);
            }

            Err(error) => {
                return Err(error.into());
            }
        };

        if !output.status.success() {
            return Ok(None);
        }

        let stdout = String::from_utf8_lossy(&output.stdout);

        Ok(Self::parse_version(&stdout))
    }

    pub(crate) fn is_compatible(path: &Path) -> Result<bool, Box<dyn Error>> {
        let Some(version) = Self::version(path)? else {
            return Ok(false);
        };

        let minimum = Self::minimum_version()?;

        Ok(version >= minimum)
    }

    pub(crate) fn is_expected_version(path: &Path) -> Result<bool, Box<dyn Error>> {
        let Some(version) = Self::version(path)? else {
            return Ok(false);
        };

        let expected = Self::managed_version()?;

        Ok(version == expected)
    }

    fn parse_version(output: &str) -> Option<Version> {
        output.split_whitespace().find_map(|token| {
            let token = token.trim_matches(|character: char| {
                !character.is_ascii_alphanumeric()
                    && character != '.'
                    && character != '-'
                    && character != '+'
            });

            let token = token.strip_prefix('v').unwrap_or(token);

            Version::parse(token).ok()
        })
    }

    fn data_directory() -> Result<PathBuf, Box<dyn Error>> {
        #[cfg(target_os = "windows")]
        {
            let local = env::var_os("LOCALAPPDATA").ok_or("LOCALAPPDATA is not defined")?;

            return Ok(PathBuf::from(local));
        }

        #[cfg(target_os = "macos")]
        {
            let home = env::var_os("HOME").ok_or("HOME is not defined")?;

            return Ok(PathBuf::from(home)
                .join("Library")
                .join("Application Support"));
        }

        #[cfg(all(unix, not(target_os = "macos")))]
        {
            if let Some(xdg) = env::var_os("XDG_DATA_HOME") {
                return Ok(PathBuf::from(xdg));
            }

            let home = env::var_os("HOME").ok_or("HOME is not defined")?;

            return Ok(PathBuf::from(home).join(".local").join("share"));
        }

        #[allow(unreachable_code)]
        Err("unsupported operating system".into())
    }

    fn platform_name() -> Result<&'static str, Box<dyn Error>> {
        let os = env::consts::OS;

        let arch = env::consts::ARCH;

        match (os, arch) {
            ("windows", "x86_64") => Ok("win32-x64"),

            ("windows", "aarch64") => Ok("win32-arm64"),

            ("macos", "x86_64") => Ok("darwin-x64"),

            ("macos", "aarch64") => Ok("darwin-arm64"),

            ("linux", "x86_64") if cfg!(target_env = "musl") => Ok("linux-x64-musl"),

            ("linux", "aarch64") if cfg!(target_env = "musl") => Ok("linux-arm64-musl"),

            ("linux", "x86_64") => Ok("linux-x64"),

            ("linux", "aarch64") => Ok("linux-arm64"),

            _ => Err(format!("Biome installation unsupported on {os}/{arch}",).into()),
        }
    }

    fn asset_name() -> Result<String, Box<dyn Error>> {
        let platform = Self::platform_name()?;

        if cfg!(target_os = "windows") {
            Ok(format!("biome-{platform}.exe",))
        } else {
            Ok(format!("biome-{platform}",))
        }
    }

    fn download_url(version: &Version) -> Result<String, Box<dyn Error>> {
        let asset = Self::asset_name()?;

        Ok(format!(
            "https://github.com/biomejs/biome/releases/download/@biomejs/biome@{version}/{asset}",
        ))
    }

    fn executable_name() -> &'static str {
        if cfg!(target_os = "windows") {
            "biome.exe"
        } else {
            "biome"
        }
    }

    fn temporary_name() -> String {
        if cfg!(target_os = "windows") {
            format!("biome.{}.download.exe", process::id(),)
        } else {
            format!("biome.{}.download", process::id(),)
        }
    }

    #[cfg(unix)]
    fn make_executable(path: &Path) -> Result<(), Box<dyn Error>> {
        use std::os::unix::fs::PermissionsExt;

        let mut permissions = fs::metadata(path)?.permissions();

        permissions.set_mode(0o755);

        fs::set_permissions(path, permissions)?;

        Ok(())
    }

    #[cfg(not(unix))]
    fn make_executable(_path: &Path) -> Result<(), Box<dyn Error>> {
        Ok(())
    }
}
