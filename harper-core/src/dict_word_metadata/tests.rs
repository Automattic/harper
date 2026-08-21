use crate::DictWordMetadata;
use crate::spell::{Dictionary, FstDictionary};

// Helper function to get metadata from the curated dictionary
pub fn md(word: &str) -> DictWordMetadata {
    FstDictionary::curated()
        .get_word_metadata_str(word)
        .unwrap_or_else(|| panic!("Word '{word}' not found in dictionary"))
        .into_owned()
}

mod noun {
    use crate::dict_word_metadata::tests::md;

    #[test]
    fn puppy_is_noun() {
        assert!(md("puppy").is_noun());
    }

    #[test]
    fn prepare_is_not_noun() {
        assert!(!md("prepare").is_noun());
    }

    #[test]
    fn paris_is_proper_noun() {
        assert!(md("Paris").is_proper_noun());
    }

    #[test]
    fn permit_is_non_proper_noun() {
        assert!(md("lapdog").is_non_proper_noun());
    }

    #[test]
    fn hound_is_singular_noun() {
        assert!(md("hound").is_singular_noun());
    }

    #[test]
    fn pooches_is_non_singular_noun() {
        assert!(md("pooches").is_non_singular_noun());
    }

    // Make sure is_non_xxx_noun methods don't behave like is_not_xxx_noun.
    // In other words, make sure they don't return true for words that are not nouns.
    // They must only pass for words that are nouns but not singular etc.
    #[test]
    fn loyal_doesnt_pass_is_non_singular_noun() {
        assert!(!md("loyal").is_non_singular_noun());
    }

    #[test]
    fn hounds_is_plural_noun() {
        assert!(md("hounds").is_plural_noun());
    }

    #[test]
    fn pooch_is_non_plural_noun() {
        assert!(md("pooch").is_non_plural_noun());
    }

    #[test]
    fn fish_is_singular_noun() {
        assert!(md("fish").is_singular_noun());
    }

    #[test]
    fn fish_is_plural_noun() {
        assert!(md("fish").is_plural_noun());
    }

    #[test]
    fn fishes_is_plural_noun() {
        assert!(md("fishes").is_plural_noun());
    }

    #[test]
    fn sheep_is_singular_noun() {
        assert!(md("sheep").is_singular_noun());
    }

    #[test]
    fn sheep_is_plural_noun() {
        assert!(md("sheep").is_plural_noun());
    }

    #[test]
    #[should_panic]
    fn sheeps_is_not_word() {
        md("sheeps");
    }

    #[test]
    fn bicep_is_singular_noun() {
        assert!(md("bicep").is_singular_noun());
    }

    #[test]
    fn biceps_is_singular_noun() {
        assert!(md("biceps").is_singular_noun());
    }

    #[test]
    fn biceps_is_plural_noun() {
        assert!(md("biceps").is_plural_noun());
    }

    #[test]
    fn aircraft_is_singular_noun() {
        assert!(md("aircraft").is_singular_noun());
    }

    #[test]
    fn aircraft_is_plural_noun() {
        assert!(md("aircraft").is_plural_noun());
    }

    #[test]
    #[should_panic]
    fn aircrafts_is_not_word() {
        md("aircrafts");
    }

    #[test]
    fn dog_apostrophe_s_is_possessive_noun() {
        assert!(md("dog's").is_possessive_noun());
    }

    #[test]
    fn dogs_is_non_possessive_noun() {
        assert!(md("dogs").is_non_possessive_noun());
    }

    // noun countability

    #[test]
    fn dog_is_countable() {
        assert!(md("dog").is_countable_noun());
    }
    #[test]
    fn dog_is_non_mass_noun() {
        assert!(md("dog").is_non_mass_noun());
    }

    #[test]
    fn furniture_is_mass_noun() {
        assert!(md("furniture").is_mass_noun());
    }
    #[test]
    fn furniture_is_non_countable_noun() {
        assert!(md("furniture").is_non_countable_noun());
    }

    #[test]
    fn equipment_is_mass_noun() {
        assert!(md("equipment").is_mass_noun());
    }
    #[test]
    fn equipment_is_non_countable_noun() {
        assert!(md("equipment").is_non_countable_noun());
    }

