use std::{env, fs, path::Path};

#[derive(Debug)]
struct StandaloneRule {
    name: String,
    relative_path: String,
}

#[derive(Debug)]
struct GroupedRule {
    public_name: String,
    children: Vec<StandaloneRule>,
}

/// Convert a Weir rule path to an `include_str!`-friendly relative path.
fn path_as_weir_relative(path: &Path, root: &Path) -> String {
    path.strip_prefix(root)
        .unwrap()
        .to_string_lossy()
        .replace('\\', "/")
}

/// Top-level `RuleName.weir` files are public as `RuleName`.
fn rule_name_from_path(path: &Path) -> String {
    path.file_stem().unwrap().to_string_lossy().to_string()
}

/// Grouped child rules use their relative path as a private name.
fn rule_name_from_relative_path(path: &Path, root: &Path) -> String {
    let mut relative = path_as_weir_relative(path, root);
    relative.truncate(relative.len() - ".weir".len());
    relative
}

/// Recursively collect child `.weir` files for a grouped rule directory.
fn collect_weir_files(dir: &Path, group_root: &Path, weir_root: &Path) -> Vec<StandaloneRule> {
    println!("cargo:rerun-if-changed={}", dir.display());

    let mut entries = fs::read_dir(dir)
        .unwrap()
        .filter_map(Result::ok)
        .collect::<Vec<_>>();

    entries.sort_by_key(|entry| entry.path());

    let mut rules = Vec::new();

    for entry in entries {
        let path = entry.path();
        let file_type = entry.file_type().unwrap();

        if file_type.is_dir() {
            rules.extend(collect_weir_files(&path, group_root, weir_root));
        } else if file_type.is_file() && path.extension().is_some_and(|ext| ext == "weir") {
            println!("cargo:rerun-if-changed={}", path.display());

            rules.push(StandaloneRule {
                name: rule_name_from_relative_path(&path, group_root),
                relative_path: path_as_weir_relative(&path, weir_root),
            });
        }
    }

    rules
}

/// Render a string as an escaped Rust string literal for generated source.
fn rust_string_literal(value: &str) -> String {
    format!("{value:?}")
}

fn write_grouped_weir_boilerplate(weir_rule_dir: &Path, dest: &Path) {
    let mut entries = fs::read_dir(weir_rule_dir)
        .unwrap()
        .filter_map(Result::ok)
        .collect::<Vec<_>>();

    entries.sort_by_key(|entry| entry.path());

    let mut standalone_rules = Vec::new();
    let mut grouped_rules = Vec::new();

    // Watch the root for top-level `.weir` files and group directories.
    println!("cargo:rerun-if-changed={}", weir_rule_dir.display());

    for entry in entries {
        let path = entry.path();
        let file_type = entry.file_type().unwrap();

        if file_type.is_dir() {
            let public_name = entry.file_name().to_string_lossy().to_string();
            let children = collect_weir_files(&path, &path, weir_rule_dir);

            if !children.is_empty() {
                grouped_rules.push(GroupedRule {
                    public_name,
                    children,
                });
            }
        } else if file_type.is_file() && path.extension().is_some_and(|ext| ext == "weir") {
            println!("cargo:rerun-if-changed={}", path.display());

            standalone_rules.push(StandaloneRule {
                name: rule_name_from_path(&path),
                relative_path: path_as_weir_relative(&path, weir_rule_dir),
            });
        }
    }

    let mut code = String::new();

    code.push_str("generate_boilerplate! {\n");
    code.push_str("    standalone: [\n");
    for rule in standalone_rules {
        code.push_str(&format!(
            "        ({}, {}),\n",
            rust_string_literal(&rule.name),
            rust_string_literal(&rule.relative_path)
        ));
    }
    code.push_str("    ],\n");

    code.push_str("    groups: [\n");
    for group in grouped_rules {
        code.push_str(&format!(
            "        ({}, [\n",
            rust_string_literal(&group.public_name)
        ));

        for child in group.children {
            code.push_str(&format!(
                "            ({}, {}),\n",
                rust_string_literal(&child.name),
                rust_string_literal(&child.relative_path)
            ));
        }

        code.push_str("        ]),\n");
    }
    code.push_str("    ],\n");
    code.push_str("}\n");

    fs::write(dest, code).unwrap();
}

fn write_flat_weir_boilerplate(weir_rule_dir: &Path, dest: &Path) {
    let mut files = match fs::read_dir(weir_rule_dir) {
        Ok(entries) => entries
            .filter_map(Result::ok)
            .map(|entry| entry.path())
            .filter(|path| path.is_file() && path.extension().is_some_and(|ext| ext == "weir"))
            .collect::<Vec<_>>(),
        Err(_) => Vec::new(),
    };

    files.sort();

    let mut code = String::new();
    code.push_str("generate_boilerplate!{[");

    for file in files {
        println!("cargo:rerun-if-changed={}", file.display());
        code.push_str(&format!(
            "{},\n",
            file.file_stem().unwrap().to_string_lossy()
        ));
    }

    code.push_str("]}");
    fs::write(dest, code).unwrap();
}

fn main() {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let weir_rule_dir = manifest_dir.join("./src/linting/weir_rules");
    let german_weir_rule_dir = manifest_dir.join("./src/language/german/linting/weir_rules/de");
    let out_dir = Path::new(&env::var("OUT_DIR").unwrap()).to_path_buf();
    let dest = out_dir.join("weir_rules_generated_list.rs");
    let german_dest = out_dir.join("german_weir_rules_generated_list.rs");

    write_grouped_weir_boilerplate(&weir_rule_dir, &dest);
    write_flat_weir_boilerplate(&german_weir_rule_dir, &german_dest);

    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rustc-env=WEIR_RULE_DIR={}", weir_rule_dir.display());
    println!(
        "cargo:rustc-env=GERMAN_WEIR_RULE_DIR={}",
        german_weir_rule_dir.display()
    );
    println!("cargo:rustc-env=WEIR_RULE_LIST={}", dest.display());
    println!(
        "cargo:rustc-env=GERMAN_WEIR_RULE_LIST={}",
        german_dest.display()
    );
}
