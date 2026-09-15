//! Fail closed: only explicit successes satisfy the required validation tier.

fn accepted(draft: bool, quick: &str, full: &str) -> bool {
    if draft {
        quick == "success"
    } else {
        full == "success"
    }
}

fn main() -> Result<(), String> {
    let args: Vec<_> = std::env::args().skip(1).collect();
    let [draft, quick, full] = args.as_slice() else {
        return Err("expected draft flag, quick result and full matrix result".into());
    };
    let draft = match draft.as_str() {
        "true" => true,
        "false" => false,
        _ => return Err("invalid draft flag".into()),
    };
    if accepted(draft, quick, full) {
        Ok(())
    } else {
        Err("required CI jobs must explicitly succeed".into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn only_explicit_success_satisfies_each_tier() {
        for quick in ["success", "failure", "cancelled", "skipped", "", "unknown"] {
            for full in ["success", "failure", "cancelled", "skipped", "", "unknown"] {
                assert_eq!(accepted(true, quick, full), quick == "success");
                assert_eq!(accepted(false, quick, full), full == "success");
            }
        }
    }
}
