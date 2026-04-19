use std::{env, fs, path::PathBuf};

fn collect_weir_files(dir: &PathBuf) -> Vec<PathBuf> {
    match fs::read_dir(dir) {
        Ok(entries) => {
            let mut files: Vec<PathBuf> = entries
                .filter_map(Result::ok)
                .filter(|e| e.file_type().unwrap().is_file())
                .map(|e| e.path())
                .filter(|p| p.extension().map(|e| e == "weir").unwrap_or(false))
                .collect();
            files.sort();
            files
        }
        Err(_) => vec![],
    }
}

fn write_boilerplate(files: &[PathBuf], dest: &PathBuf) {
    let mut code = String::new();
    code.push_str("generate_boilerplate!{[");
    for file in files {
        code.push_str(&format!(
            "{},\n",
            file.file_stem().unwrap().to_str().unwrap()
        ));
    }
    code.push_str("]}");
    fs::write(dest, code).unwrap();
}

fn main() {
    let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
    let weir_rule_dir = manifest_dir.join("./src/linting/weir_rules");
    let german_weir_rule_dir = manifest_dir.join("./src/linting/weir_rules/de");
    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());

    let dest = out_dir.join("weir_rules_generated_list.rs");
    let de_dest = out_dir.join("german_weir_rules_generated_list.rs");

    let files = collect_weir_files(&weir_rule_dir);
    write_boilerplate(&files, &dest);

    let de_files = collect_weir_files(&german_weir_rule_dir);
    write_boilerplate(&de_files, &de_dest);

    println!("cargo:rerun-if-changed={}", weir_rule_dir.display());
    println!("cargo:rerun-if-changed={}", german_weir_rule_dir.display());
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rustc-env=WEIR_RULE_DIR={}", weir_rule_dir.display());
    println!(
        "cargo:rustc-env=GERMAN_WEIR_RULE_DIR={}",
        german_weir_rule_dir.display()
    );
    println!("cargo:rustc-env=WEIR_RULE_LIST={}", dest.display());
    println!(
        "cargo:rustc-env=GERMAN_WEIR_RULE_LIST={}",
        de_dest.display()
    );
}
