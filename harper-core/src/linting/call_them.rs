use std::{ops::Range, sync::Arc};

use crate::expr::{Expr, ExprMap, SequenceExpr};
use crate::patterns::DerivedFrom;
use crate::{Token, TokenStringExt};

use super::{ExprLinter, Lint, LintKind, Suggestion};
use crate::linting::expr_linter::Chunk;

pub struct CallThem {
    expr: Box<dyn Expr>,
    map: Arc<ExprMap<Range<usize>>>,
}

impl Default for CallThem {
    fn default() -> Self {
        let mut map = ExprMap::default();

        let post_exception = Arc::new(SequenceExpr::default().t_ws().then_word_set(&["if", "it"]));

        map.insert(
            SequenceExpr::with(DerivedFrom::new_from_str("call"))
                .t_ws()
                .then_pronoun()
                .t_ws()
                .t_aco("as")
                .then_unless(post_exception.clone()),
            3..5,
        );

        map.insert(
            SequenceExpr::with(DerivedFrom::new_from_str("call"))
                .t_ws()
                .t_aco("as")
                .t_ws()
                .then_pronoun()
                .then_unless(post_exception.clone()),
            1..3,
        );

        let map = Arc::new(map);

        Self {
            expr: Box::new(map.clone()),
            map,
        }
    }
}

impl ExprLinter for CallThem {
    type Unit = Chunk;

    fn expr(&self) -> &dyn Expr {
        self.expr.as_ref()
    }

    fn match_to_lint(&self, matched_tokens: &[Token], source: &[char]) -> Option<Lint> {
        let removal_range = self.map.lookup(0, matched_tokens, source)?.clone();
        let offending_tokens = matched_tokens.get(removal_range)?;

        Some(Lint {
            span: offending_tokens.span()?,
            lint_kind: LintKind::Redundancy,
            suggestions: vec![Suggestion::Remove],
            message: "`as` is redundant in this context.".to_owned(),
            ..Default::default()
        })
    }

    fn description(&self) -> &'static str {
        "Addresses the non-idiomatic phrases `call them as`."
    }
}

#[cfg(test)]
mod tests {
    #[allow(unused_imports)]
    use crate::Document;
    use crate::linting::tests::{assert_no_lints, assert_suggestion_result};

    use super::CallThem;

    #[test]
    fn prefer_plug_and_receptacle() {
        assert_suggestion_result(
            r#"I prefer to call them as Plug (male) and Receptacle (female). Receptacles are seen in laptops, mobile phones etc.."#,
            CallThem::default(),
            r#"I prefer to call them Plug (male) and Receptacle (female). Receptacles are seen in laptops, mobile phones etc.."#,
        );
    }

    #[test]
    fn builtins_id() {
        assert_suggestion_result(
            r#"I’d categorically ignore *id* as a builtin, and when you do need it in a module, make it super explicit and `import builtins` and call it as `builtins.id`."#,
            CallThem::default(),
            r#"I’d categorically ignore *id* as a builtin, and when you do need it in a module, make it super explicit and `import builtins` and call it `builtins.id`."#,
        );
    }

    #[test]
    fn non_modal_dialogue() {
        assert_suggestion_result(
            r#"We usually call it as non-modal dialogue e.g. when hit Gmail compose button, a nonmodal dialogue opens."#,
            CallThem::default(),
            r#"We usually call it non-modal dialogue e.g. when hit Gmail compose button, a nonmodal dialogue opens."#,
        );
    }

    #[test]
    fn prefer_to_call_them() {
        assert_suggestion_result(
            r#"So, how do you typically prefer to call them as?"#,
            CallThem::default(),
            r#"So, how do you typically prefer to call them?"#,
        );
    }

    #[test]
    fn called_them_allies() {
        assert_suggestion_result(
            r#"Yes as tribes or nomads you called them as allies but you didn’t get their levies as your own."#,
            CallThem::default(),
            r#"Yes as tribes or nomads you called them allies but you didn’t get their levies as your own."#,
        );
    }

    #[test]
    fn character_development() {
        assert_suggestion_result(
            r#"I call this as character development."#,
            CallThem::default(),
            r#"I call this character development."#,
        );
    }

    #[test]
    fn fate_or_time() {
        assert_suggestion_result(
            r#"Should I Call It As Fate Or Time"#,
            CallThem::default(),
            r#"Should I Call It Fate Or Time"#,
        );
    }

    #[test]
    fn abstract_latte_art() {
        assert_suggestion_result(
            r#"Can we just call it as abstract latte art."#,
            CallThem::default(),
            r#"Can we just call it abstract latte art."#,
        );
    }

    #[test]
    fn sounding_boards() {
        assert_suggestion_result(
            r#"I call them as my ‘sounding boards’"#,
            CallThem::default(),
            r#"I call them my ‘sounding boards’"#,
        );
    }

    #[test]
    fn calling_them_disaster() {
        assert_suggestion_result(
            r#"I totally disagree with your point listed and calling them as disaster."#,
            CallThem::default(),
            r#"I totally disagree with your point listed and calling them disaster."#,
        );
    }

