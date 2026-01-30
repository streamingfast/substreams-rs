# SQE (Substreams Query Expression) Grammar Specification

## Overview

SQE is a simple boolean expression language for matching a set of keys against an expression. It supports AND, OR operations with grouping via parentheses.

## Lexical Elements

### Whitespace
- Space (` `), Tab (`\t`), Newline (`\n`)
- Whitespace is significant when separating unquoted keys (acts as implicit AND)
- Whitespace around operators is optional

### Operators
| Token | Meaning |
|-------|---------|
| `\|\|` | Logical OR (lowest precedence) |
| `&&` | Logical AND |
| `(` | Open grouping |
| `)` | Close grouping |

### Keys

#### Unquoted Keys
- Any sequence of characters EXCEPT: whitespace, `'`, `"`, `(`, `)`, `\|`, `&`
- MUST NOT start with `-`
- Examples: `test`, `test.7`, `test:8`, `type:wasm-MarketUpdated`, `test*19z_\|`

#### Single-Quoted Keys
- Delimited by single quotes `'...'`
- Content MUST NOT start with `-`
- Can contain any character except single quote
- Quotes are stripped when matching
- Examples: `'test 6'`, `'key with spaces'`

#### Double-Quoted Keys
- Delimited by double quotes `"..."`
- Content MUST NOT start with `-`
- Can contain any character except double quote
- Quotes are stripped when matching
- Examples: `"test 6"`, `"key with spaces"`

## Grammar (EBNF)

```ebnf
expression   = or , [ whitespace ] , EOF ;
or           = and , { whitespace? , "||" , whitespace? , and } ;
and          = value , { ( whitespace , value ) | ( whitespace? , "&&" , whitespace? , value ) } ;
value        = whitespace? , ( quoted_key | key | group ) , whitespace? ;
group        = "(" , whitespace? , or , whitespace? , ")" ;
quoted_key   = single_quoted | double_quoted ;
single_quoted = "'" , quoted_content_single , "'" ;
double_quoted = '"' , quoted_content_double , '"' ;
quoted_content_single = ? non-dash-start ? , { ? any char except ' ? } ;
quoted_content_double = ? non-dash-start ? , { ? any char except " ? } ;
key          = ? non-dash-start ? , { key_char } ;
key_char     = ? any char except whitespace, ', ", (, ), ||, && ? ;
whitespace   = ( " " | "\t" | "\n" ) , { " " | "\t" | "\n" } ;
```

## Operator Precedence (lowest to highest)

1. `||` (OR) - lowest precedence
2. `&&` (AND) / implicit AND (whitespace)
3. Grouping `()` - highest precedence

## Semantics

### Matching Rules

Given an expression `E` and a set of keys `K`:

1. **Key**: `key` matches if `key` is in `K`
2. **QuotedKey**: `'key'` or `"key"` matches if `key` (without quotes) is in `K`
3. **AND**: `A && B` or `A B` matches if both `A` matches AND `B` matches
4. **OR**: `A || B` matches if either `A` matches OR `B` matches
5. **Group**: `(E)` matches if `E` matches

### Short-circuit Evaluation

- OR: Stop on first match (left to right)
- AND: Stop on first non-match (left to right)

## Examples

| Expression | Keys | Result |
|------------|------|--------|
| `test` | `["test"]` | true |
| `test` | `["other"]` | false |
| `a \|\| b` | `["a"]` | true |
| `a \|\| b` | `["c"]` | false |
| `a && b` | `["a", "b"]` | true |
| `a && b` | `["a"]` | false |
| `a b` | `["a", "b"]` | true (implicit AND) |
| `(a \|\| b) && c` | `["a", "c"]` | true |
| `(a \|\| b) && c` | `["a"]` | false |
| `'key with space'` | `["key with space"]` | true |
| `type:wasm-Event` | `["type:wasm-Event"]` | true |

## Error Cases

1. Keys starting with `-` are invalid: `-test`, `'-test'`, `"-test"`
2. Unclosed quotes: `'test`, `"test`
3. Unclosed parentheses: `(test`, `test)`
4. Empty expression: ``
5. Empty quoted string followed by dash: This is covered by the "cannot start with dash" rule

## Differences from Original Pest Grammar

The original Pest grammar and the SQE specification are functionally identical. The key behaviors preserved:

1. Implicit AND with whitespace
2. Explicit AND with `&&`
3. OR with `||`
4. Parenthesis grouping
5. Single and double quoted strings
6. Dash restriction at start of keys
7. Special character support in keys (`:`, `.`, `*`, `_`, `|`)
