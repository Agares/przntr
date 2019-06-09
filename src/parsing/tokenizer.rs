use crate::parsing::token_stream::{
    SourceLocation, Token, TokenStream, TokenizerFailure, TokenizerFailureKind, TokenizerResult,
};
use std::iter::Peekable;
use std::str::CharIndices;

#[derive(Eq, PartialEq, Debug)]
enum TokenizerState {
    None,
    ReadingName { start_index: usize },
    ReadingString { start_index: usize },
}

pub struct Tokenizer<'a> {
    iter: Peekable<CharIndices<'a>>,
    data: &'a str,
    is_failed: bool,
    line: u32,
    column: u32,
}

impl<'a> Tokenizer<'a> {
    pub fn new(data: &'a str) -> Self {
        Tokenizer {
            iter: data.char_indices().peekable(),
            data,
            is_failed: false,
            line: 0,
            column: 0,
        }
    }

    fn handle_name_or_keyword(&self, name: &str) -> TokenizerResult {
        TokenizerResult::Ok(match name {
            "slide" => Token::KeywordSlide,
            "title" => Token::KeywordTitle,
            "metadata" => Token::KeywordMetadata,
            _ => Token::Name(name.into()),
        })
    }

    fn is_name_character(&self, character: char) -> bool {
        character.is_ascii_alphanumeric() || character == '_' || character == '-'
    }

    fn read_next(&mut self) -> Option<(usize, char)> {
        self.column += 1;
        let result = self.iter.next();

        if let Some((_, '\n')) = result {
            self.line += 1;
            self.column = 0;
        }

        result
    }

    fn current_location(&self) -> SourceLocation {
        SourceLocation::new(self.line, self.column)
    }
}

