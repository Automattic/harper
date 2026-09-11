use crate::{
    CharStringExt, Lint, Token,
    expr::{Expr, FirstMatchOf, SequenceExpr},
    linting::{
        ExprLinter, LintKind, Suggestion,
        expr_linter::{Chunk, find_the_only_token_matching},
    },
};

const RARE_BEFORE_DAIRY: &[&str] = &[
    "books",
    "dear",
    "desk",
    "economic",
    "electronic",
    "her",
    "his",
    "my",
    "owl",
    "personal",
    "princess",
    "red",
    "secret",
    "this",
    "travel",
    "turner",
    "vampire",
    "war",
    "women's",
    "writer's",
    "your",
];
const RARE_BEFORE_DIARY: &[&str] = &["non"];
const RARE_AFTER_DAIRY: &[&str] = &["entries", "entry", "notes", "page", "pages"];
const RARE_AFTER_DIARY: &[&str] = &[
    "allergies",
    "allergy",
    "beverage",
    "beverages",
    "cattle",
    "cow",
    "cows",
    "creamer",
    "creamers",
    "derivative",
    "derivatives",
    "drink",
    "farm",
    "farmer",
    "farmers",
    "farming",
    "farms",
    "foods",
    "formula",
    "formulas",
    "herd",
    "herds",
    "industry",
    "intolerance",
    "intolerances",
    "intolerant",
    "milk",
    "milks",
    "product",
    "products",
    "queen",
    "queens",
    "residue",
    "sensitive",
    "sensitivity",
    "sensitivities",
    "shake",
    "shakes",
    "solids",
    "substance",
];
pub struct DairyDiary {
    expr: FirstMatchOf,
}

impl Default for DairyDiary {
    fn default() -> Self {
        Self {
            expr: FirstMatchOf::new([
                Box::new(
                    SequenceExpr::word_set(RARE_BEFORE_DAIRY)
                        .t_ws()
                        .t_set(&["dairy", "dairies"]),
                ) as Box<dyn Expr>,
                Box::new(
                    SequenceExpr::word_set(RARE_BEFORE_DAIRY)
                        .t_ws()
                        .t_set(&["diary", "diaries"]),
                ) as Box<dyn Expr>,
                Box::new(SequenceExpr::aco("non").then_hyphen().t_aco("diary")),
                Box::new(
                    SequenceExpr::word_set(&["dairy", "dairies"])
                        .t_ws()
                        .t_set(RARE_AFTER_DAIRY),
                ),
                Box::new(
                    SequenceExpr::word_set(&["diary", "diaries"])
                        .t_ws()
                        .t_set(RARE_AFTER_DIARY),
                ),
            ]),
        }
    }
}

impl ExprLinter for DairyDiary {
    type Unit = Chunk;

    fn match_to_lint(&self, matched_tokens: &[Token], source: &[char]) -> Option<Lint> {
        let span = find_the_only_token_matching(matched_tokens, source, |t, c| {
            t.get_ch(c)
                .eq_any_ignore_ascii_case_str(&["dairy", "dairies", "diary", "diaries"])
        })?
        .span;

        let mut the_word = span.get_content(source).to_vec();

        the_word.swap(1, 2);

        Some(Lint {
            span,
            lint_kind: LintKind::WordChoice,
            suggestions: vec![Suggestion::ReplaceWith(the_word)],
            message: "Are you confusing `dairy` (milk products) with `diary` (journal)?".to_owned(),
            ..Default::default()
        })
    }

    fn expr(&self) -> &dyn Expr {
        &self.expr
    }

    fn description(&self) -> &str {
        "Detects mixing up `dairy` and `diary`."
    }
}

#[cfg(test)]
mod tests {
    use crate::linting::tests::{assert_no_lints, assert_suggestion_result};

    use super::DairyDiary;

    #[test]
    fn fix_dairy_entry() {
        assert_suggestion_result(
            "Dairy Entry of an Immigrant (Part 1) · IKEA.",
            DairyDiary::default(),
            "Diary Entry of an Immigrant (Part 1) · IKEA.",
        );
    }

