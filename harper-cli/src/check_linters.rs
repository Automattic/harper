use std::collections::BTreeSet;
use std::path::Path;

#[derive(Clone, PartialEq)]
struct ModLine {
    snake_norm: Vec<String>,
}

#[derive(Clone, PartialEq)]
struct UseSuperLine {
    snake_norm: Vec<String>,
    pascal_norm: Vec<String>,
}

#[derive(Clone, PartialEq)]
enum RuleKind {
    Expr,
    Struct,
}

#[derive(Clone, PartialEq)]
struct InsertRuleLine {
    kind: RuleKind,
    pascal_norm: Vec<String>,
    default_config: bool,
}

pub fn check_linters(_verbose: bool) -> anyhow::Result<()> {
    let mods = scan_mod_rs()?;
    let (use_supers, insert_rules) = scan_lint_group_rs()?;

    // 1. Check if linter entries in all files are in alphabetical order
    check_ordered(&mods, |m| m.snake_norm.join(" "), "mods");
    check_ordered(&use_supers, |m| m.snake_norm.join(" "), "use supers");
    check_ordered(&insert_rules, |m| m.pascal_norm.join(" "), "insert rules");

    // 2. Check if number of entries matches
    if mods.len() != use_supers.len()
        || mods.len() != insert_rules.len()
        || use_supers.len() != insert_rules.len()
    {
        eprintln!(
            "\n❌ Mismatch in number of items: {} mods vs {} use supers vs {} insert rules",
            mods.len(),
            use_supers.len(),
            insert_rules.len()
        );
    } else {
        println!(
            "\n✅ Number of mods, use supers, and insert rules all match: {}",
            mods.len()
        );
    }

    // 3. List items that are in one but not the other
    let mod_set: BTreeSet<_> = mods.iter().map(|m| m.snake_norm.join("_")).collect();
    let use_super_set: BTreeSet<_> = use_supers
        .iter()
        .map(|us| us.snake_norm.join("_"))
        .collect();
    let insert_rule_set: BTreeSet<_> = insert_rules
        .iter()
        .map(|ir| ir.pascal_norm.join("_").to_lowercase())
        .collect();

    let missing_in_mod_vs_use_super: Vec<_> = mod_set.difference(&use_super_set).collect();
    let extra_in_mod_vs_use_super: Vec<_> = use_super_set.difference(&mod_set).collect();
    let missing_in_mod_vs_insert_rule: Vec<_> = mod_set.difference(&insert_rule_set).collect();
    let extra_in_mod_vs_insert_rule: Vec<_> = insert_rule_set.difference(&mod_set).collect();
    let missing_in_use_super_vs_insert_rule: Vec<_> =
        use_super_set.difference(&insert_rule_set).collect();
    let extra_in_use_super_vs_insert_rule: Vec<_> =
        insert_rule_set.difference(&use_super_set).collect();

    if !missing_in_mod_vs_use_super.is_empty() || !extra_in_mod_vs_use_super.is_empty() {
        eprintln!("\n❌ Mismatch between mods and use supers:");
        if !missing_in_mod_vs_use_super.is_empty() {
            eprintln!("  Missing in mods ({}):", missing_in_mod_vs_use_super.len());
            for item in missing_in_mod_vs_use_super {
                eprintln!("    - {}", item);
            }
        }
        if !extra_in_mod_vs_use_super.is_empty() {
            eprintln!("\n  Extra in mods ({}):", extra_in_mod_vs_use_super.len());
            for item in extra_in_mod_vs_use_super {
                eprintln!("    - {}", item);
            }
        }
    } else {
        println!("\n✅ All mods have corresponding pub uses and vice versa");
    }

    if !missing_in_mod_vs_insert_rule.is_empty() || !extra_in_mod_vs_insert_rule.is_empty() {
        eprintln!("\n❌ Mismatch between mods and insert rules:");
        if !missing_in_mod_vs_insert_rule.is_empty() {
            eprintln!(
                "  Missing in mods ({}):",
                missing_in_mod_vs_insert_rule.len()
            );
            for item in missing_in_mod_vs_insert_rule {
                eprintln!("    - {}", item);
            }
        }
        if !extra_in_mod_vs_insert_rule.is_empty() {
            eprintln!("\n  Extra in mods ({}):", extra_in_mod_vs_insert_rule.len());
            for item in extra_in_mod_vs_insert_rule {
                eprintln!("    - {}", item);
            }
        }
    } else {
        println!("\n✅ All mods have corresponding pub uses and vice versa");
    }

    if !missing_in_use_super_vs_insert_rule.is_empty()
        || !extra_in_use_super_vs_insert_rule.is_empty()
    {
        eprintln!("\n❌ Mismatch between use supers and insert rules:");
        if !missing_in_use_super_vs_insert_rule.is_empty() {
            eprintln!(
                "  Missing in use supers ({}):",
                missing_in_use_super_vs_insert_rule.len()
            );
            for item in missing_in_use_super_vs_insert_rule {
                eprintln!("    - {}", item);
            }
        }
        if !extra_in_use_super_vs_insert_rule.is_empty() {
            eprintln!(
                "\n  Extra in use supers ({}):",
                extra_in_use_super_vs_insert_rule.len()
            );
            for item in extra_in_use_super_vs_insert_rule {
                eprintln!("    - {}", item);
            }
        }
    } else {
        println!("\n✅ All use supers have corresponding insert rules and vice versa");
    }

    // 4. Build a complete set of all linters mentioned
    fn join_parts(parts: &[String]) -> String {
        parts
            .iter()
            .map(|s| s.to_lowercase())
            .collect::<Vec<_>>()
            .join("_")
    }

    // Then collect into a BTreeSet to maintain order and remove duplicates
    let mut all_linters = BTreeSet::new();

    // Add mod linters (snake_case)
    for linter in &mods {
        all_linters.insert(join_parts(&linter.snake_norm));
    }

    // Add use super linters (PascalCase)
    for linter in &use_supers {
        all_linters.insert(join_parts(&linter.pascal_norm));
    }

    // Add insert rule linters (PascalCase)
    for linter in &insert_rules {
        all_linters.insert(join_parts(&linter.pascal_norm));
    }

    // Find the longest linter name length
    let max_len = all_linters.iter().map(String::len).max().unwrap_or(0);

    let linting_dir = Path::new("harper-core/src/linting");

    // Now print with aligned columns
    for linter in &all_linters {
        let in_mod = mods.iter().any(|m| join_parts(&m.snake_norm) == *linter);
        let in_lint_group_use_super = use_supers
            .iter()
            .any(|u| join_parts(&u.pascal_norm) == *linter);
        let rule_info = insert_rules.iter().find_map(|r| {
            if join_parts(&r.pascal_norm) == *linter {
                Some((r.kind.clone(), r.default_config))
            } else {
                None
            }
        });

        let module_path = linting_dir.join(format!("{}.rs", linter));
        let dir_path = linting_dir.join(linter);

        let file_icon = if module_path.exists() {
            "📄" // File exists
        } else if dir_path.exists() {
            "📁" // Directory exists
        } else {
            "  " // Neither exists
        };

        let mod_kind = if let Ok(module) = std::fs::read_to_string(&module_path) {
            let mut kind = "  "; // Default to nothing
            for line in module.lines() {
                if line.contains("impl Linter for") {
                    kind = "🧩"; // Struct linter
                    break;
                } else if line.contains("impl ExprLinter for") {
                    kind = "💬"; // Expression linter
                    break;
                }
            }
            kind
        } else {
            "  " // File doesn't exist or can't be read
        };

        let excl = if let Some((kind, _)) = &rule_info {
            match kind {
                RuleKind::Expr if mod_kind == "🧩" => {
                    "‼️ Rule says Expr but file implements Linter"
                } // Mismatch: Rule says Expr but file implements Linter
                RuleKind::Struct if mod_kind == "💬" => {
                    "‼️ Rule says Struct but file implements ExprLinter"
                } // Mismatch: Rule says Struct but file implements ExprLinter
                _ => "",
            }
        } else {
            ""
        };

        println!(
            "{:<width$} | mod.rs: {} | lint_group.rs: {}{} | file: {}{} {}",
            linter,
            if in_mod { "📦" } else { "  " },
            if in_lint_group_use_super {
                "🦸"
            } else {
                "  "
            },
            if let Some((kind, default)) = rule_info {
                match kind {
                    RuleKind::Expr => "💬",
                    RuleKind::Struct => "🧩",
                }
                .to_string()
                    + if default { "✓" } else { "✗" }
            } else {
                "   ".to_string()
            },
            file_icon,
            mod_kind,
            excl,
            width = max_len
        );
    }

    Ok(())
}

