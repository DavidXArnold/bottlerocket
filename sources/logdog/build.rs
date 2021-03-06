// Automatically generate README.md from rustdoc and generate variant symlink

use std::fs::File;
use std::io::Write;
use std::os::unix::fs::symlink;
use std::path::{Path, PathBuf};
use std::{env, fs, io, process};

// The VARIANT variable is originally BUILDSYS_VARIANT, set in the top-level Makefile.toml,
// and is passed through as VARIANT by the top-level Dockerfile.  It represents which OS
// variant we're building, and therefore which API model to use.
const ENV_VARIANT: &str = "VARIANT";

/// Creates a file, `conf/current/logdog.conf` which is a symlink to a file with `logdog` commands
/// for the current variant. Whatever the value of the `VARIANT` environment variable is, this
/// function requires a file at `conf/logdog.$VARIANT.conf` and points to it from the `logdog.conf`
/// symlink. For example, if the variant is `aws-k8s-1.17` then `conf/current/logdog.conf` will
/// point to `conf/logdog.aws-k8s-1.17.conf`.
fn symlink_variant() {
    println!("cargo:rerun-if-env-changed={}", ENV_VARIANT);
    let variant = env::var(ENV_VARIANT).unwrap_or_else(|_| {
        eprintln!(
            "For local builds, you must set the {} environment variable so we know which logdog \
            commands to build. Valid values are the directories in models/src/variants/, for \
            example 'aws-k8s-1.17'.",
            ENV_VARIANT
        );
        process::exit(1);
    });
    let variant_filename = format!("logdog.{}.conf", variant);
    if !PathBuf::from("conf").join(&variant_filename).is_file() {
        eprintln!(
            "There is no file named '{}' in the 'conf' directory for the current variant (given \
            by the '{}' environment variable) Each variant must have a file representing the \
            variant-specific commands that logdog will run.",
            variant, ENV_VARIANT
        );
        process::exit(1);
    }
    // create the symlink from conf/current/logdog.conf to the variant-specific file
    let target = format!("../{}", variant_filename);
    let link = "conf/current/logdog.conf";
    symlink_force(&target, &link).unwrap_or_else(|e| {
        eprintln!(
            "Failed to create symlink at '{}' pointing to '{}' - we need this to \
            support different logdog commands for variants.  Error: {}",
            link, target, e
        );
        process::exit(1);
    });
}

fn symlink_force<P1, P2>(target: P1, link: P2) -> io::Result<()>
where
    P1: AsRef<Path>,
    P2: AsRef<Path>,
{
    // Remove link if it already exists
    if let Err(e) = fs::remove_file(&link) {
        if e.kind() != io::ErrorKind::NotFound {
            return Err(e);
        }
    }
    // Link to requested target
    symlink(&target, &link)
}

fn generate_readme() {
    // Check for environment variable "SKIP_README". If it is set,
    // skip README generation
    if env::var_os("SKIP_README").is_some() {
        return;
    }

    let mut source = File::open("src/main.rs").unwrap();
    let mut template = File::open("README.tpl").unwrap();

    let content = cargo_readme::generate_readme(
        &PathBuf::from("."), // root
        &mut source,         // source
        Some(&mut template), // template
        // The "add x" arguments don't apply when using a template.
        true,  // add title
        false, // add badges
        false, // add license
        true,  // indent headings
    )
    .unwrap();

    let mut readme = File::create("README.md").unwrap();
    readme.write_all(content.as_bytes()).unwrap();
}

fn main() {
    symlink_variant();
    generate_readme();
}