    #[test]
    fn beer_is_countable_noun() {
        assert!(md("beer").is_countable_noun());
    }
    #[test]
    fn beer_is_mass_noun() {
        assert!(md("beer").is_mass_noun());
    }
}

mod pronoun {
    use crate::dict_word_metadata::tests::md;

    mod i_me_myself {
        use crate::dict_word_metadata::tests::md;

        #[test]
        fn i_is_pronoun() {
            assert!(md("I").is_pronoun());
        }
        #[test]
        fn i_is_personal_pronoun() {
            assert!(md("I").is_personal_pronoun());
        }
        #[test]
        fn i_is_singular_pronoun() {
            assert!(md("I").is_singular_pronoun());
        }
        #[test]
        fn i_is_subject_pronoun() {
            assert!(md("I").is_subject_pronoun());
        }

        #[test]
        fn me_is_pronoun() {
            assert!(md("me").is_pronoun());
        }
        #[test]
        fn me_is_personal_pronoun() {
            assert!(md("me").is_personal_pronoun());
        }
        #[test]
        fn me_is_singular_pronoun() {
            assert!(md("me").is_singular_pronoun());
        }
        #[test]
        fn me_is_object_pronoun() {
            assert!(md("me").is_object_pronoun());
        }

        #[test]
        fn myself_is_pronoun() {
            assert!(md("myself").is_pronoun());
        }
        #[test]
        fn myself_is_personal_pronoun() {
            assert!(md("myself").is_personal_pronoun());
        }
        #[test]
        fn myself_is_singular_pronoun() {
            assert!(md("myself").is_singular_pronoun());
        }
        #[test]
        fn myself_is_reflexive_pronoun() {
            assert!(md("myself").is_reflexive_pronoun());
        }
    }

    mod we_us_ourselves {
        use crate::dict_word_metadata::tests::md;

        #[test]
        fn we_is_pronoun() {
            assert!(md("we").is_pronoun());
        }
        #[test]
        fn we_is_personal_pronoun() {
            assert!(md("we").is_personal_pronoun());
        }
        #[test]
        fn we_is_plural_pronoun() {
            assert!(md("we").is_plural_pronoun());
        }
        #[test]
        fn we_is_subject_pronoun() {
            assert!(md("we").is_subject_pronoun());
        }

        #[test]
        fn us_is_pronoun() {
            assert!(md("us").is_pronoun());
        }
        #[test]
        fn us_is_personal_pronoun() {
            assert!(md("us").is_personal_pronoun());
        }
        #[test]
        fn us_is_plural_pronoun() {
            assert!(md("us").is_plural_pronoun());
        }
        #[test]
        fn us_is_object_pronoun() {
            assert!(md("us").is_object_pronoun());
        }

        #[test]
        fn ourselves_is_pronoun() {
            assert!(md("ourselves").is_pronoun());
        }
        #[test]
        fn ourselves_is_personal_pronoun() {
            assert!(md("ourselves").is_personal_pronoun());
        }
        #[test]
        fn ourselves_is_plural_pronoun() {
            assert!(md("ourselves").is_plural_pronoun());
        }
        #[test]
        fn ourselves_is_reflexive_pronoun() {
            assert!(md("ourselves").is_reflexive_pronoun());
        }
    }

    mod you_yourself {
        use crate::dict_word_metadata::tests::md;

        #[test]
        fn you_is_pronoun() {
            assert!(md("you").is_pronoun());
        }
        #[test]
        fn you_is_personal_pronoun() {
            assert!(md("you").is_personal_pronoun());
        }
        #[test]
        fn you_is_singular_pronoun() {
            assert!(md("you").is_singular_pronoun());
        }
        #[test]
        fn you_is_plural_pronoun() {
            assert!(md("you").is_plural_pronoun());
        }
        #[test]
        fn you_is_subject_pronoun() {
            assert!(md("you").is_subject_pronoun());
        }
        #[test]
        fn you_is_object_pronoun() {
            assert!(md("you").is_object_pronoun());
        }
        #[test]
        fn yourself_is_pronoun() {
            assert!(md("yourself").is_pronoun());
        }
        #[test]
        fn yourself_is_personal_pronoun() {
            assert!(md("yourself").is_personal_pronoun());
        }
        #[test]
        fn yourself_is_singular_pronoun() {
            assert!(md("yourself").is_singular_pronoun());
        }
        #[test]
        fn yourself_is_reflexive_pronoun() {
            assert!(md("yourself").is_reflexive_pronoun());
        }
    }

