use crate::{
    Lint, Token, TokenStringExt,
    expr::{Expr, FirstMatchOf, FixedPhrase, SequenceExpr},
    linting::{ExprLinter, LintKind, Suggestion, expr_linter::Sentence},
    patterns::WordSet,
};

pub struct WebScraping {
    expr: FirstMatchOf,
}

impl Default for WebScraping {
    fn default() -> Self {
        let scrap_verbs = &["scrap", "scrapped", "scraps", "scrapping"][..];

        let scrap_nouns = &["scrapper", "scrappers"][..];

        let all_scraps = &[scrap_verbs, scrap_nouns].concat();

        let mut closed_compound_word_set = WordSet::new(&[]);
        let mut open_compound_vec = vec![];
        let mut hyphenated_compounds_vec = vec![];

        all_scraps.iter().for_each(|s| {
            closed_compound_word_set.add(format!("web{}", s).as_str());
            open_compound_vec.push(
                Box::new(FixedPhrase::from_phrase(format!("web {}", s).as_str())) as Box<dyn Expr>,
            );
            hyphenated_compounds_vec.push(Box::new(FixedPhrase::from_phrase(
                format!("web-{}", s).as_str(),
            )) as Box<dyn Expr>);
        });

        let open_compound_first_match_of = FirstMatchOf::new(open_compound_vec);
        let hyphenated_compounds_first_match_of = FirstMatchOf::new(hyphenated_compounds_vec);

        let web_scraps = FirstMatchOf::new(vec![
            Box::new(closed_compound_word_set),
            Box::new(open_compound_first_match_of),
            Box::new(hyphenated_compounds_first_match_of),
        ]);

        let scrapables = &[
            "article",
            "articles",
            "content",
            "doc",
            "docs",
            "document",
            "documents",
            "dom",
            "html",
            "info",
            "information",
            "page",
            "pages",
            "site",
            "sites",
            "text",
            "web",
            "webpage",
            "webpages",
            "website",
            "websites",
        ];

        // An explicitly non-greedy expression. Counterintuitively, greedy doesn't always match.
        let scrap_the_web = SequenceExpr::word_set(scrap_verbs)
            .t_ws()
            .then_zero_or_more(
                SequenceExpr::with(|t: &Token, s: &[char]| {
                    t.kind.is_word() && !scrapables.contains(&t.get_str(s).as_str())
                })
                .t_ws(),
            )
            .then_word_set(scrapables);

        Self {
            expr: FirstMatchOf::new(vec![Box::new(web_scraps), Box::new(scrap_the_web)]),
        }
    }
}

fn match_web_then_scrap(toks: &[Token], src: &[char]) -> Option<Lint> {
    if ![1, 3].contains(&toks.len()) {
        return None;
    }

    let (web, sep, scrap) = if toks.len() == 1 {
        let (w, s) = toks[0].get_ch(src).split_at(3);
        (w, &[] as &[char], s) // No separator in the 1-token case
    } else {
        (
            toks[0].get_ch(src),
            toks[1].get_ch(src),
            toks[2].get_ch(src),
        )
    };

    // Standardize the prefix (web + optional separator)
    let prefix = web.iter().chain(sep).copied();

    // Generalize the "scrap" -> "scrape" logic
    let replacement_value = match scrap.len() {
        5 | 6 => prefix
            .chain(scrap.iter().take(5).copied())
            .chain(std::iter::once('e'))
            .chain(scrap.iter().skip(5).copied())
            .collect(),
        _ => prefix
            .chain(scrap.iter().take(5).copied())
            .chain(scrap.iter().skip(6).copied())
            .collect(),
    };

    Some(Lint {
        span: toks.span()?,
        lint_kind: LintKind::Eggcorn,
        suggestions: vec![Suggestion::replace_with_match_case(
            replacement_value,
            toks.span()?.get_content(src),
        )],
        message:
            "`Scrap` means `discard`. The word for gathering information from websites is `scrape`."
                .to_string(),
        ..Default::default()
    })
}

