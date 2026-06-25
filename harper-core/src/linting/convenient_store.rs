use crate::{
    CharStringExt, Lint, Token,
    expr::{Expr, SequenceExpr},
    linting::{ExprLinter, LintKind, Suggestion, expr_linter::Sentence},
};

pub struct ConvenientStore {
    expr: SequenceExpr,
}

// If any of these are in the 'words' around the match, it looks more like a mistake for "convenience store".
const NOUN_CLUES: &[&str] = &[
    "11",
    "24",
    "7",
    "99", // 99 speedmart, convenience store chain in malaysia
    "alcohol",
    "around",
    "beer",
    "bhd", // sdn bhd is Malay for pty ltd and common even in English
    "brands",
    "business",
    "buy",
    "cafes",
    "chain",
    "clerk",
    "coffee",
    "coupon",
    "coupons",
    "deli",
    "eleven",
    "family",
    "familymart",
    "groceries",
    "grocery",
    "kk", // major convenience store chain in malaysia
    "liquor",
    "located",
    "location",
    "locations",
    "lotus",    // tesco lotus, convenience store chain in southeast asia
    "malaysia", // this mistake is very common in Malaysian English
    "mall",
    "market",
    "mart",
    "marts",
    "minimart",
    "ministop", // convenience store chain
    "mrt",      // mass rapid transit in malaysia and singapore
    "mynews",
    "near",
    "nearby",
    "neighborhood",
    "neighbourhood",
    "products",
    "purchase",
    "receipt",
    "restaurant",
    "restaurants",
    "retail",
    "retailers",
    "sdn", // sdn bhd is Malay for pty ltd and common even in English
    "seven",
    "shop",
    "shopping",
    "shops",
    "singapore", // this mistake is very common in Singapore English
    "snack",
    "snacks",
    "sold",
    "speedmart", // 99 speedmart, convenience store chain in malaysia
    "stalls",
    "station",
    "supermarket",
    "supermarkets",
    "tesco", // tesco lotus, convenience store in southeast asia
    "traded",
];

// If any of these are in the 'words' around the match, it looks more like a legitimate use of "convenient store".
const ADJ_CLUES: &[&str] = &[
    "api",
    "cache",
    "client",
    "database",
    "db",
    "dbms",
    "docs",
    "extension",
    "gui",
    "methods",
    "redis",
    "server",
    "stored",
    "type",
    "value",
    "yaml",
];

impl Default for ConvenientStore {
    fn default() -> Self {
        Self {
            expr: SequenceExpr::aco("convenient")
                .t_ws()
                .t_set(&["store", "stores"]),
        }
    }
}

impl ExprLinter for ConvenientStore {
    type Unit = Sentence;

    fn match_to_lint_with_context(
        &self,
        toks: &[Token],
        src: &[char],
        ctx: Option<(&[Token], &[Token])>,
    ) -> Option<Lint> {
        let (noun_clues, adj_clues) = ctx
            .map(|(before, after)| {
                before
                    .iter()
                    .chain(after.iter())
                    .fold((0, 0), |(noun, adj), token| {
                        if token.kind.is_word_like() {
                            let ch = token.get_ch(src);
                            (
                                noun + ch.eq_any_ignore_ascii_case_str(NOUN_CLUES) as usize,
                                adj + ch.eq_any_ignore_ascii_case_str(ADJ_CLUES) as usize,
                            )
                        } else {
                            (noun, adj)
                        }
                    })
            })
            .unwrap_or((0, 0));

        if adj_clues >= noun_clues {
            return None;
        }

        let span = toks[0].span;
        let lint_kind = LintKind::Eggcorn;
        let suggestions = vec![Suggestion::replace_with_match_case_str(
            "convenience",
            span.get_content(src),
        )];
        let message = "Did you mean `convenience store`?".to_string();
        Some(Lint {
            span,
            lint_kind,
            suggestions,
            message,
            ..Default::default()
        })
    }

    fn expr(&self) -> &dyn Expr {
        &self.expr
    }

    fn description(&self) -> &str {
        "Attempts to detect when `convenient store` is mistake for `convenience store`."
    }
}

#[cfg(test)]
mod tests {
    use crate::linting::tests::{assert_no_lints, assert_suggestion_result};

    use super::ConvenientStore;

    // Real mistakes, mainly harvested from outside GitHub

    #[test]
    fn fix_7_11() {
        assert_suggestion_result(
            "7-11 convenient stores is 20 steps away!",
            ConvenientStore::default(),
            "7-11 convenience stores is 20 steps away!",
        );
    }

    #[test]
    fn fix_nearby() {
        assert_suggestion_result(
            "hi, is there any convenient stores nearby?",
            ConvenientStore::default(),
            "hi, is there any convenience stores nearby?",
        );
    }