    #[test]
    fn fix_dairy_entries() {
        assert_suggestion_result(
            "This is told in the form of dairy entries by the captain of the group.",
            DairyDiary::default(),
            "This is told in the form of diary entries by the captain of the group.",
        );
    }

    #[test]
    fn fix_dairy_notes() {
        assert_suggestion_result(
            "AI App to take dairy notes and have connection with a professional if you want.",
            DairyDiary::default(),
            "AI App to take diary notes and have connection with a professional if you want.",
        );
    }

    #[test]
    fn fix_dairy_page() {
        assert_suggestion_result(
            "it reads more like a dairy page of someone who should go to therapy rather than an entry of a competition",
            DairyDiary::default(),
            "it reads more like a diary page of someone who should go to therapy rather than an entry of a competition",
        );
    }

    #[test]
    fn fix_dairy_pages() {
        assert_suggestion_result(
            "You're really good, I hope to see more dairy pages from you.",
            DairyDiary::default(),
            "You're really good, I hope to see more diary pages from you.",
        );
    }

    #[test]
    fn fix_dear_dairy() {
        assert_suggestion_result(
            "Albert Flowerpot's diary entry 2. Dear dairy",
            DairyDiary::default(),
            "Albert Flowerpot's diary entry 2. Dear diary",
        );
    }

    #[test]
    fn fix_diary_allergies() {
        assert_suggestion_result(
            "Food for my baby with diary allergies & a mild egg allergy.",
            DairyDiary::default(),
            "Food for my baby with dairy allergies & a mild egg allergy.",
        );
    }

    #[test]
    fn fix_diary_allergy() {
        assert_suggestion_result(
            "I don't have to interpret \"denatured whey protein\" as pertinent to my diary allergy",
            DairyDiary::default(),
            "I don't have to interpret \"denatured whey protein\" as pertinent to my dairy allergy",
        );
    }

    #[test]
    fn fix_diary_beverage() {
        assert_suggestion_result(
            "find out what's so wonderful about having an entire day's worth of calories in a one diary beverage, but have no idea where to actually get one",
            DairyDiary::default(),
            "find out what's so wonderful about having an entire day's worth of calories in a one dairy beverage, but have no idea where to actually get one",
        );
    }

    #[test]
    fn fix_diary_beverages() {
        assert_suggestion_result(
            "Non-diary beverages including soy, rice and oat beverages",
            DairyDiary::default(),
            "Non-dairy beverages including soy, rice and oat beverages",
        );
    }

    #[test]
    fn fix_diary_cattle() {
        assert_suggestion_result(
            "This has been used for many kinds of animals - such as cats, dogs, diary cattle.",
            DairyDiary::default(),
            "This has been used for many kinds of animals - such as cats, dogs, dairy cattle.",
        );
    }

    #[test]
    fn fix_diary_cow() {
        assert_suggestion_result(
            "Diary cow cohabitation. Is it ideal to cohab diary cows?",
            DairyDiary::default(),
            "Dairy cow cohabitation. Is it ideal to cohab dairy cows?",
        );
    }

    #[test]
    fn fix_diary_cows() {
        assert_suggestion_result(
            "yes technically there will be 0 harm possible because we will have to kill all the diary cows",
            DairyDiary::default(),
            "yes technically there will be 0 harm possible because we will have to kill all the dairy cows",
        );
    }

    #[test]
    fn fix_diary_creamers() {
        assert_suggestion_result(
            "rather than the pre-mix matcha powders with Non-diary creamers used by boba cafes",
            DairyDiary::default(),
            "rather than the pre-mix matcha powders with Non-dairy creamers used by boba cafes",
        );
    }

    #[test]
    fn fix_diary_derivatives() {
        assert_suggestion_result(
            "There are many foods that are either dairy, diary derivatives, or made with dairy.",
            DairyDiary::default(),
            "There are many foods that are either dairy, diary derivatives, or made with dairy.",
        );
    }

    #[test]
    fn fix_diary_drink() {
        assert_suggestion_result(
            "it is a strange sparkling diary drink, made usually from horse milk",
            DairyDiary::default(),
            "it is a strange sparkling dairy drink, made usually from horse milk",
        );
    }

