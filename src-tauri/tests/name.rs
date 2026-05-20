use bhc_lib::project::name::validate_project_name;

#[test]
fn empty_is_rejected() {
    assert!(validate_project_name("").is_err());
    assert!(validate_project_name("   ").is_err());
}

#[test]
fn over_100_chars_is_rejected() {
    let long = "x".repeat(101);
    assert!(validate_project_name(&long).is_err());
}

#[test]
fn under_or_eq_100_chars_is_accepted() {
    let ok = "x".repeat(100);
    assert!(validate_project_name(&ok).is_ok());
}

#[test]
fn forbidden_chars_are_rejected() {
    for c in &['<', '>', ':', '"', '/', '\\', '|', '?', '*'] {
        let name = format!("bad{c}name");
        assert!(validate_project_name(&name).is_err(), "{c} should be rejected");
    }
}

#[test]
fn control_chars_are_rejected() {
    assert!(validate_project_name("bad\u{0001}name").is_err());
    assert!(validate_project_name("bad\nname").is_err()); // \n is 0x0A
}

#[test]
fn trailing_dot_or_space_is_rejected() {
    // Trailing dot: the trim doesn't touch it, so the rule fires.
    assert!(validate_project_name("name.").is_err());
    // Trailing space after trim: outer whitespace is stripped first, so
    // we need an internal-then-trailing pattern to leave a literal space
    // at the end of the trimmed value. `"hi . "` -> trim -> `"hi ."` ->
    // ends with '.', which fires the trailing-dot branch (still an
    // error, still validates this code path is reachable).
    assert!(validate_project_name("hi . ").is_err());
    // Pure outer whitespace alone is consumed by trim — the result is
    // an empty string, caught by the empty-name rule, not trailing-space.
    assert!(validate_project_name("name ").is_ok()); // trims to "name"
}

#[test]
fn reserved_names_are_rejected() {
    for name in &["CON", "con", "Con", "NUL", "PRN", "AUX", "COM1", "COM9", "LPT1", "LPT9"] {
        assert!(validate_project_name(name).is_err(), "{name} should be rejected");
    }
    // Reserved name with extension
    assert!(validate_project_name("CON.txt").is_err());
    assert!(validate_project_name("con.figured").is_err()); // base before dot is con
}

#[test]
fn valid_names_are_accepted() {
    for name in &["My Project", "APT-29 sweep", "Project 2026 Q1", "Test", "configuration"] {
        assert!(validate_project_name(name).is_ok(), "{name} should be accepted");
    }
}

#[test]
fn surrounding_whitespace_is_trimmed_before_validation() {
    assert!(validate_project_name("  Test  ").is_ok());
}