fn match_scrap_then_web(toks: &[Token], src: &[char]) -> Option<Lint> {
    let scrap = toks[0].get_ch(src);

    // Generalize the "scrap" -> "scrape" logic
    let replacement_value = match scrap.len() {
        5 | 6 => scrap
            .iter()
            .take(5)
            .copied()
            .chain(std::iter::once('e'))
            .chain(scrap.iter().skip(5).copied())
            .collect(),
        _ => scrap
            .iter()
            .take(5)
            .copied()
            .chain(scrap.iter().skip(6).copied())
            .collect(),
    };

    Some(Lint {
        span: toks[0].span,
        lint_kind: LintKind::Eggcorn,
        suggestions: vec![Suggestion::replace_with_match_case(
            replacement_value,
            toks[0].span.get_content(src),
        )],
        message:
            "`Scrap` means `discard`. The word for gathering information from websites is `scrape`."
                .to_string(),
        ..Default::default()
    })
}

impl ExprLinter for WebScraping {
    type Unit = Sentence;

    fn match_to_lint(&self, toks: &[Token], src: &[char]) -> Option<Lint> {
        match toks.first()?.get_ch(src).first()?.to_ascii_lowercase() {
            'w' => match_web_then_scrap(toks, src),
            's' => match_scrap_then_web(toks, src),
            _ => None,
        }
    }
    fn expr(&self) -> &dyn Expr {
        &self.expr
    }

    fn description(&self) -> &'static str {
        "Corrects `scrapping` the web to `scraping`."
    }
}

#[cfg(test)]
mod tests {
    mod web_scrape {
        mod closed {
            use crate::linting::tests::assert_suggestion_result;

            use super::super::super::WebScraping;

            #[test]
            fn scrap() {
                assert_suggestion_result("webscrap", WebScraping::default(), "webscrape");
            }

            #[test]
            fn scraps() {
                assert_suggestion_result("webscraps", WebScraping::default(), "webscrapes");
            }

            #[test]
            fn scrapped() {
                assert_suggestion_result("webscrapped", WebScraping::default(), "webscraped");
            }

            #[test]
            fn scrapper() {
                assert_suggestion_result("webscrapper", WebScraping::default(), "webscraper");
            }

            #[test]
            fn scrappers() {
                assert_suggestion_result("webscrappers", WebScraping::default(), "webscrapers");
            }

            #[test]
            fn scrapping() {
                assert_suggestion_result("webscrapping", WebScraping::default(), "webscraping");
            }
        }

        mod open {
            use crate::linting::tests::assert_suggestion_result;

            use super::super::super::WebScraping;

            #[test]
            fn scrap() {
                assert_suggestion_result("web scrap", WebScraping::default(), "web scrape");
            }

            #[test]
            fn scraps() {
                assert_suggestion_result("web scraps", WebScraping::default(), "web scrapes");
            }

            #[test]
            fn scrapped() {
                assert_suggestion_result("web scrapped", WebScraping::default(), "web scraped");
            }

            #[test]
            fn scrapper() {
                assert_suggestion_result("web scrapper", WebScraping::default(), "web scraper");
            }

            #[test]
            fn scrappers() {
                assert_suggestion_result("web scrappers", WebScraping::default(), "web scrapers");
            }

            #[test]
            fn scrapping() {
                assert_suggestion_result("web scrapping", WebScraping::default(), "web scraping");
            }
        }

        mod hyphenated {
            use crate::linting::tests::assert_suggestion_result;

            use super::super::super::WebScraping;

            #[test]
            fn scrap() {
                assert_suggestion_result("web-scrap", WebScraping::default(), "web-scrape");
            }

            #[test]
            fn scraps() {
                assert_suggestion_result("web-scraps", WebScraping::default(), "web-scrapes");
            }

            #[test]
            fn scrapped() {
                assert_suggestion_result("web-scrapped", WebScraping::default(), "web-scraped");
            }

