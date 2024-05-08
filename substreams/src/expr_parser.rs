use anyhow::{Context, Error};
use pest::{iterators::Pair, Parser};
use pest_derive::Parser;

#[derive(Parser)]
#[grammar = "expr_parser_rule.pest"]
struct EParser;

fn parsing(input: &str) -> Result<Pair<Rule>, Error> {
    let pairs = EParser::parse(Rule::expression, input)
        .context("parsing input based on the expression rule")?;

    match pairs.into_iter().next() {
        Some(pair) => Ok(pair),
        None => Err(anyhow::Error::msg("no pairs found in input"))
    }
}

pub fn matches_keys_in_parsed_expr<K: AsRef<str>, I: AsRef<str>>(keys: &[K], input: I) -> Result<bool, Error> { 
       let successful_parse = parsing(input.as_ref()).context("parsing expression")?;
    Ok(apply_rule(successful_parse, keys))
}

fn apply_rule<K: AsRef<str>>(pair: Pair<Rule>,  keys: &[K]) -> bool {
    match pair.as_rule() {
        Rule::expression => {
            let inner_pair = pair.into_inner().next().unwrap();
            return apply_rule(inner_pair, keys);
        }
        Rule::or => {
            let mut result = false;
            for inner_pair in pair.into_inner() {
                result = result || apply_rule(inner_pair, keys);
            }
            return result;
        },
        Rule::and => {
            let mut result = true;
            for inner_pair in pair.into_inner() {
                result = result && apply_rule(inner_pair, keys);
            }
            return result;
        },
        Rule::value => {
            let inner_pair = pair.into_inner().next().unwrap();
            return apply_rule(inner_pair, keys);
        }
        Rule::keyterm => {
            return keys.iter().any(|key| key.as_ref() == pair.as_str());
        }
        Rule::singleQuoteKeyTerm => {
            return keys.iter().any(|key| key.as_ref() == pair.as_str().trim_matches('\''));
        }
        Rule::doubleQuoteKeyTerm => {
            return keys.iter().any(|key| key.as_ref() == pair.as_str().trim_matches('"'));
        }
        _ => {panic!("Unexpected rule encountered")}
    } 
}

#[cfg(test)]
fn expression_to_string(parsing: Pair<Rule>) -> String {
    let rule = parsing.as_rule();
    match rule {
        Rule::expression => {
            let inner_pair = parsing.into_inner().next().unwrap();
            return expression_to_string(inner_pair);
        }
        Rule::or => {
            let mut result = String::new();
            result.push_str("[");
            for inner_pair in parsing.into_inner() {
                result.push_str(&expression_to_string(inner_pair)); 
                result.push_str("||");
            }   
        
            if result.ends_with("||") {
            result.truncate(result.len() - 2);
            }

            result.push_str("]");
            return result;
        },
        Rule::and => {
            let mut result = String::new();
            result.push_str("<");
            for inner_pair in parsing.into_inner() {
                result.push_str(&expression_to_string(inner_pair)); 
                result.push_str("&&");
            }   
        
            if result.ends_with("&&") {
            result.truncate(result.len() - 2);
            }

            result.push_str(">");
            return result;
        },
        Rule::value => {
            let inner_pair = parsing.into_inner().next().unwrap();
            return expression_to_string(inner_pair);
        }
        Rule::keyterm => {
            return parsing.as_str().to_string();
        }
        Rule::singleQuoteKeyTerm => {
            return parsing.as_str().trim_matches('\'').to_string();
        }
        Rule::doubleQuoteKeyTerm => {
            return parsing.as_str().trim_matches('\"').to_string();
        }
        _ => {panic!("Unexpected rule encountered")}
    }
}


#[cfg(test)]
mod tests {
    use rstest::rstest;
    use super::*;
    static TEST_KEYS: &[&str] = &["test", "test1", "test2", "test3", "test4", "test5", "test 6"];

    #[rstest]
    #[case(TEST_KEYS, "test", true)]
    #[case(TEST_KEYS, "'test'", true)]
    #[case(TEST_KEYS, "'test 6' || test7", true)]
    #[case(TEST_KEYS, "'test_6' && test3", false)]
    #[case(TEST_KEYS, "\"test 6\" || test7", true)]
    #[case(TEST_KEYS, "\"test 6\" && test3", true)]

    #[case(TEST_KEYS, "test1 || test", true)]
    #[case(TEST_KEYS, "test1 || test6", true)]
    #[case(TEST_KEYS, "test6 || test7", false)]

    #[case(TEST_KEYS, "test1 || test || test2", true)]
    #[case(TEST_KEYS, "test1 || test6 || test7", true)]
    #[case(TEST_KEYS, "test6 || test7 || test8", false)]

    #[case(TEST_KEYS, "test1 && test", true)]
    #[case(TEST_KEYS, "test1 && test6", false)]
    #[case(TEST_KEYS, "test6 && test7", false)]

    #[case(TEST_KEYS, "test1 && test && test2", true)]
    #[case(TEST_KEYS, "test1 && test2 && test7", false)]
    #[case(TEST_KEYS, "test6 && test7 && test8", false)]

    #[case(TEST_KEYS, "test1 test", true)]
    #[case(TEST_KEYS, "test1 test6", false)]
    #[case(TEST_KEYS, "test6 test7", false)]

    #[case(TEST_KEYS, "(test1)", true)]
    #[case(TEST_KEYS, "(test1 test6)", false)]

    #[case(TEST_KEYS, "test1     test2 ", true)]
    #[case(TEST_KEYS, "test1    && test2 ", true)]
    #[case(TEST_KEYS, "test1    &&     test6", false)]
    #[case(TEST_KEYS, "(test1   ||  test3)       &&  test6 ", false)]
    #[case(TEST_KEYS, "(test1  ||     test6 || test7  )     && (test4 || test5) && test3 ", true)]

    #[case(TEST_KEYS, "(test1 || test6 || test7) && (test4 || test5) && test3 ", true)]
    #[case(TEST_KEYS, "(test1 && test6 && test7) || (test4 && test5) || test3 ", true)]

    fn test_matches_keys_in_parsed_expr(#[case] keys: &[&str], #[case] input: &str, #[case] expected: bool) {
        let pair = parsing(input).unwrap();
        let expr_as_string = expression_to_string(pair);

        let result = matches_keys_in_parsed_expr(keys, input).expect("matching keys in parsed expression");
        
        assert_eq!(result, expected, "This expression ast is {expr_as_string}");
    }

    #[rstest]
    #[case(TEST_KEYS, "test1 *213 ", "parsing expression")]
    #[case(TEST_KEYS, "|213 test", "parsing expression")]
    #[case(TEST_KEYS, "", "parsing expression")]

    fn test_matches_keys_in_parsed_expr_error(#[case] keys: &[&str], #[case] input: &str, #[case] expected_error: &str) {
        let result = matches_keys_in_parsed_expr(keys, input).expect_err("parsing is not failing");
        assert_eq!(result.to_string(), expected_error);
    }
    
}