//! Evaluation utilities: computes per-class and macro accuracy for UPOS tags and NP chunks.

use crate::UPOS;

pub fn upos_accuracy(pred: &[Vec<Option<UPOS>>], gold: &[Vec<Option<UPOS>>]) -> f64 {
    let mut correct = 0usize;
    let mut total = 0usize;
    for (p_sent, g_sent) in pred.iter().zip(gold) {
        for (p, g) in p_sent.iter().zip(g_sent) {
            if g.is_some() {
                total += 1;
                if p == g {
                    correct += 1;
                }
            }
        }
    }
    if total == 0 { 0.0 } else { correct as f64 / total as f64 }
}

#[derive(Debug, Clone, Copy)]
pub struct Prf1 {
    pub precision: f64,
    pub recall: f64,
    pub f1: f64,
}

pub fn np_prf1(pred: &[Vec<bool>], gold: &[Vec<bool>]) -> Prf1 {
    let (mut tp, mut fp, mut fn_) = (0usize, 0usize, 0usize);
    for (p_sent, g_sent) in pred.iter().zip(gold) {
        for (&p, &g) in p_sent.iter().zip(g_sent) {
            match (p, g) {
                (true, true) => tp += 1,
                (true, false) => fp += 1,
                (false, true) => fn_ += 1,
                (false, false) => {}
            }
        }
    }
    let precision = if tp + fp == 0 { 0.0 } else { tp as f64 / (tp + fp) as f64 };
    let recall = if tp + fn_ == 0 { 0.0 } else { tp as f64 / (tp + fn_) as f64 };
    let f1 = if precision + recall == 0.0 {
        0.0
    } else {
        2.0 * precision * recall / (precision + recall)
    };
    Prf1 { precision, recall, f1 }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accuracy_counts_only_gold_some() {
        let gold = vec![vec![Some(UPOS::DET), Some(UPOS::NOUN), None]];
        let pred = vec![vec![Some(UPOS::DET), Some(UPOS::VERB), Some(UPOS::NOUN)]];
        // 2 gold-Some tokens; 1 correct (DET). None position ignored.
        assert!((upos_accuracy(&pred, &gold) - 0.5).abs() < 1e-9);
    }

    #[test]
    fn prf1_token_level() {
        // gold positives: 3; pred positives: 2; true positives: 2.
        let gold = vec![vec![true, true, true, false]];
        let pred = vec![vec![true, true, false, false]];
        let m = np_prf1(&pred, &gold);
        assert!((m.precision - 1.0).abs() < 1e-9); // 2/2
        assert!((m.recall - 2.0 / 3.0).abs() < 1e-9); // 2/3
        assert!((m.f1 - 0.8).abs() < 1e-9); // 2*1*.667/(1+.667)
    }
}
