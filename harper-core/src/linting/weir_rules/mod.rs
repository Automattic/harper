use super::LintGroup;
use crate::weir::WeirLinter;

macro_rules! generate_boilerplate {
    (
        standalone: [$(($standalone_name:literal, $standalone_path:literal)),* $(,)?],
        groups: [$(($group_name:literal, [$(($child_name:literal, $child_path:literal)),* $(,)?])),* $(,)?],
    ) => {
        pub fn lint_group() -> LintGroup {
            let mut group = LintGroup::default();

            // Standalone `.weir` files remain one public Harper rule each.
            $(
                add_weir_linter(
                    &mut group,
                    $standalone_name,
                    include_str!(concat!(env!("WEIR_RULE_DIR"), "/", $standalone_path)),
                );
            )*

            $(
                {
                    // A directory under `weir_rules` is exposed as a single public rule.
                    // Its children are regular Weir rules inside an inner LintGroup, so they
                    // run independently while the outer configuration only sees `$group_name`.
                    let mut grouped_rule = LintGroup::default();

                    $(
                        add_weir_linter(
                            &mut grouped_rule,
                            $child_name,
                            include_str!(concat!(env!("WEIR_RULE_DIR"), "/", $child_path)),
                        );
                    )*

                    grouped_rule.set_all_rules_to(Some(true));
                    group.add($group_name, grouped_rule);
                }
            )*

            group.set_all_rules_to(Some(true));

            group
        }

        /// Add a Weir rule using its declared chunk/sentence scope.
        fn add_weir_linter(group: &mut LintGroup, name: &str, weir_code: &str) {
            let linter = WeirLinter::new(weir_code).unwrap();
            match linter.into_sentence_linter() {
                Ok(linter) => group.add_sentence_expr_linter(name, linter),
                Err(linter) => group.add_chunk_expr_linter(
                    name,
                    linter.into_chunk_linter().unwrap_or_else(|_| unreachable!()),
                ),
            };
        }

        #[cfg(test)]
        mod tests {
            use crate::weir::{TestResult, WeirLinter};

            // Run EVERY rule and aggregate the failures, rather than asserting
            // per-rule (which panics on the first failing rule and masks the
            // rest). One broken rule used to hide every later one in iteration
            // order — see the joint-tagger retrain, where a single weir failure
            // concealed three more.
            #[test]
            fn run_tests_for_weir_rules() {
                let mut failures: Vec<(&str, Vec<TestResult>)> = Vec::new();

                $(
                    let mut linter = WeirLinter::new(
                        include_str!(concat!(env!("WEIR_RULE_DIR"), "/", $standalone_path)),
                    ).unwrap();
                    let res = linter.run_tests();
                    if !res.is_empty() {
                        failures.push(($standalone_name, res));
                    }
                )*

                $($(
                    let mut linter = WeirLinter::new(
                        include_str!(concat!(env!("WEIR_RULE_DIR"), "/", $child_path)),
                    ).unwrap();
                    let res = linter.run_tests();
                    if !res.is_empty() {
                        failures.push(($child_name, res));
                    }
                )*)*

                assert!(
                    failures.is_empty(),
                    "{} weir rule(s) have failing test cases:\n{}",
                    failures.len(),
                    failures
                        .iter()
                        .map(|(name, res)| format!(
                            "  {name}: {}",
                            res.iter()
                                .map(|r| format!("[expected {:?}, got {:?}]", r.expected, r.got))
                                .collect::<Vec<_>>()
                                .join(", ")
                        ))
                        .collect::<Vec<_>>()
                        .join("\n")
                );
            }
        }
    };
}

include!(env!("WEIR_RULE_LIST"));
