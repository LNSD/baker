use std::collections::BTreeMap;
use std::convert::Infallible;
use std::str::FromStr;

use serde::Deserializer;
use serde_with::formats::SpaceSeparator;
use serde_with::{serde_as, StringWithSeparator};

#[serde_as]
#[derive(Debug, Clone, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub struct ProjectConfig {
    /// The header of every kas configuration file. It contains information about
    /// the context of the file.
    pub header: Header,

    /// Defines the bitbake-based build system.
    ///
    /// Known build systems are `openembedded` (or `oe`) and `isar`. If set, this restricts the
    /// search of kas for the init script in the configured repositories to `oe-init-build-env` or
    /// `isar-init-build-env`, respectively. If `kas-container` finds this property in the
    /// top-level kas configuration file (includes are not evaluated), it will automatically
    /// select the required container image and invocation mode.
    #[serde(default)]
    pub build_system: Option<BuildSystem>,

    /// Contains the value of the `MACHINE` variable that is written into the
    /// `local.conf`. Can be overwritten by the `KAS_MACHINE` environment
    /// variable and defaults to `qemux86-64`.
    #[serde(default)]
    pub machine: Option<String>,

    /// Contains the value of the `DISTRO` variable that is written into the
    /// `local.conf`. Can be overwritten by the `KAS_DISTRO` environment
    /// variable and defaults to `poky`.
    #[serde(default)]
    pub distro: Option<String>,

    /// Contains the target or a list of targets to build by bitbake. Can be
    /// overwritten by the `KAS_TARGET` environment variable and defaults to
    /// `core-image-minimal`. Space is used as a delimiter if multiple targets
    /// should be specified via the environment variable.
    #[serde_as(as = "StringWithSeparator::<SpaceSeparator, String>")]
    #[serde(default)]
    pub target: Vec<String>,

    /// Contains environment variable names with either default values or None.
    /// These variables are made available to bitbake via `BB_ENV_EXTRAWHITE`
    /// and can be overwritten by the variables of the environment in which
    /// kas is started.
    /// Either a string or nothing (None) can be assigned as value.
    /// The former one serves as a default value whereas the latter one will lead
    /// to add the variable only to `BB_ENV_EXTRAWHITE` and not to the
    /// environment where kas is started.
    #[serde(default)]
    pub env: BTreeMap<String, Option<String>>,

    /// Contains the task to build by bitbake. Can be overwritten by the
    /// `KAS_TASK` environment variable and defaults to `build`.
    #[serde(default)]
    pub task: Option<String>,

    /// Contains the definitions of all available repos and layers.
    #[serde(default)]
    pub repos: BTreeMap<String, Option<Repo>>,
}

/// The header of every kas configuration file. It contains information about
/// the context of the file.
#[derive(Debug, Clone, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub struct Header {
    /// Lets kas check if it is compatible with this file. See the
    /// [configuration format changelog](https://kas.readthedocs.io/en/latest/format-changelog.html)
    /// for the format history and the latest available version.
    pub version: String,

    /// A list of configuration files this current file is based on. They are
    /// merged in order they are stated. So a latter one could overwrite
    /// settings from previous files. The current file can overwrite settings
    /// from every included file. An item in this list can have one of two types:
    #[serde(default)]
    pub includes: Vec<HeaderInclude>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub struct HeaderInclude {
    /// The id of the repository where the file is located. The repo
    /// needs to be defined in the `repos` dictionary as `<repo-id>`.
    pub repo: String,

    /// The path to the file, relative to the root of the specified
    /// repository.
    pub file: String,
}

impl FromStr for HeaderInclude {
    type Err = Infallible;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        Ok(Self {
            repo: "".to_string(),
            file: value.to_string(),
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum BuildSystem {
    /// OpenEmbedded, the build framework for embedded Linux.
    ///
    /// See: https://www.openembedded.org/
    OpenEmbedded,
    /// Integration System for Automated Root filesystem generation
    ///
    /// See: https://github.com/ilbers/isar
    Isar,
}

impl<'de> serde::de::Deserialize<'de> for BuildSystem {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        match String::deserialize(deserializer)?.to_lowercase().as_str() {
            "openembedded" | "oe" => Ok(Self::OpenEmbedded),
            "isar" => Ok(Self::Isar),
            _ => Err(serde::de::Error::custom(
                "invalid build_system, expected 'openembedded', 'oe' or 'isar'",
            )),
        }
    }
}

impl serde::Serialize for BuildSystem {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match self {
            BuildSystem::OpenEmbedded => Ok(serializer.serialize_str("openembedded")?),
            BuildSystem::Isar => Ok(serializer.serialize_str("isar")?),
        }
    }
}

/// Contains the definition of a repository and the layers, that should be
/// part of the build. If the value is `None`, the repository, where the
/// current configuration file is located is defined as `<repo-id>` and
/// added as a layer to the build.
///
/// It is recommended that the `<repo-id>` is related to the containing repository/layer to
/// ease cross-project referencing.
#[derive(Debug, Clone, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub struct Repo {
    /// Defines under which name the repository is stored. If its missing
    /// the `<repo-id>` will be used.
    pub name: Option<String>,