            #[test]
            fn scrapper() {
                assert_suggestion_result("web-scrapper", WebScraping::default(), "web-scraper");
            }

            #[test]
            fn scrappers() {
                assert_suggestion_result("web-scrappers", WebScraping::default(), "web-scrapers");
            }

            #[test]
            fn scrapping() {
                assert_suggestion_result("web-scrapping", WebScraping::default(), "web-scraping");
            }
        }

        mod harvested_examples {
            use crate::linting::tests::assert_suggestion_result;

            use super::super::super::WebScraping;

            #[test]
            fn web_scrap_lowercase() {
                assert_suggestion_result(
                    "The goal of the project is to web scrap data from all pages of the website with capability of handling exceptions.",
                    WebScraping::default(),
                    "The goal of the project is to web scrape data from all pages of the website with capability of handling exceptions.",
                );
            }

            #[test]
            fn web_scrap_uppercase() {
                assert_suggestion_result(
                    "Web Scrap on Jabama website to generate and analyze a dataset",
                    WebScraping::default(),
                    "Web Scrape on Jabama website to generate and analyze a dataset",
                );
            }

            #[test]
            fn web_scrapped() {
                assert_suggestion_result(
                    "Web scrapped an amazon page , automated the scraping, stored the data in csv file and created an email alert when the drop prices",
                    WebScraping::default(),
                    "Web scraped an amazon page , automated the scraping, stored the data in csv file and created an email alert when the drop prices",
                );
            }

            #[test]
            fn web_scrapped_titlecase() {
                assert_suggestion_result(
                    "This project uses the data collected (Web Scrapped) from a website that list the houses for sale in Rwanda",
                    WebScraping::default(),
                    "This project uses the data collected (Web Scraped) from a website that list the houses for sale in Rwanda",
                );
            }

            #[test]
            fn web_scrapped_hyphenated() {
                assert_suggestion_result(
                    "Web-Scrapped Datasets",
                    WebScraping::default(),
                    "Web-Scraped Datasets",
                );
            }

            #[test]
            fn web_scrapper_titlecase() {
                assert_suggestion_result(
                    "Web Scrapper Built Using Golang.",
                    WebScraping::default(),
                    "Web Scraper Built Using Golang.",
                );
            }

            #[test]
            fn web_scrappers_lowercase() {
                assert_suggestion_result(
                    "Internet bots and web scrappers that will save a lot of your time!",
                    WebScraping::default(),
                    "Internet bots and web scrapers that will save a lot of your time!",
                );
            }

            #[test]
            fn web_scrappers_hyphenated() {
                assert_suggestion_result(
                    "A Collection of web-scrappers with GUI written in Pyside6/PyQt6.",
                    WebScraping::default(),
                    "A Collection of web-scrapers with GUI written in Pyside6/PyQt6.",
                );
            }

            #[test]
            fn web_scrapping_lowercase() {
                assert_suggestion_result(
                    "ScrapPaper is a web scrapping method to extract journal information from PubMed and Google Scholar using Python script.",
                    WebScraping::default(),
                    "ScrapPaper is a web scraping method to extract journal information from PubMed and Google Scholar using Python script.",
                );
            }

            #[test]
            fn web_scrapping_titlecase() {
                assert_suggestion_result(
                    "Web Scrapping Examples using Beautiful Soup in Python.",
                    WebScraping::default(),
                    "Web Scraping Examples using Beautiful Soup in Python.",
                );
            }

            #[test]
            fn web_scrapping_hyphenated() {
                assert_suggestion_result(
                    "some websites allow web-scrapping some don't.",
                    WebScraping::default(),
                    "some websites allow web-scraping some don't.",
                );
            }

            #[test]
            fn webscrapped() {
                assert_suggestion_result(
                    "Example of webscrapped document : click here.",
                    WebScraping::default(),
                    "Example of webscraped document : click here.",
                );
            }

            #[test]
            fn webscrapper() {
                assert_suggestion_result(
                    "A webscrapper to scrape all the words and their meanings from urban dictionary.",
                    WebScraping::default(),
                    "A webscraper to scrape all the words and their meanings from urban dictionary.",
                );
            }

