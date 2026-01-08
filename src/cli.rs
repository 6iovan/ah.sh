use std::{
    collections::{HashMap, HashSet},
    path::Path,
};

use clap::{Parser, command};
use serde_json::from_str;

use crate::{command::exec_nix_develop, env};

#[derive(Parser)]
#[command(version, about, long_about = None)]
pub struct Cli {
    pub language: Vec<String>,
}

pub fn languages() {
    let cli = Cli::parse();

    let supported = supported_langs_of_devenv();
    let ensures = ensure_languages(cli.language, supported).unwrap_or_else(|e| {
        eprintln!("{e}");
        std::process::exit(1);
    });

    let pkgs = flatten_pkgs(&ensures, query_pkgs_of_supported_langs());

    let env_ahsh_languages = serde_json::to_string(&ensures).unwrap();
    let env_ahsh_packages = serde_json::to_string(&pkgs).unwrap();

    exec_nix_develop(env_ahsh_languages, env_ahsh_packages);
}

fn supported_langs_of_devenv() -> Vec<String> {
    Path::new(env::AHSH_DEVENV_SRC)
        .join("src")
        .join("modules")
        .join("languages")
        .read_dir()
        .ok()
        .unwrap()
        .filter_map(Result::ok)
        .map(|x| {
            if x.file_type().unwrap().is_file() && x.path().extension().unwrap() == "nix" {
                x.path().file_stem().unwrap().to_str().unwrap().to_owned()
            } else {
                x.path().file_name().unwrap().to_str().unwrap().to_owned()
            }
        })
        .collect::<Vec<_>>()
}

fn ensure_languages(
    langs: Vec<String>,
    supported_langs: Vec<String>,
) -> Result<Vec<String>, String> {
    let supported: HashSet<String> = supported_langs.into_iter().collect();

    let invalids: Vec<String> = langs
        .iter()
        .filter(|&l| !supported.contains(l))
        .cloned()
        .collect();

    if invalids.is_empty() {
        Ok(langs)
    } else {
        Err(format!("Languages {:?} are not supported", invalids))
    }
}

fn query_pkgs_of_supported_langs() -> HashMap<String, Vec<String>> {
    let json_str = include_str!("./assets/lang_pkgs.json");
    from_str(json_str).expect("Internal error")
}

fn flatten_pkgs(ensures: &Vec<String>, pkgs: HashMap<String, Vec<String>>) -> Vec<String> {
    let merged_pkgs: Vec<String> = ensures
        .iter()
        .filter_map(|x| pkgs.get(x))
        .flatten()
        .cloned()
        .collect();
    merged_pkgs
}
