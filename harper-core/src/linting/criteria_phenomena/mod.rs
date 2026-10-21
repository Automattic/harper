mod plural_criterion_phenomenon;
mod singular_criteria_phenomena;

use super::merge_linters::merge_linters;
use plural_criterion_phenomenon::PluralCriterionPhenomenon;
use singular_criteria_phenomena::SingularCriteriaPhenomena;

merge_linters!(CriteriaPhenomena => SingularCriteriaPhenomena, PluralCriterionPhenomenon => "The words “criteria” and “phenomena” are the plurals of “criterion” and “phenomenon”, respectively. They are often incorrectly used with the wrong number.");
