use std::{
    borrow::Cow,
    collections::HashSet,
    fs::File,
    io,
    path::{Path, PathBuf},
    str::FromStr
};

use anyhow::{anyhow, bail, Context, Error};
use blaze_core::{
    common::{parallelism::Parallelism, selector::ProjectSelector},
    run, GlobalOptions, RunOptions, SelectorSource,
};
use flate2::{read::GzDecoder, write::GzEncoder, Compression};
use fs4::{lock_contended_error, FileExt};
use possibly::possibly;
use serde::Deserialize;

type Result<T> = std::result::Result<T, anyhow::Error>;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct CacheOptions {
    key: String,
    storage: PathBuf,
    #[serde(default)]
    extra_dirs: HashSet<PathBuf>,
}

const ROOT: &str = "ROOT";
const PARALLELISM: &str = "PARALLELISM";
const SELECTOR: &str = "SELECTOR";
const TARGETS: &str = "TARGETS";
const VARIABLES: &str = "VARIABLES";
const CACHE_ENABLED: &str = "CACHE_ENABLED";
const CACHE_STORAGE: &str = "CACHE_STORAGE";
const CACHE_KEY: &str = "CACHE_KEY";
const CACHE_EXTRA_DIRS: &str = "CACHE_EXTRA_DIRS";

const UTF8_ERROR: &str = "parameter is not valid utf-8";

fn main() -> Result<()> {
    let root = match std::env::var_os(ROOT) {
        Some(root) => PathBuf::from(root),
        None => std::env::current_dir()?,
    };

    println!("workspace root: {}", root.display());

    let selector = match std::env::var(SELECTOR) {
        Ok(selector) => Some(
            serde_json::from_str::<ProjectSelector>(&selector)
                .context("could not deserialize selector")?,
        ),
        Err(std::env::VarError::NotPresent) => None,
        Err(_) => bail!("could not read {SELECTOR} variable"),
    };

    match &selector {
        Some(selector) => println!("using selector: {selector}"),
        None => println!("using default selector"),
    };

    let parallelism = std::env::var_os(PARALLELISM)
        .map(|p| {
            Ok::<_, Error>(serde_json::from_str::<Parallelism>(
                &p.to_str().ok_or_else(|| anyhow!(UTF8_ERROR))?,
            )?)
        })
        .transpose()
        .context("could not deserialize parallelism level")?
        .unwrap_or_default();

    println!("parallelism: {parallelism}");

    let targets = std::env::var(TARGETS)?;

    println!("targets: {targets:?}");

    let variables = std::env::var_os(VARIABLES)
        .map(|s| {
            s.to_str()
                .map(str::to_owned)
                .ok_or_else(|| anyhow!("could not read {VARIABLES} variable"))
        })
        .transpose()?;

    match variables {
        Some(v) => println!("variables overrides: {v}"),
        None => println!("no variables provided"),
    };

    let cache_enabled = std::env::var_os(CACHE_ENABLED)
        .map(|c| {
            Ok::<_, Error>(bool::from_str(
                &c.to_str().ok_or_else(|| anyhow!(UTF8_ERROR))?,
            )?)
        })
        .transpose()?
        .unwrap_or_default();

    let cache = cache_enabled
        .then(|| -> Result<_> {
            Ok(CacheOptions {
                extra_dirs: std::env::var_os(CACHE_EXTRA_DIRS)
                    .map(|dirs| -> Result<_> {
                        Ok(serde_json::from_str(
                            &dirs.to_str().ok_or_else(|| anyhow!(UTF8_ERROR))?,
                        )?)
                    })
                    .transpose()
                    .context("invalid cache extra dirs array")?
                    .unwrap_or_default(),
                key: std::env::var(CACHE_KEY).context("missing or invalid cache key")?,
                storage: std::env::var(CACHE_STORAGE)
                    .map(PathBuf::from)
                    .context("missing or invalid cache storage")?,
            })
        })
        .transpose()?;

    match &cache {
        Some(c) => println!("using cache options: {c:?}"),
        None => println!("no cache used"),
    };

    let mut lockfile: Option<File> = None;

    if let Some(cache) = &cache {
        let _ = lockfile.insert(restore_cache(cache)?);
    }

    let mut failed_projects: HashSet<String> = HashSet::new();

    for target in targets.split(',') {
        println!("running target \"{target}\"");

        let mut global_options = GlobalOptions::new();

        if cache.is_none() {
            global_options = global_options.without_cache();
        }

        let results = run(
            &root,
            RunOptions::new(target)
                .with_selector_source(SelectorSource::Provided(ProjectSelector::include_exclude(
                    [".+"],
                    failed_projects.iter().map(|s| s.as_str()),
                )))
                .with_parallelism(parallelism)
                .displaying_graph(),
            global_options,
        )?;

        failed_projects.extend(
            results
                .root_executions()
                .values()
                .filter_map(|execution_result| {
                    possibly!(
                        execution_result.result,
                        None|Some(Err(_)) =>
                            execution_result.execution.get_project().name().to_owned()
                    )
                }),
        );
    }

    if let Some(cache) = &cache {
        store_cache(&root, lockfile.unwrap(), cache)?;
    }

    if !failed_projects.is_empty() {
        bail!("CI failed for projects: {failed_projects:?}");
    }

    Ok(())
}

