use std::{
    borrow::Cow,
    collections::HashSet,
    fs::File,
    io,
    path::{Path, PathBuf},
    str::FromStr,
};

use anyhow::{anyhow, bail, Context, Error};
use blaze_core::{
    common::{
        configuration_file::ConfigurationFileFormat, parallelism::Parallelism,
        selector::ProjectSelector, value::Value, variables::VariablesOverride,
    },
    run, GlobalOptions, RunOptions, SelectorSource,
};
use flate2::{read::GzDecoder, write::GzEncoder, Compression};
use fs4::{lock_contended_error, FileExt};
use possibly::possibly;
use serde::Deserialize;

type Result<T> = std::result::Result<T, anyhow::Error>;

const CACHE_ENABLED: &str = "CACHE_ENABLED";
const CACHE_STORAGE: &str = "CACHE_STORAGE";
const CACHE_KEY: &str = "CACHE_KEY";
const CACHE_EXTRA_DIRS: &str = "CACHE_EXTRA_DIRS";
const CACHE_IGNORE_EXISTING: &str = "CACHE_IGNORE_EXISTING";

#[derive(Debug)]
struct CacheOptions {
    key: String,
    storage: PathBuf,
    extra_dirs: HashSet<PathBuf>,
    ignore_existing: bool,
}

