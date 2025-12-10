use crate::{
    Lint, Token,
    expr::{Expr, FirstMatchOf, FixedPhrase, SequenceExpr},
    linting::{ExprLinter, debug::format_lint_match, expr_linter::Chunk},
};

pub struct FascinatedAbout {
    expr: Box<dyn Expr>,
}

impl Default for FascinatedAbout {
    fn default() -> Self {
        Self {
            expr: Box::new(FirstMatchOf::new(vec![
                Box::new(SequenceExpr::aco("fascinated").t_ws().then_preposition()),
                Box::new(
                    SequenceExpr::aco("available")
                        .t_ws()
                        .t_aco("at")
                        .t_ws()
                        .then_possessive_determiner()
                        .t_ws()
                        .t_aco("disposal"),
                ),
                Box::new(SequenceExpr::word_set(&["tell", "tells", "told", "telling"]).t_ws().t_aco("about")),
            ])),
        }
    }
}

impl ExprLinter for FascinatedAbout {
    type Unit = Chunk;

    fn expr(&self) -> &dyn Expr {
        self.expr.as_ref()
    }

    fn match_to_lint_with_context(
        &self,
        toks: &[Token],
        src: &[char],
        ctx: Option<(&[Token], &[Token])>,
    ) -> Option<Lint> {
        eprintln!("🏆 {}", format_lint_match(toks, ctx, src));
        None
    }

    fn description(&self) -> &str {
        "Checks for unusual prepositions used with `fascinated`."
    }
}

#[cfg(test)]
mod tests {
    use crate::linting::{
        fascinated_about::FascinatedAbout,
        tests::{assert_good_and_bad_suggestions, assert_suggestion_result},
    };

    // fascinated about

    #[test]
    fn fascinated_about() {
        assert_good_and_bad_suggestions(
            "Now, one aspect of the Amiga that I've always been fascinated about is making my own games for the Amiga.",
            FascinatedAbout::default(),
            &[
                "Now, one aspect of the Amiga that I've always been fascinated by is making my own games for the Amiga.",
                "Now, one aspect of the Amiga that I've always been fascinated with is making my own games for the Amiga.",
            ][..],
            &[],
        );
    }

    // also why I am very fascinated about the micro:bit itself
    // Self-learner, fascinated about software development, especially computer graphics and web - marcus-phi.
    // Fascinated about Computer Science, Finance and Statistics.
    // m relatively new to deCONZ and Conbee2 but already very fascinated about the possibilities compared to Philips and Ikea's
    // I have been using browser use in local mode for a while and i am pretty fascinated about the project.
    // Hey guys, I am really fascinated about your work and I tried to build Magisk so I will be able to contribute for the project.
    // I am a retired Dutch telecom engineer and fascinated about AIS applications.
    // Software Developer fascinated about innovative ideas, love to learn and share new technologies and ideas.
    // m fascinated about coding and and sharing my code to the world.

    // available at one's disposal

    #[test]
    fn available_at_your_disposal() {
        assert_good_and_bad_suggestions(
            "1/2 of the capacity of the total amount of sprites available at your disposal",
            FascinatedAbout::default(),
            &[
                "1/2 of the capacity of the total amount of sprites available",
                "1/2 of the capacity of the total amount of sprites at your disposal",
            ][..],
            &[],
        );
    }

    // Since now you can start marking code with your attribute and after recompilation, the generated code should be available at your disposal.
    // If not, what are the other options available at my disposal?

    // tell about

    #[test]
    fn tell_about() {
        assert_good_and_bad_suggestions(
            "I'm going to tell about why we started to restore it.",
            FascinatedAbout::default(),
            &[
                "I'm going to talk about why we started to restore it.",
                "I'm going to tell you about why we started to restore it.",
                // TODO: describe
                // TODO: explain
                // TODO: show
            ][..],
            &[],
        );
    }

    // A Frame which tells about your Fame.
    // This repo tells about how to mirror OS using ZFS on Ubuntu 24.4
    // Here's I'm telling about how to install Fast cli tool in termux without root device.
    // This is my personal repo in which readme tells about my culture and background , and tech , places etc
    // This my readme file which will tell about me and give details about my work.
    // Seven Wonders is an Alexa Skill which tells about all the seven wonders of the world.
    // Nothing to tell about.
    // Well, there is not much to tell about... chrizator has 5 repositories available.
    // Repo telling about discord bot that runs programs in different languages on remote server
    // My personal portfolio website which tells about my skills etc.
    // Pay attention. Be astonished. Tell about it.
    // This website is my portfolio that tell about me like skills, projects, experiences, and services with other additional data.
    // Molecular transformer multiblock description tells about tungstensteel coils
    // A repository telling about the success of the preparation of the "LittleRobot" team for the WRO "Future Engineers" competitions.
}