    #[test]
    fn fix_diary_farm() {
        assert_suggestion_result(
            "This is a Diary Farm management System that manages staff,registers sellers and prints receipts and reports.",
            DairyDiary::default(),
            "This is a Dairy Farm management System that manages staff,registers sellers and prints receipts and reports.",
        );
    }

    #[test]
    fn fix_diary_farmers() {
        assert_suggestion_result(
            "Here in Canada, diary farmers need to be licensed to sell dairy products, as do poultry producers",
            DairyDiary::default(),
            "Here in Canada, dairy farmers need to be licensed to sell dairy products, as do poultry producers",
        );
    }

    #[test]
    fn fix_diary_farming() {
        assert_suggestion_result(
            "Actually the subsidies mostly go to diary farming.",
            DairyDiary::default(),
            "Actually the subsidies mostly go to dairy farming.",
        );
    }

    #[test]
    fn fix_diary_farms() {
        assert_suggestion_result(
            "I live in a place with quite a lot of diary farms around and its pretty normal to drink milk at meals.",
            DairyDiary::default(),
            "I live in a place with quite a lot of dairy farms around and its pretty normal to drink milk at meals.",
        );
    }

    #[test]
    fn fix_diary_foods() {
        assert_suggestion_result(
            "Avoid diary foods! Milk is unhealthy for humans.",
            DairyDiary::default(),
            "Avoid dairy foods! Milk is unhealthy for humans.",
        );
    }

    #[test]
    fn fix_diary_formula() {
        assert_suggestion_result(
            "We used toddler nom diary formula and pea protein milk.",
            DairyDiary::default(),
            "We used toddler nom dairy formula and pea protein milk.",
        );
    }

    #[test]
    fn fix_diary_formulas() {
        assert_suggestion_result(
            "You have to check the ingredients. Diary formulas have natural sweetness.",
            DairyDiary::default(),
            "You have to check the ingredients. Dairy formulas have natural sweetness.",
        );
    }

    #[test]
    fn fix_diary_herd() {
        assert_suggestion_result(
            "the management of foot and mouth disease lesions in a diary herd",
            DairyDiary::default(),
            "the management of foot and mouth disease lesions in a dairy herd",
        );
    }

    #[test]
    fn fix_diary_herds() {
        assert_suggestion_result(
            "Detection of brucellosis in diary herds after an outbreak of the disease using a delayed-type hypersensitivity test",
            DairyDiary::default(),
            "Detection of brucellosis in dairy herds after an outbreak of the disease using a delayed-type hypersensitivity test",
        );
    }

    #[test]
    fn fix_diary_industry() {
        assert_suggestion_result(
            "However, having spent much of my existence around the Canadian diary industry it is clear that it has always been the biggest sticking point in US and Canada relations.",
            DairyDiary::default(),
            "However, having spent much of my existence around the Canadian dairy industry it is clear that it has always been the biggest sticking point in US and Canada relations.",
        );
    }

    #[test]
    fn fix_diary_intolerance() {
        assert_suggestion_result(
            "If your baby had a diary intolerance when did you try incorporating it again?",
            DairyDiary::default(),
            "If your baby had a dairy intolerance when did you try incorporating it again?",
        );
    }

    #[test]
    fn fix_diary_intolerances() {
        assert_suggestion_result(
            "I'v got diary intolerances and I'm staring at that blue croissant dough",
            DairyDiary::default(),
            "I'v got dairy intolerances and I'm staring at that blue croissant dough",
        );
    }

    #[test]
    fn fix_diary_intolerant() {
        assert_suggestion_result(
            "So I'm traveling with my sister who is gluten and diary intolerant.",
            DairyDiary::default(),
            "So I'm traveling with my sister who is gluten and dairy intolerant.",
        );
    }

    #[test]
    fn fix_diary_milk() {
        assert_suggestion_result(
            "Is the term \"milk\" synonymous with diary milk?",
            DairyDiary::default(),
            "Is the term \"milk\" synonymous with dairy milk?",
        );
    }