    /// The url of the repository. If this is missing, no version control
    /// operations are performed.
    #[serde(default)]
    pub url: Option<String>,

    /// The type of version control system. The default value is `git`
    ///: and `hg` is also supported.
    #[serde(default)]
    pub vcs: Option<RepoVcs>,

    /// The commit ID (branch names, no symbolic refs, no tags) that should be
    /// used. If `url` was specified but no `commit` and no `branch`, the
    /// revision you get depends on the defaults of the version control system
    /// used.
    #[serde(default)]
    pub commit: Option<String>,

    /// The upstream branch that should be tracked. If no `commit` was
    /// specified, the head of the upstream is checked out.
    #[serde(default)]
    pub branch: Option<String>,

    // TODO: Add documentation for `refspec`
    #[serde(default)]
    pub refspec: Option<String>,

    /// The path where the repository is stored.
    /// If the `url` and `path` is missing, the repository where the
    /// current configuration file is located is defined.
    /// If the `url` is missing and the path defined, this entry references
    /// the directory the path points to.
    /// If the `url` as well as the `path` is defined, the path is used to
    /// overwrite the checkout directory, that defaults to `kas_work_dir`
    /// + `repo.name`.
    /// In case of a relative path name `kas_work_dir` is prepended.
    #[serde(default)]
    pub path: Option<String>,

    /// Contains the layers from this repository that should be added to the
    /// `bblayers.conf`. If this is missing or `None` or an empty
    /// dictionary, the path to the repo itself is added as a layer.
    /// Additionally, `.` is a valid value if the repo itself should be added
    /// as a layer. This allows combinations:
    ///
    /// ```yaml
    ///
    /// repos:
    ///   meta-foo:
    ///     url: https://github.com/bar/meta-foo.git
    ///     path: layers/meta-foo
    ///     branch: master
    ///     layers:
    ///       .:
    ///       contrib:
    /// ```
    ///
    /// This adds both `layers/meta-foo` and `layers/meta-foo/contrib` from
    /// the `meta-foo` repository to `bblayers.conf`.
    // TODO:
    //    `<layer-path>`: enum [optional]
    //     Adds the layer with `<layer-path>` that is relative to the
    //     repository root directory, to the `bblayers.conf` if the value of
    //     this entry is not in this list: `['disabled', 'excluded', 'n', 'no',
    //     '0', 'false']`. This way it is possible to overwrite the inclusion
    //     of a layer in latter loaded configuration files.
    #[serde(default)]
    pub layers: BTreeMap<String, Option<String>>,

    ///  Contains the patches that should be applied to this repo before it is used.
    #[serde(default)]
    pub patches: BTreeMap<String, RepoPatch>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum RepoVcs {
    Git,
    Hg,
}

impl<'de> serde::de::Deserialize<'de> for RepoVcs {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        match String::deserialize(deserializer)?.to_lowercase().as_str() {
            "git" => Ok(Self::Git),
            "hg" => Ok(Self::Hg),
            _ => Err(serde::de::Error::custom(
                "invalid repo_type, expected 'git' or 'hg'",
            )),
        }
    }
}

impl serde::Serialize for RepoVcs {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match self {
            RepoVcs::Git => Ok(serializer.serialize_str("git")?),
            RepoVcs::Hg => Ok(serializer.serialize_str("hg")?),
        }
    }
}

///  One entry in patches with its specific and unique id. All available
///  patch entries are applied in the order of their sorted
///  `<patches-id>`.
#[derive(Debug, Clone, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub struct RepoPatch {
    /// The identifier of the repo where the path of this entry is relative to.
    pub repo: String,

    /// The path to one patch file or a quilt formatted patchset directory.
    pub path: String,
}