    mod he_him_himself {
        use crate::dict_word_metadata::tests::md;

        #[test]
        fn he_is_pronoun() {
            assert!(md("he").is_pronoun());
        }
        #[test]
        fn he_is_personal_pronoun() {
            assert!(md("he").is_personal_pronoun());
        }
        #[test]
        fn he_is_singular_pronoun() {
            assert!(md("he").is_singular_pronoun());
        }
        #[test]
        fn he_is_subject_pronoun() {
            assert!(md("he").is_subject_pronoun());
        }

        #[test]
        fn him_is_pronoun() {
            assert!(md("him").is_pronoun());
        }
        #[test]
        fn him_is_personal_pronoun() {
            assert!(md("him").is_personal_pronoun());
        }
        #[test]
        fn him_is_singular_pronoun() {
            assert!(md("him").is_singular_pronoun());
        }
        #[test]
        fn him_is_object_pronoun() {
            assert!(md("him").is_object_pronoun());
        }

        #[test]
        fn himself_is_pronoun() {
            assert!(md("himself").is_pronoun());
        }
        #[test]
        fn himself_is_personal_pronoun() {
            assert!(md("himself").is_personal_pronoun());
        }
        #[test]
        fn himself_is_singular_pronoun() {
            assert!(md("himself").is_singular_pronoun());
        }
        #[test]
        fn himself_is_reflexive_pronoun() {
            assert!(md("himself").is_reflexive_pronoun());
        }
    }

    mod she_her_herself {
        use crate::dict_word_metadata::tests::md;

        #[test]
        fn she_is_pronoun() {
            assert!(md("she").is_pronoun());
        }
        #[test]
        fn she_is_personal_pronoun() {
            assert!(md("she").is_personal_pronoun());
        }
        #[test]
        fn she_is_singular_pronoun() {
            assert!(md("she").is_singular_pronoun());
        }
        #[test]
        fn she_is_subject_pronoun() {
            assert!(md("she").is_subject_pronoun());
        }

        #[test]
        fn her_is_pronoun() {
            assert!(md("her").is_pronoun());
        }
        #[test]
        fn her_is_personal_pronoun() {
            assert!(md("her").is_personal_pronoun());
        }
        #[test]
        fn her_is_singular_pronoun() {
            assert!(md("her").is_singular_pronoun());
        }
        #[test]
        fn her_is_object_pronoun() {
            assert!(md("her").is_object_pronoun());
        }

        #[test]
        fn herself_is_pronoun() {
            assert!(md("herself").is_pronoun());
        }
        #[test]
        fn herself_is_personal_pronoun() {
            assert!(md("herself").is_personal_pronoun());
        }
        #[test]
        fn herself_is_singular_pronoun() {
            assert!(md("herself").is_singular_pronoun());
        }
        #[test]
        fn herself_is_reflexive_pronoun() {
            assert!(md("herself").is_reflexive_pronoun());
        }
    }

    mod it_itself {
        use crate::dict_word_metadata::tests::md;

        #[test]
        fn it_is_pronoun() {
            assert!(md("it").is_pronoun());
        }
        #[test]
        fn it_is_personal_pronoun() {
            assert!(md("it").is_personal_pronoun());
        }
        #[test]
        fn it_is_singular_pronoun() {
            assert!(md("it").is_singular_pronoun());
        }
        #[test]
        fn it_is_subject_pronoun() {
            assert!(md("it").is_subject_pronoun());
        }
        #[test]
        fn it_is_object_pronoun() {
            assert!(md("it").is_object_pronoun());
        }

        #[test]
        fn itself_is_pronoun() {
            assert!(md("itself").is_pronoun());
        }
        #[test]
        fn itself_is_personal_pronoun() {
            assert!(md("itself").is_personal_pronoun());
        }
        #[test]
        fn itself_is_singular_pronoun() {
            assert!(md("itself").is_singular_pronoun());
        }
        #[test]
        fn itself_is_reflexive_pronoun() {
            assert!(md("itself").is_reflexive_pronoun());
        }
    }

