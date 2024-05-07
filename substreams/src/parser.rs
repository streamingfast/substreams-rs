use pest::{iterators::Pair, Parser};
use pest_derive::Parser;

#[derive(Parser)]
#[grammar = "parser_rule.pest"]
pub struct EParser;

pub fn evaluate_expression(keys: Vec<String>, input: &str) -> bool {
    let successful_parse = EParser::parse(Rule::expression, input).unwrap();
    return evaluate_rule(successful_parse.clone().into_iter().next().unwrap(), keys);
}

fn evaluate_rule(pair: Pair<Rule>,  keys: Vec<String>) -> bool {
    match pair.as_rule() {
        Rule::expression => {
            let inner_pair = pair.into_inner().next().unwrap();
            return evaluate_rule(inner_pair, keys);
        }
        Rule::or => {
            let mut result = false;
            for inner_pair in pair.into_inner() {
                result = result || evaluate_rule(inner_pair, keys.clone());
            }
            return result;
        },
        Rule::and => {
            let mut result = true;
            for inner_pair in pair.into_inner() {
                result = result && evaluate_rule(inner_pair, keys.clone());
            }
            return result;
        },
        Rule::value => {
            let inner_pair = pair.into_inner().next().unwrap();
            return evaluate_rule(inner_pair, keys);
        }
        Rule::keyterm => {
            return keys.contains(&pair.as_str().to_string());
        }
        Rule::singleQuoteKeyTerm => {
            return keys.contains(&pair.as_str().to_string().replace("'", ""));
        }
        Rule::doubleQuoteKeyTerm => {
            return keys.contains(&pair.as_str().to_string().replace("\"", ""));
        }
        _ => {panic!("Unexpected rule encountered")}
    } 
}





