use rstest::rstest;
use substreams::parser::{evaluate_expression};
 
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

fn test_parse(#[case] keys: &[&str], #[case] input: &str, #[case] expected: bool) {
    let keys: Vec<String> = keys.iter().map(|s| s.to_string()).collect();
    assert_eq!(evaluate_expression(keys, input), expected);
}

