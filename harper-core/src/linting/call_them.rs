use std::{ops::Range, sync::Arc};

use crate::expr::{Expr, ExprMap, SequenceExpr};
use crate::patterns::DerivedFrom;
use crate::{Token, TokenStringExt};

use super::{ExprLinter, Lint, LintKind, Suggestion};

pub struct CallThem {
    expr: Box<dyn Expr>,
    map: Arc<ExprMap<Range<usize>>>,
}

impl Default for CallThem {
    fn default() -> Self {
        let mut map = ExprMap::default();

        map.insert(
            SequenceExpr::default()
                .then(DerivedFrom::new_from_str("call"))
                .t_ws()
                .then_pronoun()
                .t_ws()
                .t_aco("as"),
            3..5,
        );

        map.insert(
            SequenceExpr::default()
                .then(DerivedFrom::new_from_str("call"))
                .t_ws()
                .t_aco("as")
                .t_ws()
                .then_pronoun(),
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
    use crate::linting::tests::assert_suggestion_result;

    use super::CallThem;

    fn assert_fix(original: &str, expected: &str) {
        assert_suggestion_result(original, CallThem::default(), expected);
    }

    #[test]
    fn prefer_plug_and_receptacle() {
        assert_fix(
            r#"I prefer to call them as Plug (male) and Receptacle (female). Receptacles are seen in laptops, mobile phones etc.."#,
            r#"I prefer to call them Plug (male) and Receptacle (female). Receptacles are seen in laptops, mobile phones etc.."#,
        );
    }

    #[test]
    fn builtins_id() {
        assert_fix(
            r#"I’d categorically ignore *id* as a builtin, and when you do need it in a module, make it super explicit and `import builtins` and call it as `builtins.id`."#,
            r#"I’d categorically ignore *id* as a builtin, and when you do need it in a module, make it super explicit and `import builtins` and call it `builtins.id`."#,
        );
    }

    #[test]
    fn non_modal_dialogue() {
        assert_fix(
            r#"We usually call it as non-modal dialogue e.g. when hit Gmail compose button, a nonmodal dialogue opens."#,
            r#"We usually call it non-modal dialogue e.g. when hit Gmail compose button, a nonmodal dialogue opens."#,
        );
    }

    #[test]
    fn prefer_to_call_them() {
        assert_fix(
            r#"So, how do you typically prefer to call them as?"#,
            r#"So, how do you typically prefer to call them?"#,
        );
    }

    #[test]
    fn called_them_allies() {
        assert_fix(
            r#"Yes as tribes or nomads you called them as allies but you didn’t get their levies as your own."#,
            r#"Yes as tribes or nomads you called them allies but you didn’t get their levies as your own."#,
        );
    }

    #[test]
    fn character_development() {
        assert_fix(
            r#"I call this as character development."#,
            r#"I call this character development."#,
        );
    }

    #[test]
    fn fate_or_time() {
        assert_fix(
            r#"Should I Call It As Fate Or Time"#,
            r#"Should I Call It Fate Or Time"#,
        );
    }

    #[test]
    fn abstract_latte_art() {
        assert_fix(
            r#"Can we just call it as abstract latte art."#,
            r#"Can we just call it abstract latte art."#,
        );
    }

    #[test]
    fn sounding_boards() {
        assert_fix(
            r#"I call them as my ‘sounding boards’"#,
            r#"I call them my ‘sounding boards’"#,
        );
    }

    #[test]
    fn calling_them_disaster() {
        assert_fix(
            r#"I totally disagree with your point listed and calling them as disaster."#,
            r#"I totally disagree with your point listed and calling them disaster."#,
        );
    }

    #[test]
    fn battle_of_boxes() {
        assert_fix(
            r#"Windows Sandbox and VirtualBox or I would like to call this as “Battle of Boxes.”"#,
            r#"Windows Sandbox and VirtualBox or I would like to call this “Battle of Boxes.”"#,
        );
    }

    #[test]
    fn called_her_shinnasan() {
        assert_fix(
            r#"Nice meeting a follower from reddit I called her as Shinna-san, welcome again to Toram!!"#,
            r#"Nice meeting a follower from reddit I called her Shinna-san, welcome again to Toram!!"#,
        );
    }

    #[test]
    fn calling_it_otp() {
        assert_fix(
            r#"Calling it as OTP in this case misleading"#,
            r#"Calling it OTP in this case misleading"#,
        );
    }

    #[test]
    fn call_it_procrastination() {
        assert_fix(
            r#"To summarise it in just one word I would call it as procrastination."#,
            r#"To summarise it in just one word I would call it procrastination."#,
        );
    }

    #[test]
    fn call_her_important() {
        assert_fix(
            r#"Liked the article overall but to call her as important to rap as Jay or Dre is a bold overstatement."#,
            r#"Liked the article overall but to call her important to rap as Jay or Dre is a bold overstatement."#,
        );
    }

    #[test]
    fn call_him_kindles() {
        assert_fix(
            r#"The days when I had my first best friend, I would rather call him as human version of kindle audiobook, who keeps on talking about everything under the umbrella."#,
            r#"The days when I had my first best friend, I would rather call him human version of kindle audiobook, who keeps on talking about everything under the umbrella."#,
        );
    }

    #[test]
    fn call_them_defenders() {
        assert_fix(
            r#"Declaring war challenging land of a vassal should call them as defenders!"#,
            r#"Declaring war challenging land of a vassal should call them defenders!"#,
        );
    }

    #[test]
    fn call_it_magical() {
        assert_fix(
            r#"I would like to call it as magical."#,
            r#"I would like to call it magical."#,
        );
    }

    #[test]
    fn forward_lateral() {
        assert_fix(
            r#"Surprised the refs didn’t call this as a forward lateral."#,
            r#"Surprised the refs didn’t call this a forward lateral."#,
        );
    }

    #[test]
    fn calling_best_friend() {
        assert_fix(
            r#"Meet my buddy! I love calling him as my best friend, because he never failed to bring some cheer in me!"#,
            r#"Meet my buddy! I love calling him my best friend, because he never failed to bring some cheer in me!"#,
        );
    }

    #[test]
    fn calling_everyone_titles() {
        assert_fix(
            r#"Currently, I’m teaching in Asia and the students have the local custom of calling everyone as Mr. Givenname or Miss Givenname"#,
            r#"Currently, I’m teaching in Asia and the students have the local custom of calling everyone Mr. Givenname or Miss Givenname"#,
        );
    }

    #[test]
    fn called_as_he() {
        assert_fix(
            r#"I prefer to be called as he when referred in 3rd person and I’m sure that everyone would be ok to call me as he."#,
            r#"I prefer to be called he when referred in 3rd person and I’m sure that everyone would be ok to call me he."#,
        );
    }

    #[test]
    fn calls_him_bob() {
        assert_fix(
            r#"In Twelve Monkeys, Cole hears someone who calls him as “Bob”"#,
            r#"In Twelve Monkeys, Cole hears someone who calls him “Bob”"#,
        );
    }

    #[test]
    fn pliny_called_it() {
        assert_fix(
            r#"Pliny the Elder called it as lake of Gennesaret or Taricheae in his encyclopedia, Natural History."#,
            r#"Pliny the Elder called it lake of Gennesaret or Taricheae in his encyclopedia, Natural History."#,
        );
    }

    #[test]
    fn students_call_you() {
        assert_fix(
            r#"In the same way your students will call you as ~先生 even after they graduated/move to higher education."#,
            r#"In the same way your students will call you ~先生 even after they graduated/move to higher education."#,
        );
    }

    #[test]
    fn paradoxical_reaction() {
        assert_fix(
            r#"We can call it as Paradoxical Reaction which means a medicine which is used to reduce pain increases the pain when it is"#,
            r#"We can call it Paradoxical Reaction which means a medicine which is used to reduce pain increases the pain when it is"#,
        );
    }
}