    mod they_them_themselves {
        use crate::dict_word_metadata::tests::md;

        #[test]
        fn they_is_pronoun() {
            assert!(md("they").is_pronoun());
        }
        #[test]
        fn they_is_personal_pronoun() {
            assert!(md("they").is_personal_pronoun());
        }
        #[test]
        fn they_is_plural_pronoun() {
            assert!(md("they").is_plural_pronoun());
        }
        #[test]
        fn they_is_subject_pronoun() {
            assert!(md("they").is_subject_pronoun());
        }

        #[test]
        fn them_is_pronoun() {
            assert!(md("them").is_pronoun());
        }
        #[test]
        fn them_is_personal_pronoun() {
            assert!(md("them").is_personal_pronoun());
        }
        #[test]
        fn them_is_plural_pronoun() {
            assert!(md("them").is_plural_pronoun());
        }
        #[test]
        fn them_is_object_pronoun() {
            assert!(md("them").is_object_pronoun());
        }

        #[test]
        fn themselves_is_pronoun() {
            assert!(md("themselves").is_pronoun());
        }
        #[test]
        fn themselves_is_personal_pronoun() {
            assert!(md("themselves").is_personal_pronoun());
        }
        #[test]
        fn themselves_is_plural_pronoun() {
            assert!(md("themselves").is_plural_pronoun());
        }
        #[test]
        fn themselves_is_reflexive_pronoun() {
            assert!(md("themselves").is_reflexive_pronoun());
        }
    }

    // Possessive pronouns (not to be confused with possessive adjectives/determiners)
    #[test]
    fn mine_is_pronoun() {
        assert!(md("mine").is_pronoun());
    }
    #[test]
    fn ours_is_pronoun() {
        assert!(md("ours").is_pronoun());
    }
    #[test]
    fn yours_is_pronoun() {
        assert!(md("yours").is_pronoun());
    }
    #[test]
    fn his_is_pronoun() {
        assert!(md("his").is_pronoun());
    }
    #[test]
    fn hers_is_pronoun() {
        assert!(md("hers").is_pronoun());
    }
    #[test]
    fn its_is_pronoun() {
        assert!(md("its").is_pronoun());
    }
    #[test]
    fn theirs_is_pronoun() {
        assert!(md("theirs").is_pronoun());
    }

    // archaic pronouns
    #[test]
    fn archaic_pronouns() {
        assert!(md("thou").is_pronoun());
        assert!(md("thee").is_pronoun());
        assert!(md("thyself").is_pronoun());
        assert!(md("thine").is_pronoun());
    }

    // generic pronouns
    #[test]
    fn generic_pronouns() {
        assert!(md("one").is_pronoun());
        assert!(md("oneself").is_pronoun());
    }

    // relative and interrogative pronouns
    #[test]
    fn relative_and_interrogative_pronouns() {
        assert!(md("who").is_pronoun());
        assert!(md("whom").is_pronoun());
        assert!(md("whose").is_pronoun());
        assert!(md("which").is_pronoun());
        assert!(md("what").is_pronoun());
    }

    // nonstandard pronouns
    #[test]
    #[ignore = "not in dictionary"]
    fn nonstandard_pronouns() {
        assert!(md("themself").pronoun.is_some());
        assert!(md("y'all'").pronoun.is_some());
    }
}

mod nominal {
    use crate::dict_word_metadata::tests::md;

    #[test]
    fn my_is_possessive_nominal() {
        assert!(md("my").is_possessive_nominal());
    }

    #[test]
    fn mine_is_not_possessive_nominal() {
        assert!(!md("mine").is_possessive_nominal());
    }

    #[test]
    fn freds_is_possessive_nominal() {
        assert!(md("Fred's").is_possessive_nominal());
    }

    #[test]
    fn fred_is_not_possessive_nominal() {
        assert!(!md("Fred").is_possessive_nominal());
    }

    #[test]
    fn dogs_is_possessive_nominal() {
        assert!(md("dog's").is_possessive_nominal());
    }

    #[test]
    fn microsofts_is_possessive_nominal() {
        assert!(md("Microsoft's").is_possessive_nominal());
    }
}

