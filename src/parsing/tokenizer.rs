use crate::parsing::token_stream::{
    SourceLocation, SourceLocationRange, Token, TokenStream, TokenizerFailure,
    TokenizerFailureKind, TokenizerResult,
};
use std::iter::Peekable;
use std::str::CharIndices;

#[derive(Eq, PartialEq, Debug)]
enum TokenizerState {
    None,
    ReadingName {
        start_index: usize,
        start_location: SourceLocation,
    },
    ReadingString {
        start_index: usize,
        start_location: SourceLocation,
    },
    ReadingNumber {
        start_index: usize,
        start_location: SourceLocation,
    },
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

    fn handle_name_or_keyword(&self, name: &str, start: SourceLocation) -> TokenizerResult {
        TokenizerResult::Ok(
            match name {
                "slide" => Token::KeywordSlide,
                "title" => Token::KeywordTitle,
                "metadata" => Token::KeywordMetadata,
                "style" => Token::KeywordStyle,
                "font" => Token::KeywordFont,
                "name" => Token::KeywordName,
                "path" => Token::KeywordPath,
                "weight" => Token::KeywordWeight,
                "italic" => Token::KeywordItalic,
                _ => Token::Name(name.into()),
            },
            SourceLocationRange::new(start, self.current_location()),
        )
    }