impl CacheOptions {
    fn from_env() -> Result<Option<Self>> {
        let cache_enabled = std::env::var_os(CACHE_ENABLED)
            .map(|c| {
                Ok::<_, Error>(bool::from_str(
                    c.to_str().ok_or_else(|| anyhow!(UTF8_ERROR))?,
                )?)
            })
            .transpose()?
            .unwrap_or_default();

        cache_enabled
            .then(|| -> Result<_> {
                Ok(CacheOptions {
                    extra_dirs: std::env::var_os(CACHE_EXTRA_DIRS)
                        .map(|dirs| -> Result<_> {
                            Ok(dirs
                                .to_str()
                                .ok_or_else(|| anyhow!(UTF8_ERROR))?
                                .split(',')
                                .map(PathBuf::from)
                                .collect())
                        })
                        .transpose()
                        .context("invalid cache extra dirs array")?
                        .unwrap_or_default(),
                    key: std::env::var(CACHE_KEY).context("missing or invalid cache key")?,
                    storage: std::env::var(CACHE_STORAGE)
                        .map(PathBuf::from)
                        .context("missing or invalid cache storage")?,
                    ignore_existing: std::env::var_os(CACHE_IGNORE_EXISTING)
                        .map(|c| {
                            Ok::<_, Error>(bool::from_str(
                                c.to_str().ok_or_else(|| anyhow!(UTF8_ERROR))?,
                            )?)
                        })
                        .transpose()?
                        .unwrap_or_default(),
                })
            })
            .transpose()
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct PipelineParameters {
    parallelism: Option<Parallelism>,
    selector: Option<ProjectSelector>,
    named_selector: Option<String>,
    variables: Option<Value>,
    target: String,
}

const PARALLELISM: &str = "PARALLELISM";
const SELECTOR: &str = "SELECTOR";
const NAMED_SELECTOR: &str = "NAMED_SELECTOR";
const TARGET: &str = "TARGET";
const VARIABLES: &str = "VARIABLES";

impl PipelineParameters {
    fn from_env() -> Result<Self> {
        let selector = match std::env::var(SELECTOR) {
            Ok(selector) => Some(
                serde_json::from_str::<ProjectSelector>(&selector)
                    .context("could not deserialize selector")?,
            ),
            Err(std::env::VarError::NotPresent) => None,
            Err(_) => bail!("could not read {SELECTOR} variable"),
        };

        let named_selector = match std::env::var(NAMED_SELECTOR) {
            Ok(name) => Some(name),
            Err(std::env::VarError::NotPresent) => None,
            Err(_) => bail!("could not read {NAMED_SELECTOR} variable"),
        };

        let parallelism = std::env::var_os(PARALLELISM)
            .map(|p| {
                Ok::<_, Error>(serde_json::from_str::<Parallelism>(
                    p.to_str().ok_or_else(|| anyhow!(UTF8_ERROR))?,
                )?)
            })
            .transpose()
            .context("could not deserialize parallelism level")?;

        let target = std::env::var(TARGET).context(format!("could not get {TARGET}"))?;

        let variables = std::env::var_os(VARIABLES)
            .map(|s| {
                serde_json::from_str(
                    &s.to_str()
                        .map(str::to_owned)
                        .ok_or_else(|| anyhow!("could not read {VARIABLES} variable"))?,
                )
                .context("cannot deserialize variables")
            })
            .transpose()?;

        Ok(Self {
            named_selector,
            selector,
            parallelism,
            target,
            variables,
        })
    }
}

const UTF8_ERROR: &str = "parameter is not valid utf-8";

enum ScmEvent {
    Push,
    PullRequest,
}

enum PipelineTrigger {
    #[allow(unused)]
    Scm {
        branch: String,
        event: ScmEvent,
    },
    Manual,
}

impl PipelineTrigger {
    pub fn get() -> Result<Option<Self>> {
        let get_branch = || std::env::var("DRONE_BRANCH");
        Ok(match std::env::var("DRONE_BUILD_EVENT")?.as_str() {
            "push" => Some(Self::Scm {
                branch: get_branch()?,
                event: ScmEvent::Push,
            }),
            "pull_request" => Some(Self::Scm {
                branch: get_branch()?,
                event: ScmEvent::PullRequest,
            }),
            "custom" => Some(Self::Manual),
            _ => None,
        })
    }
}

fn get_root() -> Result<PathBuf> {
    std::env::var("DRONE_WORKSPACE")
        .map(|p| Ok::<_, anyhow::Error>(PathBuf::from(p)))
        .unwrap_or_else(|_| Ok(std::env::current_dir()?))
}

fn main() -> Result<()> {
    let trigger = PipelineTrigger::get()
        .context("could not get pipeline trigger event type")?
        .ok_or_else(|| anyhow!("pipeline trigger event type is not supported"))?;

    let root = get_root().context("could not get workspace root")?;

    let params = match trigger {
        PipelineTrigger::Scm { .. } => {
            serde_json::from_reader(File::open(root.join("ci/scm-builds/parameters.json"))?)?
        }
        PipelineTrigger::Manual => PipelineParameters::from_env()?,
    };

    let cache = CacheOptions::from_env()?;

    let mut lockfile = None::<File>;

    if let Some(cache) = &cache {
        let _ = lockfile.insert(restore_cache(cache)?);
    }

    let mut failed_projects: HashSet<String> = HashSet::new();

    println!("running target \"{}\"", params.target);

    let mut global_options = GlobalOptions::new();

    if cache.is_none() {
        global_options = global_options.without_cache();
    }

    if let Some(variables) = params.variables {
        global_options = global_options.with_variable_overrides([VariablesOverride::Code {
            format: ConfigurationFileFormat::Json,
            code: serde_json::to_string(&variables)?,
        }])
    }

    let mut run_options = RunOptions::new(&params.target).displaying_graph();

    if let Some(parallelism) = params.parallelism {
        run_options = run_options.with_parallelism(parallelism);
    }

    if params.named_selector.is_some() && params.selector.is_some() {
        bail!("cannot provide both selector and named selector")
    }

    if let Some(name) = params.named_selector {
        run_options = run_options.with_selector_source(SelectorSource::Named(name));
    }

    if let Some(selector) = params.selector {
        run_options = run_options.with_selector_source(SelectorSource::Provided(selector));
    }

    let results = run(&root, run_options, global_options)?;

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

    if let Err(err) = lockfile.try_lock_exclusive() {
        if err.kind() != lock_contended_error().kind() {
            bail!("lockfile error: {err}")
        }
        println!(
            "waiting for lock to release for cache key {}...",
            options.key
        );
        lockfile.lock_exclusive()?;
    }

    println!("reading cache at {}", archive_path.display());
    let mut archive = match File::open(&archive_path) {
        Ok(file) => {
            if options.ignore_existing {
                println!(
                    "cache archive exists at {} but will be ignored",
                    archive_path.display()
                );
                return Ok(lockfile);
            }

            let mut archive = tar::Archive::new(GzDecoder::new(file));
            archive.set_preserve_mtime(true);
            archive.set_preserve_ownerships(true);
            archive.set_preserve_permissions(true);
            archive.set_overwrite(true);
            archive
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
            let project_root = root.join(reference.path());
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
        tar.append_dir_all(inner_path, &*source_path)?;
    }
    println!("finalizing cache archive...");
    tar.finish()?;
    println!("releasing cache lockfile...");
    lockfile.unlock()?;
    Ok(())
}