    #[test]
    fn battle_of_boxes() {
        assert_suggestion_result(
            r#"Windows Sandbox and VirtualBox or I would like to call this as “Battle of Boxes.”"#,
            CallThem::default(),
            r#"Windows Sandbox and VirtualBox or I would like to call this “Battle of Boxes.”"#,
        );
    }

    #[test]
    fn called_her_shinnasan() {
        assert_suggestion_result(
            r#"Nice meeting a follower from reddit I called her as Shinna-san, welcome again to Toram!!"#,
            CallThem::default(),
            r#"Nice meeting a follower from reddit I called her Shinna-san, welcome again to Toram!!"#,
        );
    }

    #[test]
    fn calling_it_otp() {
        assert_suggestion_result(
            r#"Calling it as OTP in this case misleading"#,
            CallThem::default(),
            r#"Calling it OTP in this case misleading"#,
        );
    }

    #[test]
    fn call_it_procrastination() {
        assert_suggestion_result(
            r#"To summarise it in just one word I would call it as procrastination."#,
            CallThem::default(),
            r#"To summarise it in just one word I would call it procrastination."#,
        );
    }

    #[test]
    fn call_her_important() {
        assert_suggestion_result(
            r#"Liked the article overall but to call her as important to rap as Jay or Dre is a bold overstatement."#,
            CallThem::default(),
            r#"Liked the article overall but to call her important to rap as Jay or Dre is a bold overstatement."#,
        );
    }

    #[test]
    fn call_him_kindles() {
        assert_suggestion_result(
            r#"The days when I had my first best friend, I would rather call him as human version of kindle audiobook, who keeps on talking about everything under the umbrella."#,
            CallThem::default(),
            r#"The days when I had my first best friend, I would rather call him human version of kindle audiobook, who keeps on talking about everything under the umbrella."#,
        );
    }

    #[test]
    fn call_them_defenders() {
        assert_suggestion_result(
            r#"Declaring war challenging land of a vassal should call them as defenders!"#,
            CallThem::default(),
            r#"Declaring war challenging land of a vassal should call them defenders!"#,
        );
    }

    #[test]
    fn call_it_magical() {
        assert_suggestion_result(
            r#"I would like to call it as magical."#,
            CallThem::default(),
            r#"I would like to call it magical."#,
        );
    }

    #[test]
    fn forward_lateral() {
        assert_suggestion_result(
            r#"Surprised the refs didn’t call this as a forward lateral."#,
            CallThem::default(),
            r#"Surprised the refs didn’t call this a forward lateral."#,
        );
    }

    #[test]
    fn calling_best_friend() {
        assert_suggestion_result(
            r#"Meet my buddy! I love calling him as my best friend, because he never failed to bring some cheer in me!"#,
            CallThem::default(),
            r#"Meet my buddy! I love calling him my best friend, because he never failed to bring some cheer in me!"#,
        );
    }

    #[test]
    fn calling_everyone_titles() {
        assert_suggestion_result(
            r#"Currently, I’m teaching in Asia and the students have the local custom of calling everyone as Mr. Givenname or Miss Givenname"#,
            CallThem::default(),
            r#"Currently, I’m teaching in Asia and the students have the local custom of calling everyone Mr. Givenname or Miss Givenname"#,
        );
    }

    #[test]
    fn called_as_he() {
        assert_suggestion_result(
            r#"I prefer to be called as he when referred in 3rd person and I’m sure that everyone would be ok to call me as he."#,
            CallThem::default(),
            r#"I prefer to be called he when referred in 3rd person and I’m sure that everyone would be ok to call me he."#,
        );
    }

    #[test]
    fn calls_him_bob() {
        assert_suggestion_result(
            r#"In Twelve Monkeys, Cole hears someone who calls him as “Bob”"#,
            CallThem::default(),
            r#"In Twelve Monkeys, Cole hears someone who calls him “Bob”"#,
        );
    }

    #[test]
    fn pliny_called_it() {
        assert_suggestion_result(
            r#"Pliny the Elder called it as lake of Gennesaret or Taricheae in his encyclopedia, Natural History."#,
            CallThem::default(),
            r#"Pliny the Elder called it lake of Gennesaret or Taricheae in his encyclopedia, Natural History."#,
        );
    }

    #[test]
    fn students_call_you() {
        assert_suggestion_result(
            r#"In the same way your students will call you as ~先生 even after they graduated/move to higher education."#,
            CallThem::default(),
            r#"In the same way your students will call you ~先生 even after they graduated/move to higher education."#,
        );
    }

    #[test]
    fn paradoxical_reaction() {
        assert_suggestion_result(
            r#"We can call it as Paradoxical Reaction which means a medicine which is used to reduce pain increases the pain when it is"#,
            CallThem::default(),
            r#"We can call it Paradoxical Reaction which means a medicine which is used to reduce pain increases the pain when it is"#,
        );
    }

    #[test]
    fn rust_module() {
        assert_no_lints(
            "I want to call them as if they were just another Rust module",
            CallThem::default(),
        );
    }

    #[test]
    fn want_to_do() {
        assert_no_lints(
            "however its a design choice to not call it as it does things I don't want to do.",
            CallThem::default(),
        );
    }
}
