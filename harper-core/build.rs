use std::{env, fs, path::PathBuf};

fn main() {
    let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
    let weir_rule_dir = manifest_dir.join("./src/linting/weir_rules");
    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
    let dest = out_dir.join("weir_rules_generated_list.rs");

    let mut files: Vec<PathBuf> = fs::read_dir(&weir_rule_dir)
        .unwrap()
        .into_iter()
        .filter_map(Result::ok)
        .filter(|e| e.file_type().unwrap().is_file())
        .map(|e| e.path().to_path_buf())
        .collect();

    files.sort();

    let mut code = String::new();

    code.push_str("generate_boilerplate!{[");

    for file in files {
        if file
            .file_name()
            .unwrap()
            .to_string_lossy()
            .ends_with(".weir")
        {
            code.push_str(&format!(
                "{},\n",
                file.file_stem().unwrap().to_str().unwrap()
            ));
        }
    }

    code.push_str("]}");

    fs::write(&dest, code).unwrap();

    println!("cargo:rerun-if-changed={}", weir_rule_dir.display());
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rustc-env=WEIR_RULE_DIR={}", weir_rule_dir.display());
    println!("cargo:rustc-env=WEIR_RULE_LIST={}", dest.display());
}