// enum ModRsState {
//     BeforeMods,
//     InMods,
//     BetweenModsAndPubUses,
//     InPubUses,
//     AfterPubUses,
// }

fn scan_mod_rs() -> anyhow::Result<Vec<ModLine>> {
    let mut mods = Vec::new();
    // let mut state = ModRsState::BeforeMods;

    let content = std::fs::read_to_string("harper-core/src/linting/mod.rs")?;

    for line in content.lines() {
        // starts with "mod " / ends with ';' / snake_case only
        if line.starts_with("mod ")
            && let Some(after) = line.strip_prefix("mod ")
            && let Some(linter) = after.strip_suffix(';')
        {
            let linter_norm_split = split_snake_case(linter);
            // eprintln!("🧱 '{linter}' -> {:?}", linter_norm_split);

            mods.push(ModLine {
                snake_norm: linter_norm_split,
            });
        }
    }

    Ok(mods)
}

// enum LintGroupRsState {
//     BeforeUseSupers,
//     InUseSupers,
//     BetweenUseSupersAndRules,
//     InRules,
//     AfterRules,
// }

fn scan_lint_group_rs() -> anyhow::Result<(Vec<UseSuperLine>, Vec<InsertRuleLine>)> {
    let mut use_supers = Vec::new();
    let mut insert_rules = Vec::new();
    // let mut state = LintGroupRsState::BeforeUseSupers;

    let content = std::fs::read_to_string("harper-core/src/linting/lint_group.rs")?;

    for line in content.lines() {
        // starts with "use super::" / ends with ';' / snake_case `::` PascalCase
        if line.starts_with("use super::")
            && let Some(after) = line.strip_prefix("use super::")
            && let Some(snake_pascal) = after.strip_suffix(';')
            && let Some((snake, pascal)) = snake_pascal.split_once("::")
        {
            let (snake_norm, pascal_norm) = (split_snake_case(snake), split_pascal_case(pascal));

            if snake_norm.len() != pascal_norm.len()
                || !snake_norm
                    .iter()
                    .zip(&pascal_norm)
                    .all(|(s, p)| s.eq_ignore_ascii_case(p))
            {
                eprintln!(
                    "🦸 '{snake}' / '{pascal}' -> {:?} / {:?}",
                    snake_norm, pascal_norm
                );
            } else {
                use_supers.push(UseSuperLine {
                    snake_norm,
                    pascal_norm,
                });
            }
        }

        if line.starts_with("        insert_")
            && let Some(after) = line.strip_prefix("        insert_")
            && let Some(stuff) = after.strip_suffix(");")
            && let Some((kind, rule_default_config)) = stuff.split_once("_rule!(")
            && let Some((rule, default_config)) = rule_default_config.split_once(", ")
        {
            if (kind == "expr" || kind == "struct")
                && (default_config == "true" || default_config == "false")
            {
                let pascal_norm = split_pascal_case(rule);
                insert_rules.push(InsertRuleLine {
                    kind: if kind == "expr" {
                        RuleKind::Expr
                    } else {
                        RuleKind::Struct
                    },
                    pascal_norm,
                    default_config: default_config == "true",
                });
            } else {
                eprintln!("❌ insert rule: '{kind}' // '{rule}' // '{default_config}'");
            }
        }
    }

    Ok((use_supers, insert_rules))
}

