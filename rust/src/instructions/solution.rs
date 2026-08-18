use crate::core::config::solution::SolutionIntensity;

/// Returns the active Solution Intelligence decision ladder.
///
/// The balanced wording intentionally extends the plan's Section 3.8 shorthand:
/// `(active)` identifies injected guidance and `Before implementing` scopes when
/// the unchanged "stop at the first level" rule applies.
#[must_use]
pub fn solution_ladder_text(intensity: &SolutionIntensity) -> &'static str {
    match intensity {
        SolutionIntensity::Off => "",
        SolutionIntensity::Minimal => {
            "SOLUTION HINTS (active):\nWhen implementing, consider whether stdlib or a native platform feature\nalready covers this. Name the alternative if it does."
        }
        SolutionIntensity::Balanced => {
            "SOLUTION EFFICIENCY (active):\nBefore implementing, stop at the first level that applies:\n1. Is this needed at all? Skip speculative work.\n2. Already exists in this codebase? Reuse it.\n3. Standard library covers it? Use stdlib.\n4. Native platform feature? Use it (CSS > JS, DB constraint > app code).\n5. Already-installed dependency? Use it, don't add new ones for few lines.\n6. Can be one line? Write one line.\n7. Only then: minimum working code.\n\nUnderstand the full flow BEFORE picking a level. Shortest correct diff wins.\nNever simplify away: validation, security, error handling, accessibility.\nMark deliberate simplifications: // lean-ctx: <ceiling>, upgrade when <condition>"
        }
        SolutionIntensity::Aggressive => {
            "SOLUTION EFFICIENCY — AGGRESSIVE (active):\nChallenge every requirement. Prefer deletion over addition.\n1. Does this need to exist? Most things don't. Say so.\n2. Already in codebase? Reuse or extend existing code.\n3. Stdlib does it? Always prefer stdlib.\n4. Native platform? Always prefer native.\n5. Installed dep? Use it, never add new dependencies.\n6. One line? Ship the one-liner, challenge the rest.\n7. Only if proven necessary: absolute minimum.\n\nShip the simplest version. Question remaining scope in the same response."
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn off_has_no_instruction_text() {
        assert_eq!(solution_ladder_text(&SolutionIntensity::Off), "");
    }

    #[test]
    fn minimal_has_the_solution_hint() {
        assert_eq!(
            solution_ladder_text(&SolutionIntensity::Minimal),
            "SOLUTION HINTS (active):\nWhen implementing, consider whether stdlib or a native platform feature\nalready covers this. Name the alternative if it does."
        );
    }

    #[test]
    fn balanced_has_the_decision_ladder() {
        assert_eq!(
            solution_ladder_text(&SolutionIntensity::Balanced),
            "SOLUTION EFFICIENCY (active):\nBefore implementing, stop at the first level that applies:\n1. Is this needed at all? Skip speculative work.\n2. Already exists in this codebase? Reuse it.\n3. Standard library covers it? Use stdlib.\n4. Native platform feature? Use it (CSS > JS, DB constraint > app code).\n5. Already-installed dependency? Use it, don't add new ones for few lines.\n6. Can be one line? Write one line.\n7. Only then: minimum working code.\n\nUnderstand the full flow BEFORE picking a level. Shortest correct diff wins.\nNever simplify away: validation, security, error handling, accessibility.\nMark deliberate simplifications: // lean-ctx: <ceiling>, upgrade when <condition>"
        );
    }

    #[test]
    fn aggressive_challenges_remaining_scope() {
        assert_eq!(
            solution_ladder_text(&SolutionIntensity::Aggressive),
            "SOLUTION EFFICIENCY — AGGRESSIVE (active):\nChallenge every requirement. Prefer deletion over addition.\n1. Does this need to exist? Most things don't. Say so.\n2. Already in codebase? Reuse or extend existing code.\n3. Stdlib does it? Always prefer stdlib.\n4. Native platform? Always prefer native.\n5. Installed dep? Use it, never add new dependencies.\n6. One line? Ship the one-liner, challenge the rest.\n7. Only if proven necessary: absolute minimum.\n\nShip the simplest version. Question remaining scope in the same response."
        );
    }
}