            #[test]
            fn webscrappers_capitalized() {
                assert_suggestion_result(
                    "A collection of Webscrappers I built using Scrapy while learning it hands on - SIdR4g/Scrapy_practice.",
                    WebScraping::default(),
                    "A collection of Webscrapers I built using Scrapy while learning it hands on - SIdR4g/Scrapy_practice.",
                );
            }

            #[test]
            fn webscrappers_camelcase() {
                assert_suggestion_result(
                    "Awesome-WebScrappers. Collection of powerful and efficient web scrapers built using Python and BeautifulSoup.",
                    WebScraping::default(),
                    "Awesome-WebScrapers. Collection of powerful and efficient web scrapers built using Python and BeautifulSoup.",
                );
            }

            #[test]
            fn webscrapping() {
                assert_suggestion_result(
                    "Webscrapping to identify and download latest pdf documents.",
                    WebScraping::default(),
                    "Webscraping to identify and download latest pdf documents.",
                );
            }

            #[test]
            fn webscraps_lowercase() {
                assert_suggestion_result(
                    "gostapafor is a tool that webscraps and forwards html pages to other consumers",
                    WebScraping::default(),
                    "gostapafor is a tool that webscrapes and forwards html pages to other consumers",
                );
            }

            #[test]
            fn webscraps_camelcase() {
                assert_suggestion_result(
                    "WebScraps the University of California, Santa Cruz's menu and texts it to the user for Breakfast, Lunch, Dinner, and Late Night.",
                    WebScraping::default(),
                    "WebScrapes the University of California, Santa Cruz's menu and texts it to the user for Breakfast, Lunch, Dinner, and Late Night.",
                );
            }
        }
    }

    mod scrape_the_web {
        use crate::linting::tests::assert_suggestion_result;

        use super::super::WebScraping;

        #[test]
        fn scrap_page() {
            assert_suggestion_result("scrap page", WebScraping::default(), "scrape page");
        }

        #[test]
        fn scrapped_pages() {
            assert_suggestion_result("scrapped pages", WebScraping::default(), "scraped pages");
        }

        #[test]
        fn scraps_html() {
            assert_suggestion_result("scraps html", WebScraping::default(), "scrapes html");
        }

        #[test]
        fn scrapping_web() {
            assert_suggestion_result("scrapping web", WebScraping::default(), "scraping web");
        }

        #[test]
        fn scrapping_web_all_caps() {
            assert_suggestion_result("SCRAPPING WEB", WebScraping::default(), "SCRAPING WEB");
        }

        #[test]
        fn scrapping_web_mixed_case() {
            assert_suggestion_result("Scrapping Web", WebScraping::default(), "Scraping Web");
        }

        #[test]
        fn scrap_html() {
            assert_suggestion_result(
                "Scrap a website's HTML, CSS and JS using this API.",
                WebScraping::default(),
                "Scrape a website's HTML, CSS and JS using this API.",
            );
        }

        #[test]
        fn scrapped_news_articles() {
            assert_suggestion_result(
                "Scrapped various news article including all the details using scrapy framework.",
                WebScraping::default(),
                "Scrapped various news article including all the details using scrapy framework.",
            );
        }

        #[test]
        fn scrapping_different_websites() {
            assert_suggestion_result(
                "This repository contains Python scripts based on Scrapping different websites.",
                WebScraping::default(),
                "This repository contains Python scripts based on Scraping different websites.",
            );
        }

        #[test]
        fn scrapping_web_content() {
            assert_suggestion_result(
                "Scrapping web content using PHP and Python.",
                WebScraping::default(),
                "Scraping web content using PHP and Python.",
            );
        }

        #[test]
        fn scraps_from_pages() {
            assert_suggestion_result(
                "It requests the website and scraps news from different pages.",
                WebScraping::default(),
                "It requests the website and scrapes news from different pages.",
            );
        }
    }
}