mod adjective {
    use crate::{Degree, dict_word_metadata::tests::md};

    // Getting degrees

    #[test]
    #[ignore = "not marked yet because it might not be reliable"]
    fn big_is_positive() {
        assert_eq!(md("big").get_degree(), Some(Degree::Positive));
    }

    #[test]
    fn bigger_is_comparative() {
        assert_eq!(md("bigger").get_degree(), Some(Degree::Comparative));
    }

    #[test]
    fn biggest_is_superlative() {
        assert_eq!(md("biggest").get_degree(), Some(Degree::Superlative));
    }

    #[test]
    #[should_panic(expected = "Word 'bigly' not found in dictionary")]
    fn bigly_is_not_an_adjective_form_we_track() {
        assert_eq!(md("bigly").get_degree(), None);
    }

    // Calling is_ methods

    // TODO: positive degree not implemented

    #[test]
    fn bigger_is_comparative_adjective() {
        assert!(md("bigger").is_comparative_adjective());
    }

    #[test]
    fn biggest_is_superlative_adjective() {
        assert!(md("biggest").is_superlative_adjective());
    }
}

#[test]
fn the_is_determiner() {
    assert!(md("the").is_determiner());
}
#[test]
fn this_is_demonstrative_determiner() {
    assert!(md("this").is_demonstrative_determiner());
}
#[test]
fn your_is_possessive_determiner() {
    assert!(md("your").is_possessive_determiner());
}

#[test]
fn every_is_quantifier() {
    assert!(md("every").is_quantifier());
}

#[test]
fn the_isnt_quantifier() {
    assert!(!md("the").is_quantifier());
}

#[test]
fn equipment_is_mass_noun() {
    assert!(md("equipment").is_mass_noun());
}

#[test]
fn equipment_is_non_countable_noun() {
    assert!(md("equipment").is_non_countable_noun());
}

#[test]
fn equipment_isnt_countable_noun() {
    assert!(!md("equipment").is_countable_noun());
}

mod verb {
    use crate::dict_word_metadata::tests::md;

    #[test]
    fn lemma_walk() {
        let md = md("walk");
        assert!(md.is_verb_lemma())
    }

    #[test]
    fn lemma_fix() {
        let md = md("fix");
        assert!(md.is_verb_lemma())
    }

    #[test]
    fn progressive_walking() {
        let md = md("walking");
        assert!(md.is_verb_progressive_form())
    }

    #[test]
    fn past_walked() {
        let md = md("walked");
        assert!(md.is_verb_past_form())
    }

    #[test]
    fn regular_past_thought() {
        let md = md("thought");
        assert!(md.is_verb_regular_past_form())
    }

    #[test]
    fn simple_past_ate() {
        let md = md("ate");
        assert!(md.is_verb_simple_past_form())
    }

    #[test]
    fn past_participle_eaten() {
        let md = md("eaten");
        assert!(md.is_verb_past_participle_form())
    }

    #[test]
    fn ate_is_simple_past_only() {
        let md = md("ate");
        assert!(md.is_verb_simple_past_only());
        assert!(!md.is_verb_past_participle_only());
    }

    #[test]
    fn eaten_is_past_participle_only() {
        let md = md("eaten");
        assert!(md.is_verb_past_participle_only());
        assert!(!md.is_verb_simple_past_only());
    }

    #[test]
    fn thought_is_neither_past_form_only() {
        let md = md("thought");
        assert!(!md.is_verb_simple_past_only());
        assert!(!md.is_verb_past_participle_only());
    }

    #[test]
    fn shared_past_forms_are_neither_past_form_only() {
        let md = md("thought");
        assert!(!md.is_verb_simple_past_only());
        assert!(!md.is_verb_past_participle_only());
        assert!(md.is_verb_regular_past_form());
    }

    #[test]
    fn distinct_past_forms_are_not_regular_past() {
        assert!(!md("ate").is_verb_regular_past_form());
        assert!(!md("eaten").is_verb_regular_past_form());
        assert!(!md("walked").is_verb_regular_past_form());
    }

    #[test]
    fn third_pers_sing_walks() {
        let md = md("walks");
        assert!(md.is_verb_third_person_singular_present_form())
    }
}
