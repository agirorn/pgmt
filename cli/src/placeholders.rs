use std::collections::HashMap;
use std::env;

pub fn collect_placeholders_from_environment_variable() -> HashMap<String, String> {
    let prefix = "PGMT_PLACEHOLDERS_";

    env::vars()
        .filter_map(|(key, value)| {
            key.strip_prefix(prefix)
                .map(|stripped| (stripped.to_string().to_lowercase(), value))
        })
        .collect()
}

#[test]
fn test_it() {
    use pretty_assertions::assert_eq;
    unsafe {
        env::set_var("SHULD_NOT_GET_THIS_ENV", "XXX");
        env::set_var("PGMT_PLACEHOLDERS_MY_VAR", "123");
        env::set_var("PGMT_PLACEHOLDERS_OTHER", "abc");
    }

    let placeholders = collect_placeholders_from_environment_variable();
    let expected: HashMap<String, String> = HashMap::from([
        ("my_var".to_string(), "123".to_string()),
        ("other".to_string(), "abc".to_string()),
    ]);
    assert_eq!(placeholders, expected);
}