    fn handle_integer(&self, integer: &str, start: SourceLocation) -> TokenizerResult {
        let parsed = integer.parse();

        if let Ok(parsed) = parsed {
            TokenizerResult::Ok(
                Token::Integer(parsed),
                SourceLocationRange::new(start, self.current_location()),
            )
        } else {
            TokenizerResult::Err(TokenizerFailure::new(
                self.current_location(),
                TokenizerFailureKind::InvalidIntegerValue(integer.into()),
            ))
        }
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

    fn peek(&mut self) -> Option<&(usize, char)> {
        self.iter.peek()
    }

    fn check_next(&mut self, what: char) -> bool {
        if let Some((_, x)) = self.peek() {
            *x == what
        } else {
            false
        }
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
                    state = TokenizerState::ReadingName {
                        start_index: index,
                        start_location: self.current_location(),
                    };

                    if self.check_next(',') {
                        return self.handle_name_or_keyword(
                            &self.data[index..=index],
                            self.current_location(),
                        );
                    }
                }
                TokenizerState::ReadingName {
                    start_index,
                    start_location,
                } => {
                    let is_next_character_a_comma = self.check_next(',');

                    if self.is_name_character(character) && !is_next_character_a_comma {
                        continue;
                    }

                    if character.is_ascii_whitespace() || is_next_character_a_comma {
                        let actual_index = if is_next_character_a_comma { 1 } else { 0 } + index;

                        return self.handle_name_or_keyword(
                            &self.data[start_index..actual_index],
                            start_location,
                        );
                    } else {
                        self.is_failed = true;

                        println!("Failure! {:?}", state);

                        return TokenizerResult::Err(TokenizerFailure::new(
                            self.current_location(),
                            TokenizerFailureKind::UnexpectedCharacterInName { index, character },
                        ));
                    }
                }
                TokenizerState::None if character == '"' => {
                    state = TokenizerState::ReadingString {
                        start_index: index,
                        start_location: self.current_location(),
                    }
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
                TokenizerState::ReadingString {
                    start_index,
                    start_location,
                } if character == '"' => {
                    return TokenizerResult::Ok(
                        Token::String(
                            self.data[start_index + 1..index]
                                .to_owned()
                                .replace("\\\"", "\""),
                        ),
                        SourceLocationRange::new(start_location, self.current_location()),
                    );
                }
                TokenizerState::ReadingString { .. } => {}
                TokenizerState::None if character.is_ascii_digit() || character == '-' => {
                    state = TokenizerState::ReadingNumber {
                        start_index: index,
                        start_location: self.current_location(),
                    }
                }
                TokenizerState::ReadingNumber {
                    start_index,
                    start_location,
                } => match self.peek() {
                    None => {
                        return self
                            .handle_integer(&self.data[start_index..=index], start_location);
                    }
                    Some((_, next_character)) => {
                        if !next_character.is_ascii_digit() {
                            return self
                                .handle_integer(&self.data[start_index..=index], start_location);
                        }
                    }
                },
                TokenizerState::None => {
                    if character.is_ascii_whitespace() {
                        continue;
                    }

                    match character {
                        '{' => {
                            return TokenizerResult::Ok(
                                Token::OpeningBrace,
                                SourceLocationRange::new_single(self.current_location()),
                            );
                        }
                        '}' => {
                            return TokenizerResult::Ok(
                                Token::ClosingBrace,
                                SourceLocationRange::new_single(self.current_location()),
                            );
                        }
                        ',' => {
                            return TokenizerResult::Ok(
                                Token::Comma,
                                SourceLocationRange::new_single(self.current_location()),
                            )
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
            TokenizerState::ReadingName {
                start_index,
                start_location,
            } => self.handle_name_or_keyword(&self.data[start_index..], start_location),
            TokenizerState::None => TokenizerResult::End,
            TokenizerState::ReadingString { .. } => TokenizerResult::Err(TokenizerFailure::new(
                self.current_location(),
                TokenizerFailureKind::UnclosedString,
            )),
            TokenizerState::ReadingNumber {
                start_index,
                start_location,
            } => self.handle_integer(&self.data[start_index..], start_location),
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
                    let result = tokenizer.next();

                    if let TokenizerResult::Ok(token, _) = result {
                        assert_eq!(token, $expected_token);
                    } else {
                        panic!(format!("Unexpected result: {:?}", result));
                    }
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
            TokenizerFailureKind::UnexpectedCharacterInName {
                index: 4,
                character: '\"'
            }
        )
    );

    #[test]
    pub fn returns_end_after_a_failure() {
        let mut tokenizer = Tokenizer::new("name\" othername");

        assert_eq!(
            TokenizerResult::Err(TokenizerFailure::new(
                SourceLocation::new(0, 5),
                TokenizerFailureKind::UnexpectedCharacterInName {
                    index: 4,
                    character: '\"'
                }
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
    tokenizer_test!(handles_style_as_keyword, "style", Token::KeywordStyle);
    tokenizer_test!(handles_font_as_keyword, "font", Token::KeywordFont);
    tokenizer_test!(handles_path_as_keyword, "path", Token::KeywordPath);
    tokenizer_test!(handles_name_as_keyword, "name", Token::KeywordName);
    tokenizer_test!(handles_weight_as_keyword, "weight", Token::KeywordWeight);
    tokenizer_test!(handles_italic_as_keyword, "italic", Token::KeywordItalic);
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

    tokenizer_test!(
        can_handle_commas_between_names,
        "aa,bb,cc",
        Token::Name("aa".into()),
        Token::Comma,
        Token::Name("bb".into()),
        Token::Comma,
        Token::Name("cc".into())
    );
    tokenizer_test!(
        can_handle_commas_with_single_letter_names,
        "a,b,c",
        Token::Name("a".into()),
        Token::Comma,
        Token::Name("b".into()),
        Token::Comma,
        Token::Name("c".into())
    );
    tokenizer_test!(
        can_handle_comma_between_strings,
        "\"a\",\"b\"",
        Token::String("a".into()),
        Token::Comma,
        Token::String("b".into())
    );
    tokenizer_test!(
        can_handle_comma_between_string_and_name,
        "\"a\",b",
        Token::String("a".into()),
        Token::Comma,
        Token::Name("b".into())
    );
    tokenizer_test!(
        can_handle_comma_between_name_and_string,
        "a,\"b\"",
        Token::Name("a".into()),
        Token::Comma,
        Token::String("b".into())
    );
    tokenizer_test!(
        can_handle_positive_integers,
        "123456789",
        Token::Integer(123456789)
    );
    tokenizer_test!(can_handle_negative_integers, "-123", Token::Integer(-123));

    tokenizer_test!(
        can_handle_name_followed_by_integer,
        "aaa 123",
        Token::Name("aaa".into()),
        Token::Integer(123)
    );

    tokenizer_test!(
        can_handle_integer_followed_by_a_name,
        "123 aaa",
        Token::Integer(123),
        Token::Name("aaa".into())
    );

    tokenizer_test!(
        can_handle_name_followed_by_a_comma,
        "aaa,",
        Token::Name("aaa".into()),
        Token::Comma
    );

    tokenizer_test!(
        can_handle_integer_followed_by_a_comma,
        "1234,",
        Token::Integer(1234),
        Token::Comma
    );
}
