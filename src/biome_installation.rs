use reqwest::blocking::Client;
use std::{
    env,
    error::Error,
    fs::{self, File},
    io,
    path::{Path, PathBuf},
    process::Command,
};

pub(crate) struct BiomeInstallation;

impl BiomeInstallation {
    pub(crate) const VERSION: &'static str = "2.6.8";

    pub(crate) fn managed_path() -> Result<PathBuf, Box<dyn Error>> {
        let base = Self::data_directory()?;

        Ok(base
            .join("standardizer")
            .join("tools")
            .join("biome")
            .join(Self::VERSION)
            .join(Self::executable_name()))
    }

    pub(crate) fn install(force: bool) -> Result<PathBuf, Box<dyn Error>> {
        let destination = Self::managed_path()?;

        if destination.is_file() && !force && Self::verify(&destination)? {
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

        let url = Self::download_url()?;

        eprintln!(
            "standardizer: téléchargement de Biome {} pour {}",
            Self::VERSION,
            Self::platform_name()?
        );

        let client = Client::builder()
            .user_agent(format!(
                "standardizer/{}",
                env!("CARGO_PKG_VERSION")
            ))
            .build()?;

        let mut response = client
            .get(url)
            .send()?
            .error_for_status()?;

        let mut file = File::create(&temporary)?;
        io::copy(&mut response, &mut file)?;
        drop(file);

        Self::make_executable(&temporary)?;

        if !Self::verify(&temporary)? {
            let _ = fs::remove_file(&temporary);
            return Err(format!(
                "downloaded Biome does not report expected version {}",
                Self::VERSION
            )
            .into());
        }

        if destination.exists() {
            fs::remove_file(&destination)?;
        }

        fs::rename(&temporary, &destination)?;

        eprintln!(
            "standardizer: Biome {} installé dans {}",
            Self::VERSION,
            destination.display()
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
        }

        Ok(true)
    }

    pub(crate) fn verify(path: &Path) -> Result<bool, Box<dyn Error>> {
        let output = match Command::new(path).arg("--version").output() {
            Ok(output) => output,
            Err(error) if error.kind() == io::ErrorKind::NotFound => {
                return Ok(false);
            }
            Err(error) => return Err(error.into()),
        };

        if !output.status.success() {
            return Ok(false);
        }

        let stdout = String::from_utf8_lossy(&output.stdout);

        Ok(stdout.contains(Self::VERSION))
    }

    pub(crate) fn version(path: &Path) -> Result<Option<String>, Box<dyn Error>> {
        let output = match Command::new(path).arg("--version").output() {
            Ok(output) => output,
            Err(error) if error.kind() == io::ErrorKind::NotFound => {
                return Ok(None);
            }
            Err(error) => return Err(error.into()),
        };

        if !output.status.success() {
            return Ok(None);
        }

        let version = String::from_utf8_lossy(&output.stdout)
            .trim()
            .to_owned();

        if version.is_empty() {
            Ok(None)
        } else {
            Ok(Some(version))
        }
    }

    fn data_directory() -> Result<PathBuf, Box<dyn Error>> {
        #[cfg(target_os = "windows")]
        {
            let local = env::var_os("LOCALAPPDATA")
                .ok_or("LOCALAPPDATA is not defined")?;
            return Ok(PathBuf::from(local));
        }

        #[cfg(target_os = "macos")]
        {
            let home = env::var_os("HOME")
                .ok_or("HOME is not defined")?;
            return Ok(PathBuf::from(home)
                .join("Library")
                .join("Application Support"));
        }

        #[cfg(all(unix, not(target_os = "macos")))]
        {
            if let Some(xdg) = env::var_os("XDG_DATA_HOME") {
                return Ok(PathBuf::from(xdg));
            }

            let home = env::var_os("HOME")
                .ok_or("HOME is not defined")?;

            return Ok(PathBuf::from(home)
                .join(".local")
                .join("share"));
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
            ("linux", "x86_64") if cfg!(target_env = "musl") => {
                Ok("linux-x64-musl")
            }
            ("linux", "aarch64") if cfg!(target_env = "musl") => {
                Ok("linux-arm64-musl")
            }
            ("linux", "x86_64") => Ok("linux-x64"),
            ("linux", "aarch64") => Ok("linux-arm64"),
            _ => Err(format!(
                "Biome installation unsupported on {os}/{arch}"
            )
            .into()),
        }
    }

    fn asset_name() -> Result<String, Box<dyn Error>> {
        let platform = Self::platform_name()?;

        if cfg!(target_os = "windows") {
            Ok(format!("biome-{platform}.exe"))
        } else {
            Ok(format!("biome-{platform}"))
        }
    }

    fn download_url() -> Result<String, Box<dyn Error>> {
        let asset = Self::asset_name()?;

        Ok(format!(
            "https://github.com/biomejs/biome/releases/download/@biomejs/biome@{}/{asset}",
            Self::VERSION
        ))
    }

    fn executable_name() -> &'static str {
        if cfg!(target_os = "windows") {
            "biome.exe"
        } else {
            "biome"
        }
    }

    fn temporary_name() -> &'static str {
        if cfg!(target_os = "windows") {
            "biome.download.exe"
        } else {
            "biome.download"
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
