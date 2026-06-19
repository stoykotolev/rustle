use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LetterState {
    Correct,
    Present,
    Absent,
}

pub fn evaluate(guess: &[char; 5], solution: &[char; 5]) -> [LetterState; 5] {
    let mut result = [LetterState::Absent; 5];
    let mut remaining: HashMap<char, usize> = HashMap::new();

    // Pass 1: mark exact matches and count remaining solution letters
    for i in 0..5 {
        if guess[i] == solution[i] {
            result[i] = LetterState::Correct;
        } else {
            *remaining.entry(solution[i]).or_insert(0) += 1;
        }
    }

    // Pass 2: mark present letters (wrong position)
    for i in 0..5 {
        if result[i] == LetterState::Correct {
            continue;
        }
        if let Some(c) = remaining.get_mut(&guess[i]) {
            if *c > 0 {
                result[i] = LetterState::Present;
                *c -= 1;
            }
        }
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use LetterState::{Absent as A, Correct as C, Present as P};

    fn chars(s: &str) -> [char; 5] {
        let v: Vec<char> = s.chars().collect();
        v.try_into().expect("test word must be 5 chars")
    }

    #[test]
    fn test_all_correct() {
        assert_eq!(evaluate(&chars("spell"), &chars("spell")), [C, C, C, C, C]);
    }

    #[test]
    fn test_all_absent() {
        assert_eq!(evaluate(&chars("dawui"), &chars("spell")), [A, A, A, A, A]);
    }

    #[test]
    fn test_one_present_one_correct() {
        // solution=spell, guess=dewli:
        // Pass1: d vs s(no), e vs p(no), w vs e(no), l vs l(C), i vs l(no)
        // remaining={s:1,p:1,e:1,l:1}
        // Pass2: d→no, e→Present, w→no, i→no
        assert_eq!(evaluate(&chars("dewli"), &chars("spell")), [A, P, A, C, A]);
    }

    #[test]
    fn test_two_present() {
        // solution=spell, guess=dllai:
        // Pass1: d vs s(no), l vs p(no), l vs e(no), a vs l(no), i vs l(no)
        // remaining={s:1,p:1,e:1,l:2}
        // Pass2: d→no, l→Present(l:1), l→Present(l:0), a→no, i→no
        assert_eq!(evaluate(&chars("dllai"), &chars("spell")), [A, P, P, A, A]);
    }

    #[test]
    fn test_correct_no_duplicate() {
        // solution=spelt, guess=diill:
        // Pass1: d vs s(no), i vs p(no), i vs e(no), l vs l(C), l vs t(no)
        // remaining={s:1,p:1,e:1,t:1}
        // Pass2: d→no, i→no, i→no, l(skipped-Correct), l→no
        assert_eq!(evaluate(&chars("diill"), &chars("spelt")), [A, A, A, C, A]);
    }

    #[test]
    fn test_more_duplicate_letters() {
        // solution=spell, guess=seell:
        // Pass1: s vs s(C), e vs p(no), e vs e(C), l vs l(C), l vs l(C)
        // remaining={p:1}
        // Pass2: i=1: 'e' not in remaining → Absent
        assert_eq!(evaluate(&chars("seell"), &chars("spell")), [C, A, C, C, C]);
    }

    #[test]
    fn test_two_pass_regression_single_r() {
        // guess=rrxxx, solution=crane: r at idx1 is an exact match, so the
        // idx0 r is Absent (only one r in the solution).
        assert_eq!(evaluate(&chars("rrxxx"), &chars("crane")), [A, C, A, A, A]);
    }

    #[test]
    fn test_two_pass_regression_llama() {
        // guess=llxxx, solution=llama: l correct at 0, l correct at 1
        assert_eq!(evaluate(&chars("llxxx"), &chars("llama")), [C, C, A, A, A]);
    }
}