    #[test]
    fn fix_diary_milks() {
        assert_suggestion_result(
            "the additional charge for non diary milks",
            DairyDiary::default(),
            "the additional charge for non dairy milks",
        );
    }

    #[test]
    fn fix_diary_product() {
        assert_suggestion_result(
            "Milk and Diary Product Technology - BWD 40603.",
            DairyDiary::default(),
            "Milk and Dairy Product Technology - BWD 40603.",
        );
    }

    #[test]
    fn fix_diary_products() {
        assert_suggestion_result(
            "Adhesion of fermented diary products to packaging materials.",
            DairyDiary::default(),
            "Adhesion of fermented dairy products to packaging materials.",
        );
    }

    #[test]
    fn fix_diary_queen() {
        assert_suggestion_result(
            "Sort of like how Diary Queen aren't allowed to call their desserts “Ice Cream” because there isn't enough dairy",
            DairyDiary::default(),
            "Sort of like how Dairy Queen aren't allowed to call their desserts “Ice Cream” because there isn't enough dairy",
        );
    }

    #[test]
    fn fix_diary_queens() {
        assert_suggestion_result(
            "Actually this is Diary Queens prices not DoorDash.",
            DairyDiary::default(),
            "Actually this is Dairy Queens prices not DoorDash.",
        );
    }

    #[test]
    fn fix_diary_residue() {
        assert_suggestion_result(
            "What I've heard is it takes 2-3 weeks of time for diary residue to leave your system",
            DairyDiary::default(),
            "What I've heard is it takes 2-3 weeks of time for dairy residue to leave your system",
        )
    }

    #[test]
    fn fix_diary_sensitive() {
        assert_suggestion_result(
            "The formula was for diary sensitive and i'm also lactose intolerant.",
            DairyDiary::default(),
            "The formula was for dairy sensitive and i'm also lactose intolerant.",
        );
    }

    #[test]
    fn fix_diary_sensitivity() {
        assert_suggestion_result(
            "Formula suggestions for diary sensitivity?",
            DairyDiary::default(),
            "Formula suggestions for dairy sensitivity?",
        );
    }

    #[test]
    fn fix_diary_sensitivities() {
        assert_suggestion_result(
            "I was into my early 20s that I discovered I had gluten and diary sensitivities",
            DairyDiary::default(),
            "I was into my early 20s that I discovered I had gluten and dairy sensitivities",
        );
    }

    #[test]
    fn fix_diary_shake() {
        assert_suggestion_result(
            "Cheese spreads, diary shake powders, those kinds of things will usually go off first.",
            DairyDiary::default(),
            "Cheese spreads, dairy shake powders, those kinds of things will usually go off first.",
        );
    }

    #[test]
    #[ignore = "rare false positive?"]
    fn dont_flag_diary_shakes() {
        assert_no_lints(
            "the newly found pages of Edwina's diary shake in my hands with each pulse",
            DairyDiary::default(),
        );
    }

    #[test]
    fn fix_diary_shakes() {
        assert_suggestion_result(
            "next to the list of diary shakes, there is a little icon that says, \"Make Non-Dairy for $2\"",
            DairyDiary::default(),
            "next to the list of dairy shakes, there is a little icon that says, \"Make Non-Dairy for $2\"",
        );
    }

    #[test]
    fn fix_diary_solids() {
        assert_suggestion_result(
            "I know usually it's coconut milk but some coconut milk has diary solids.",
            DairyDiary::default(),
            "I know usually it's coconut milk but some coconut milk has dairy solids.",
        );
    }

    #[test]
    fn fix_diary_substance() {
        assert_suggestion_result(
            "topped with melted diary substance and spicy bean mix",
            DairyDiary::default(),
            "topped with melted dairy substance and spicy bean mix",
        );
    }

    #[test]
    fn fix_non_diary() {
        assert_suggestion_result(
            "Why is non-diary coffee creamer so prevalent when only a small portion of the population is lactose intolerant.",
            DairyDiary::default(),
            "Why is non-dairy coffee creamer so prevalent when only a small portion of the population is lactose intolerant.",
        );
    }
}
