use crate::error::{Result, MissingVariableTemplateError };
use regex::Regex;
use std::collections::HashMap;

pub fn fill_template(template: &str, vars: &HashMap<String, String>) -> Result<String> {
    let re = Regex::new(r"\$\{([A-Za-z0-9_]+)\}").unwrap();

    let mut result = String::new();
    let mut last_match_end = 0;

    for caps in re.captures_iter(template) {
        let full_match = caps.get(0).unwrap(); // the entire ${...}
        let variable_name = &caps[1];

        // Push the text between the last match and this one
        result.push_str(&template[last_match_end..full_match.start()]);

        // Lookup the value
        if let Some(val) = vars.get(variable_name) {
            result.push_str(val);
        } else {
            return Err(MissingVariableTemplateError  {
                name: variable_name.to_string(),
            }
            .into());
        }

        last_match_end = full_match.end();
    }

    // Push remaining text after the last match
    result.push_str(&template[last_match_end..]);

    Ok(result)
}
#[test]
fn test_it() {
    use pretty_assertions::assert_eq;
    let mut vars = HashMap::new();
    vars.insert("my_var".to_string(), "Hello".to_string());
    vars.insert("other".to_string(), "World".to_string());

    let template = "Greeting: ${my_var}, Target: ${other}!";
    let res = fill_template(template, &vars).unwrap();

    assert_eq!(res, "Greeting: Hello, Target: World!");

    let bad_template = "Greeting: ${undefinde_variable}!";
    let res = fill_template(bad_template, &vars);
    assert!(res.is_err());
    if let Err(error) = res {
        assert_eq!(
            error.to_string(), 
            r#"MissingVariableTemplateError(MissingVariableTemplateError { name: "undefinde_variable" })"#.to_string()
        );
    }
}