    #[test]
    fn fix_around() {
        assert_suggestion_result(
            "Yes they do have convenient stores around.",
            ConvenientStore::default(),
            "Yes they do have convenience stores around.",
        );
    }

    #[test]
    fn fix_24_7() {
        assert_suggestion_result(
            "There is also a convenient store just downstairs which is 24/7.",
            ConvenientStore::default(),
            "There is also a convenience store just downstairs which is 24/7.",
        );
    }

    #[test]
    fn fix_coupons() {
        assert_suggestion_result(
            "Mobile Application for small Convenient stores to keep track of the coupons by scanning the coupon barcode.",
            ConvenientStore::default(),
            "Mobile Application for small Convenience stores to keep track of the coupons by scanning the coupon barcode.",
        );
    }

    #[test]
    fn fix_malaysia() {
        assert_suggestion_result(
            "Find deals on Kanak Kanak Convenient Store products online with Lazada Malaysia",
            ConvenientStore::default(),
            "Find deals on Kanak Kanak Convenience Store products online with Lazada Malaysia",
        );
    }

    #[test]
    fn fix_singapore() {
        assert_suggestion_result(
            "Buy Sweet Bean Convenient Store in Singapore,Singapore.",
            ConvenientStore::default(),
            "Buy Sweet Bean Convenience Store in Singapore,Singapore.",
        );
    }

    #[test]
    fn fix_shopping_snacks() {
        assert_suggestion_result(
            "Ok, so I was at a convenient store and I noticed this young woman reading through magazines while I was shopping for some snacks.",
            ConvenientStore::default(),
            "Ok, so I was at a convenience store and I noticed this young woman reading through magazines while I was shopping for some snacks.",
        );
    }

    #[test]
    fn fix_shopping_mall_restaurants() {
        assert_suggestion_result(
            "Connected to Mercury Ville shopping mall comprising of restaurants, convenient stores, clinics and banks",
            ConvenientStore::default(),
            "Connected to Mercury Ville shopping mall comprising of restaurants, convenience stores, clinics and banks",
        );
    }

    #[test]
    fn fix_minimart() {
        assert_suggestion_result(
            "There are many convenient store, 7-11 minimart,Tesco Lotus supermarket,Century Mall, Big C Supermarket,Fresh market . ",
            ConvenientStore::default(),
            "There are many convenience store, 7-11 minimart,Tesco Lotus supermarket,Century Mall, Big C Supermarket,Fresh market . ",
        );
    }

    // Potential false positives, mainly harvested from GitHub

    #[test]
    fn dont_flag_api() {
        assert_no_lints(
            "Convenient \"Store\" API (deprecates ObjectBox and Builder API)",
            ConvenientStore::default(),
        );
    }

    #[test]
    fn dont_flag_gui() {
        assert_no_lints(
            "It can be used for a small convenient store after certain modifications to the GUI.",
            ConvenientStore::default(),
        );
    }

    #[test]
    fn dont_flag_methods_docs() {
        assert_no_lints(
            "Add convenient store methods; Improve the docs; Improve some internals.",
            ConvenientStore::default(),
        );
    }

    #[test]
    fn dont_flag_stored() {
        assert_no_lints(
            "So, managing and transferring states becomes easier as all the states are stored in the same convenient store.",
            ConvenientStore::default(),
        );
    }

    #[test]
    fn dont_flag_value_type() {
        assert_no_lints(
            "A safe and convenient store for one value of each type.",
            ConvenientStore::default(),
        );
    }

    #[test]
    fn dont_flag_redis() {
        assert_no_lints(
            "A Redis server, running on the local client device can serve as a convenient store for messages.",
            ConvenientStore::default(),
        );
    }

    #[test]
    fn dong_flag_yaml() {
        assert_no_lints(
            "Two reasons: it's a more convenient store than yaml, especially for binary data, and it's easier to re-encrypt when gpg-keys change.",
            ConvenientStore::default(),
        );
    }

    #[test]
    fn dont_flag_mini() {
        assert_no_lints(
            "Mini glass bottle collection of convenient store.",
            ConvenientStore::default(),
        );
    }

    // Real sentences for which it's hard to determine if it's a true or false positive.

    #[test]
    fn dont_flag_frame_dump() {
        assert_no_lints(
            "GE frame dump at convenient store.",
            ConvenientStore::default(),
        );
    }

    #[test]
    fn dont_flag_convenient_store_means_handy() {
        assert_no_lints(
            "It is a great feature as customers can choose a convenient store for their order delivery and they know when it can be picked up.",
            ConvenientStore::default(),
        );
    }

    #[test]
    fn dont_flag() {
        assert_no_lints(
            "It also allows you to create a convenient store that includes any additional structure and the actions and selectors to be used to manage the entities.",
            ConvenientStore::default(),
        );
    }
}