fn get_archive_path(options: &CacheOptions) -> PathBuf {
    options.storage.join(format!("{}.tar.gz", options.key))
}

fn get_lockfile_path(options: &CacheOptions) -> PathBuf {
    options.storage.join(format!("{}.lock", options.key))
}

fn restore_cache(options: &CacheOptions) -> Result<File> {
    let archive_path = get_archive_path(options);
    let lockfile_path = get_lockfile_path(options);
    let lockfile = File::create(lockfile_path)?;

    if let Err(err) = lockfile.try_lock_exclusive(){
        if err.kind() != lock_contended_error().kind(){
            bail!("lockfile error: {err}")
        }
        println!("waiting for lock to release for cache key {}...", options.key);
        lockfile.lock_exclusive()?;
    }

    println!("reading cache at {}", archive_path.display());
    let mut archive = match File::open(&archive_path) {
        Ok(file) => {
            let mut archive = tar::Archive::new(GzDecoder::new(file));
            archive.set_preserve_mtime(true);
            archive.set_preserve_ownerships(true);
            archive.set_preserve_permissions(true);
            archive.set_overwrite(true);
            tar::Archive::from(archive)
        }
        Err(not_found) if not_found.kind() == std::io::ErrorKind::NotFound => {
            eprintln!("no cache archive was found at {}", archive_path.display());
            return Ok(lockfile);
        }
        Err(error) => return Err(error.into()),
    };

    println!("restoring cache from {}...", archive_path.display());

    archive.unpack("/")?;

    Ok(lockfile)
}

fn store_cache(root: &Path, lockfile: File, options: &CacheOptions) -> Result<()> {
    let globals = blaze_core::WorkspaceGlobals::new(root, GlobalOptions::default())?;

    let project_cached_folders = globals
        .workspace_handle()
        .inner()
        .projects()
        .values()
        .map(|reference| {
            let project_root = root.join(&reference.path());
            let cache_metadata_path = project_root.join(".cache.json");
            let cache_metadata_file = match File::open(cache_metadata_path) {
                Ok(file) => file,
                Err(err) if err.kind() == std::io::ErrorKind::NotFound => {
                    return Ok(HashSet::default())
                }
                Err(err) => bail!(err),
            };
            let cached: HashSet<PathBuf> = serde_json::from_reader(cache_metadata_file)?;
            Ok(cached
                .into_iter()
                .map(|relative_path| project_root.join(relative_path))
                .collect::<HashSet<_>>())
        })
        .collect::<Result<Vec<_>>>()?
        .into_iter()
        .flatten()
        .collect::<HashSet<_>>();

    let archive_path = get_archive_path(options);
    println!("building cache at {}", archive_path.display());
    let output = File::create(&archive_path)?;
    let encoder = GzEncoder::new(output, Compression::default());
    let mut tar = tar::Builder::new(encoder);
    tar.follow_symlinks(false);

    for source_folder in project_cached_folders
        .iter()
        .chain(options.extra_dirs.iter())
    {
        let mut source_path = Cow::Borrowed(source_folder);

        if source_path.is_relative() {
            source_path = Cow::Owned(root.join(&*source_path));
        }

        match std::fs::metadata(&*source_path) {
            Ok(metadata) if metadata.is_dir() => {}
            Ok(_) => bail!(
                "only directory can be cached ({} is not a directory)",
                source_path.display()
            ),
            Err(err) if err.kind() == io::ErrorKind::NotFound => {
                eprintln!(
                    "folder {} does not exist. ignoring...",
                    source_path.display()
                );
                continue;
            }
            Err(err) => return Err(err.into()),
        };

        let inner_path = source_path.strip_prefix("/")?;
        println!(
            "caching folder {} in archive path {}...",
            source_path.display(),
            inner_path.display()
        );
        tar.append_dir_all(&inner_path, &*source_path)?;
    }
    println!("finalizing cache archive...");
    tar.finish()?;
    println!("releasing cache lockfile...");
    lockfile.unlock()?;
    Ok(())
}