impl<'a> TokenStream for Tokenizer<'a> {
    fn next(&mut self) -> TokenizerResult {
        if self.is_failed {
            return TokenizerResult::End;
        }

        let mut state = TokenizerState::None;

        while let Some((index, character)) = self.read_next() {
            match state {
                TokenizerState::None if character.is_ascii_alphabetic() => {
                    state = TokenizerState::ReadingName { start_index: index }
                }
                TokenizerState::ReadingName { .. } if self.is_name_character(character) => {}
                TokenizerState::ReadingName { start_index }
                    if (character.is_ascii_whitespace()) =>
                {
                    return self.handle_name_or_keyword(&self.data[start_index..index]);
                }
                TokenizerState::ReadingName { .. } => {
                    self.is_failed = true;

                    return TokenizerResult::Err(TokenizerFailure::new(
                        self.current_location(),
                        TokenizerFailureKind::UnexpectedCharacterInName { index },
                    ));
                }
                TokenizerState::None if character == '"' => {
                    state = TokenizerState::ReadingString { start_index: index }
                }
                TokenizerState::ReadingString { .. } if character == '\\' => {
                    match self.iter.peek() {
                        Some((_, '\"')) => {
                            self.read_next();
                        }
                        Some((_, character)) => {
                            self.is_failed = true;
                            let failure_kind =
                                TokenizerFailureKind::UnknownEscapeSequence(*character);
                            return TokenizerResult::Err(TokenizerFailure::new(
                                self.current_location(),
                                failure_kind,
                            ));
                        }
                        _ => {
                            return TokenizerResult::Err(TokenizerFailure::new(
                                self.current_location(),
                                TokenizerFailureKind::UnfinishedEscapeSequence,
                            ));
                        }
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
                TokenizerState::None => {
                    if character.is_ascii_whitespace() {
                        continue;
                    }

                    match character {
                        '{' => {
                            return TokenizerResult::Ok(Token::OpeningBrace);
                        }
                        '}' => {
                            return TokenizerResult::Ok(Token::ClosingBrace);
                        }
                        c => {
                            return TokenizerResult::Err(TokenizerFailure::new(
                                self.current_location(),
                                TokenizerFailureKind::UnexpectedCharacter(c),
                            ));
                        }
                    }
                }
            }
        }

        match state {
            TokenizerState::ReadingName { start_index } => {
                self.handle_name_or_keyword(&self.data[start_index..])
            }
            TokenizerState::None => TokenizerResult::End,
            TokenizerState::ReadingString { .. } => {
                TokenizerResult::Err(TokenizerFailure::new(
                    self.current_location(),
                    TokenizerFailureKind::UnclosedString,
                ))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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

    macro_rules! tokenizer_fail_test {
        ( $test_name: ident, $test_string: expr, $expected_error:expr ) => {
            #[test]
            pub fn $test_name() {
                let mut tokenizer = Tokenizer::new($test_string);

                assert_eq!(TokenizerResult::Err($expected_error), tokenizer.next());
                assert_eq!(TokenizerResult::End, tokenizer.next());
            }
        };
    }

    tokenizer_test!(can_tokenize_a_name, "a", Token::Name("a".into()));
    tokenizer_test!(
        can_tokenize_a_multicharacter_name,
        "test",
        Token::Name("test".into())
    );
    tokenizer_test!(
        can_tokenize_multiple_names,
        "something else",
        Token::Name("something".into()),
        Token::Name("else".into())
    );
    tokenizer_test!(
        can_read_names_separated_by_unix_newlines,
        "first\nsecond",
        Token::Name("first".into()),
        Token::Name("second".into())
    );
    tokenizer_fail_test!(
        fails_on_invalid_character_in_name,
        "name\"",
        TokenizerFailure::new(
            SourceLocation::new(0, 5),
            TokenizerFailureKind::UnexpectedCharacterInName { index: 4 }
        )
    );

    #[test]
    pub fn returns_end_after_a_failure() {
        let mut tokenizer = Tokenizer::new("name\" othername");

        assert_eq!(
            TokenizerResult::Err(TokenizerFailure::new(
                SourceLocation::new(0, 5),
                TokenizerFailureKind::UnexpectedCharacterInName { index: 4 }
            )),
            tokenizer.next()
        );
        assert_eq!(TokenizerResult::End, tokenizer.next());
    }

    tokenizer_test!(
        can_read_a_simple_string,
        "\"some string\"",
        Token::String("some string".into())
    );

    tokenizer_fail_test!(
        fails_on_unclosed_string,
        "\"bla",
        TokenizerFailure::new(
            SourceLocation::new(0, 5),
            TokenizerFailureKind::UnclosedString
        )
    );

    tokenizer_test!(
        can_read_a_string_with_escaped_quotation_mark,
        "\"test\\\"some\\\"words\"",
        Token::String("test\"some\"words".into())
    );

    tokenizer_fail_test!(
        fails_on_unknown_escape_sequence,
        "\"\\a",
        TokenizerFailure::new(
            SourceLocation::new(0, 2),
            TokenizerFailureKind::UnknownEscapeSequence('a')
        )
    );
    tokenizer_fail_test!(
        fails_on_unfinished_escape_sequence,
        "\"\\",
        TokenizerFailure::new(
            SourceLocation::new(0, 2),
            TokenizerFailureKind::UnfinishedEscapeSequence
        )
    );

    tokenizer_test!(
        can_read_braces,
        "{}",
        Token::OpeningBrace,
        Token::ClosingBrace
    );
    tokenizer_test!(
        ignores_whitespace,
        "somename \t \"aaa\" \t {\r\n}\t",
        Token::Name("somename".into()),
        Token::String("aaa".into()),
        Token::OpeningBrace,
        Token::ClosingBrace
    );
    tokenizer_fail_test!(
        fails_on_unexpected_character,
        "🆒",
        TokenizerFailure::new(
            SourceLocation::new(0, 1),
            TokenizerFailureKind::UnexpectedCharacter('🆒')
        )
    );

    tokenizer_test!(
        allows_underscore_in_name,
        "na_me",
        Token::Name("na_me".into())
    );

    tokenizer_test!(allows_hyphen_in_name, "na-me", Token::Name("na-me".into()));
    tokenizer_test!(
        allows_digits_in_name,
        "n12345",
        Token::Name("n12345".into())
    );

    tokenizer_test!(handles_slide_as_keyword, "slide", Token::KeywordSlide);
    tokenizer_test!(handles_title_as_keyword, "title", Token::KeywordTitle);
    tokenizer_test!(
        handles_metadata_as_keyword,
        "metadata",
        Token::KeywordMetadata
    );

    tokenizer_fail_test!(
        keeps_track_of_column,
        "    🆒",
        TokenizerFailure::new(
            SourceLocation::new(0, 5),
            TokenizerFailureKind::UnexpectedCharacter('🆒')
        )
    );
    tokenizer_fail_test!(
        keeps_track_of_line,
        "    \n🆒",
        TokenizerFailure::new(
            SourceLocation::new(1, 1),
            TokenizerFailureKind::UnexpectedCharacter('🆒')
        )
    );
}