fn split_snake_case(s: &str) -> Vec<String> {
    s.split('_').map(String::from).collect()
}

fn split_pascal_case(s: &str) -> Vec<String> {
    let mut result = Vec::new();
    let mut current = String::new();

    for c in s.chars() {
        if c.is_uppercase() && !current.is_empty() {
            result.push(current);
            current = String::new();
        }
        current.push(c);
    }

    if !current.is_empty() {
        result.push(current);
    }

    result
}

/// Checks if a sequence is in order according to the given key function.
fn check_ordered<T, F, K>(items: &[T], key_fn: F, item_name: &str)
where
    F: Fn(&T) -> K,
    K: std::cmp::PartialOrd + std::fmt::Display,
{
    let mut total_errors = 0;

    for (i, window) in items.windows(2).enumerate() {
        let a = key_fn(&window[0]);
        let b = key_fn(&window[1]);

        if b < a {
            total_errors += 1;
            if total_errors <= 3 {
                eprintln!(
                    "❌ {} are out of order at {}/{} where '{}' should come after '{}'",
                    item_name,
                    i,
                    i + 1,
                    a,
                    b
                );
            }
        }
    }

    if total_errors > 3 {
        eprintln!(
            "   and {} more {} out of order entries (total: {})",
            total_errors - 3,
            item_name.to_lowercase(),
            total_errors
        );
    }
}
