use regex::Regex;
use std::sync::LazyLock;

pub static KEY_REGEX: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^[A-Z0-9]+$").expect("Invalid regex pattern"));
