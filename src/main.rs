use std::iter::Peekable;
use std::str::CharIndices;

#[derive(Debug, Eq, PartialEq)]
enum TokenizerFailure {
    UnexpectedCharacterInName { index: usize },
    UnclosedString,
}

#[derive(Debug, Eq, PartialEq)]
enum TokenizerResult<'a> {
    Ok(Token<'a>),
    Err(TokenizerFailure),
    End,
}

#[derive(Eq, PartialEq, Debug)]
enum TokenizerState {
    None,
    ReadingName { start_index: usize },
    ReadingString { start_index: usize },
}

struct Tokenizer<'a> {
    iter: Peekable<CharIndices<'a>>,
    data: &'a str,
    is_failed: bool,
}

impl<'a> Tokenizer<'a> {
    pub fn new(data: &'a str) -> Self {
        Tokenizer {
            iter: data.char_indices().peekable(),
            data,
            is_failed: false,
        }
    }

    fn next(&mut self) -> TokenizerResult {
        if self.is_failed {
            return TokenizerResult::End;
        }

        let mut state = TokenizerState::None;

        while let Some((index, character)) = self.iter.next() {
            match state {
                TokenizerState::None if character.is_ascii_alphabetic() => {
                    state = TokenizerState::ReadingName { start_index: index }
                }
                TokenizerState::ReadingName { .. } if character.is_ascii_alphabetic() => {}
                TokenizerState::ReadingName { start_index } if character.is_ascii_whitespace() => {
                    return TokenizerResult::Ok(Token::Name(&self.data[start_index..index]))
                }
                TokenizerState::ReadingName { .. } => {
                    self.is_failed = true;

                    return TokenizerResult::Err(TokenizerFailure::UnexpectedCharacterInName {
                        index,
                    });
                }
                TokenizerState::None if character == '"' => {
                    state = TokenizerState::ReadingString { start_index: index }
                }
                TokenizerState::ReadingString { .. } if character == '\\' => {
                    match self.iter.peek() {
                        Some((_, '\"')) => {
                            self.iter.next();
                        }
                        _ => {}
                    }
                }
                TokenizerState::ReadingString { start_index } if character == '"' => {
                    return TokenizerResult::Ok(Token::String(
                        self.data[start_index + 1..index]
                            .to_owned()
                            .replace("\\\"", "\""),
                    ))
                }
                TokenizerState::ReadingString { .. } => {}
                TokenizerState::None => unimplemented!(),
            }
        }

        match state {
            TokenizerState::ReadingName { start_index } => {
                TokenizerResult::Ok(Token::Name(&self.data[start_index..]))
            }
            TokenizerState::None => TokenizerResult::End,
            TokenizerState::ReadingString { .. } => {
                TokenizerResult::Err(TokenizerFailure::UnclosedString)
            }
        }
    }
}

#[derive(Eq, PartialEq, Debug)]
enum Token<'a> {
    Name(&'a str),
    // must be a String, not a &str to source, since strings with escape sequence will be different
    // from the representation in source, e.g. "test\"string" will have `\"` replaced with `"`
    String(String),
}

fn main() {}

#[cfg(test)]
mod tests {
    use crate::{Token, Tokenizer, TokenizerFailure, TokenizerResult};

    macro_rules! tokenizer_test {
        ( $test_name: ident, $test_string: expr, $($expected_token:expr),+ ) => {
            #[test]
            pub fn $test_name() {
                let mut tokenizer = Tokenizer::new($test_string);

                $(
                    assert_eq!(TokenizerResult::Ok($expected_token), tokenizer.next());
                )*

                assert_eq!(TokenizerResult::End, tokenizer.next());
            }
        };
    }

    tokenizer_test!(can_tokenize_a_name, "a", Token::Name("a"));
    tokenizer_test!(
        can_tokenize_a_multicharacter_name,
        "test",
        Token::Name("test")
    );
    tokenizer_test!(
        can_tokenize_multiple_names,
        "something else",
        Token::Name("something"),
        Token::Name("else")
    );
    tokenizer_test!(
        can_read_names_separated_by_unix_newlines,
        "first\nsecond",
        Token::Name("first"),
        Token::Name("second")
    );

    #[test]
    pub fn fails_on_invalid_character_in_name() {
        let mut tokenizer = Tokenizer::new("name\"");

        assert_eq!(
            TokenizerResult::Err(TokenizerFailure::UnexpectedCharacterInName { index: 4 }),
            tokenizer.next()
        );
        assert_eq!(TokenizerResult::End, tokenizer.next());
    }

    #[test]
    pub fn returns_end_after_a_failure() {
        let mut tokenizer = Tokenizer::new("name\" othername");

        assert_eq!(
            TokenizerResult::Err(TokenizerFailure::UnexpectedCharacterInName { index: 4 }),
            tokenizer.next()
        );
        assert_eq!(TokenizerResult::End, tokenizer.next());
    }

    tokenizer_test!(
        can_read_a_simple_string,
        "\"some string\"",
        Token::String("some string".into())
    );

    #[test]
    pub fn fails_on_unclosed_string() {
        let mut tokenizer = Tokenizer::new("\"bla");

        assert_eq!(
            TokenizerResult::Err(TokenizerFailure::UnclosedString),
            tokenizer.next()
        );
        assert_eq!(TokenizerResult::End, tokenizer.next());
    }

    tokenizer_test!(
        can_read_a_string_with_escaped_quotation_mark,
        "\"test\\\"some\\\"words\"",
        Token::String("test\"some\"words".into())
    );
}
