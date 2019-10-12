use super::token_stream::{
    PeekableTokenStream, Token, TokenStream, TokenizerFailure, TokenizerResult,
};
use crate::parsing::token_stream::SourceLocationRange;
use crate::presentation::{Font, Presentation, Slide, Style};
use std::string::ParseError;

#[derive(Debug, Eq, PartialEq)]
pub enum ParserError {
    UnexpectedToken {
        actual: String,
        expected: String,
        location: SourceLocationRange,
    },
    UnexpectedEndOfStream {
        expected: String,
    },
    TokenizerFailure(TokenizerFailure),
}

pub struct Parser<'a, T: TokenStream> {
    token_stream: PeekableTokenStream<'a, T>,
}

macro_rules! consume {
    ($self:expr, $expected:pat) => {
        match $self.token_stream.next() {
            TokenizerResult::Ok($expected, _) => {}
            result => {
                return Self::handle_invalid_result(
                    &result,
                    stringify!($expected).to_string().replace("Token::", ""),
                )
            }
        }
    };
    ($self:expr, $expected:pat => $action:expr) => {
        match $self.token_stream.next() {
            TokenizerResult::Ok($expected, _) => $action,
            result => {
                return Self::handle_invalid_result(
                    &result,
                    stringify!($expected).to_string().replace("Token::", ""),
                )
            }
        }
    };
}

impl<'a, T: TokenStream> Parser<'a, T> {
    pub fn new(token_stream: &'a mut T) -> Self {
        Parser {
            token_stream: PeekableTokenStream::new(token_stream),
        }
    }

    pub fn parse(&mut self) -> Result<Presentation, ParserError> {
        let mut slides: Vec<Slide> = Vec::new();
        let mut style = None;
        let title: String = self.parse_metadata()?;

        loop {
            match self.token_stream.peek() {
                Some(TokenizerResult::End) | None => break,
                Some(TokenizerResult::Ok(Token::KeywordSlide, _)) => {
                    slides.push(self.parse_slide()?)
                }
                Some(TokenizerResult::Ok(Token::KeywordStyle, _)) => {
                    style = Some(self.parse_style()?);
                }
                Some(result) => {
                    return Self::handle_invalid_result(
                        result,
                        "KeywordSlide or KeywordMetadata".into(),
                    )
                }
            }
        }

        Ok(Presentation::new(
            title,
            slides,
            style.unwrap_or(Style::new(vec![])),
        ))
    }

    fn parse_slide(&mut self) -> Result<Slide, ParserError> {
        consume!(self, Token::KeywordSlide);
        let slide_name = consume!(self, Token::String(slide_name) => Ok(slide_name))?;
        consume!(self, Token::OpeningBrace);
        consume!(self, Token::ClosingBrace);

        Ok(Slide::new(slide_name))
    }

    fn parse_metadata(&mut self) -> Result<String, ParserError> {
        consume!(self, Token::KeywordMetadata);
        consume!(self, Token::OpeningBrace);
        consume!(self, Token::KeywordTitle);
        let title = consume!(self, Token::String(title) => title);
        consume!(self, Token::ClosingBrace);

        Ok(title)
    }

    fn parse_style(&mut self) -> Result<Style, ParserError> {
        let mut fonts: Vec<Font> = vec![];

        consume!(self, Token::KeywordStyle);
        consume!(self, Token::OpeningBrace);

        match self.token_stream.peek() {
            Some(TokenizerResult::Ok(Token::KeywordFont, _)) => {
                fonts.push(self.parse_font()?);
            }
            Some(result) => return Self::handle_invalid_result(result, "KeywordFont".into()),
            None => unreachable!(), // todo verify unreachability and consider adding an actual message
        }

        consume!(self, Token::ClosingBrace);

        Ok(Style::new(fonts))
    }

    fn parse_font(&mut self) -> Result<Font, ParserError> {
        let mut italic = false;
        let mut name: Option<String> = None;
        let mut path: Option<String> = None;
        let mut weight: Option<i128> = None;

        consume!(self, Token::KeywordFont);
        consume!(self, Token::OpeningBrace);

        while let TokenizerResult::Ok(token, location) = self.token_stream.next() {
            // todo try_consume! macro?
            if let Token::KeywordName = token {
                name = Some(consume!(self, Token::Name(font_name) => font_name));
            } else if let Token::KeywordPath = token {
                path = Some(consume!(self, Token::String(font_path) => font_path));
            } else if let Token::KeywordWeight = token {
                weight = Some(consume!(self, Token::Integer(font_weight) => font_weight));
            } else if let Token::ClosingBrace = token {
                break;
            } else {
                return Err(ParserError::UnexpectedToken {
                    expected: "KeywordName, KeywordPath, KeywordWeight or ClosingBrace".into(),
                    actual: format!("{:?}", token),
                    location: location.clone(),
                });
            }

            consume!(self, Token::Comma);
        }

        // todo return error instead of unwrap panicking
        Ok(Font::new(
            name.unwrap(),
            path.unwrap(),
            weight.unwrap() as u32,
            italic,
        ))
    }

    fn handle_invalid_result<TOk>(
        result: &TokenizerResult,
        expected: String,
    ) -> Result<TOk, ParserError> {
        Err(match result {
            TokenizerResult::Ok(token, location) => ParserError::UnexpectedToken {
                actual: format!("{:?}", token),
                expected,
                location: *location,
            },
            TokenizerResult::Err(error) => ParserError::TokenizerFailure(error.clone()),
            TokenizerResult::End => ParserError::UnexpectedEndOfStream { expected },
        })
    }
}

#[cfg(test)]
mod test {
    use super::super::token_stream::{
        MockTokenStream, SourceLocation, SourceLocationRange, TokenizerFailureKind,
    };
    use super::*;
    use crate::presentation::Font;

    macro_rules! parser_test_fail {
        ($test_name:ident, $results:expr, $expected_error:expr) => {
            #[test]
            pub fn $test_name() {
                let mut tokens = $results
                    .drain(..)
                    .map(|token| {
                        TokenizerResult::Ok(
                            token,
                            SourceLocationRange::new_single(SourceLocation::new(0, 0)),
                        )
                    })
                    .collect();
                let mut stream = MockTokenStream::new(&mut tokens);
                let mut parser = Parser::new(&mut stream);

                let error: ParserError = $expected_error;
                assert_eq!(parser.parse(), Err(error));
            }
        };
    }

    macro_rules! parser_test {
        ($test_name:ident, $results:expr, $expected_presentation:expr) => {
            #[test]
            pub fn $test_name() {
                let mut tokens = $results
                    .drain(..)
                    .map(|token| {
                        TokenizerResult::Ok(
                            token,
                            SourceLocationRange::new_single(SourceLocation::new(0, 0)),
                        )
                    })
                    .collect();
                let mut stream = MockTokenStream::new(&mut tokens);
                let mut parser = Parser::new(&mut stream);

                let parsed = parser.parse().unwrap();

                assert_eq!(parsed, $expected_presentation);
            }
        };
    }

    parser_test_fail!(
        fails_on_slide_before_metadata,
        vec![
            Token::KeywordSlide,
            Token::String("some slide".into()),
            Token::OpeningBrace,
            Token::ClosingBrace,
        ],
        ParserError::UnexpectedToken {
            actual: "KeywordSlide".into(),
            expected: "KeywordMetadata".into(),
            location: SourceLocationRange::new_single(SourceLocation::new(0, 0))
        }
    );

    parser_test!(
        can_parse_metadata_block,
        vec![
            Token::KeywordMetadata,
            Token::OpeningBrace,
            Token::KeywordTitle,
            Token::String("some title".into()),
            Token::ClosingBrace,
        ],
        Presentation::new("some title".into(), vec![], Style::new(vec![]))
    );

    parser_test!(
        can_parse_slide_after_metadata,
        vec![
            Token::KeywordMetadata,
            Token::OpeningBrace,
            Token::KeywordTitle,
            Token::String("some title".into()),
            Token::ClosingBrace,
            Token::KeywordSlide,
            Token::String("first slide".into()),
            Token::OpeningBrace,
            Token::ClosingBrace
        ],
        Presentation::new(
            "some title".into(),
            vec![Slide::new("first slide".into())],
            Style::new(vec![])
        )
    );

    parser_test_fail!(
        fails_if_block_type_is_not_slide,
        vec![
            Token::KeywordMetadata,
            Token::OpeningBrace,
            Token::KeywordTitle,
            Token::String("some title".into()),
            Token::ClosingBrace,
            Token::Name("notslide".into()),
            Token::String("some slide".into()),
            Token::OpeningBrace,
            Token::ClosingBrace,
        ],
        ParserError::UnexpectedToken {
            actual: "Name(\"notslide\")".into(),
            expected: "KeywordSlide or KeywordMetadata".into(),
            location: SourceLocationRange::new_single(SourceLocation::new(0, 0))
        }
    );

    parser_test_fail!(
        fails_on_missing_braces,
        vec![
            Token::KeywordMetadata,
            Token::OpeningBrace,
            Token::KeywordTitle,
            Token::String("some title".into()),
            Token::ClosingBrace,
            Token::KeywordSlide,
            Token::String("some slide".into()),
        ],
        ParserError::UnexpectedEndOfStream {
            expected: "OpeningBrace".into()
        }
    );

    parser_test_fail!(
        fails_on_unexpected_token_after_slide_name,
        vec![
            Token::KeywordMetadata,
            Token::OpeningBrace,
            Token::KeywordTitle,
            Token::String("some title".into()),
            Token::ClosingBrace,
            Token::KeywordSlide,
            Token::String("some slide".into()),
            Token::ClosingBrace,
        ],
        ParserError::UnexpectedToken {
            actual: "ClosingBrace".into(),
            expected: "OpeningBrace".into(),
            location: SourceLocationRange::new_single(SourceLocation::new(0, 0))
        }
    );

    parser_test_fail!(
        fails_on_unexpected_token_after_slide_opening_brace,
        vec![
            Token::KeywordMetadata,
            Token::OpeningBrace,
            Token::KeywordTitle,
            Token::String("some title".into()),
            Token::ClosingBrace,
            Token::KeywordSlide,
            Token::String("some slide".into()),
            Token::OpeningBrace,
            Token::OpeningBrace,
        ],
        ParserError::UnexpectedToken {
            actual: "OpeningBrace".into(),
            expected: "ClosingBrace".into(),
            location: SourceLocationRange::new_single(SourceLocation::new(0, 0))
        }
    );

    parser_test!(
        can_parse_single_font,
        vec![
            Token::KeywordMetadata,
            Token::OpeningBrace,
            Token::KeywordTitle,
            Token::String("some title".into()),
            Token::ClosingBrace,
            Token::KeywordStyle,
            Token::OpeningBrace,
            Token::KeywordFont,
            Token::OpeningBrace,
            Token::KeywordPath,
            Token::String("some_path".into()),
            Token::Comma,
            Token::KeywordName,
            Token::Name("my-wonderful-font".into()),
            Token::Comma,
            Token::KeywordWeight,
            Token::Integer(500),
            Token::Comma,
            Token::ClosingBrace,
            Token::ClosingBrace
        ],
        Presentation::new(
            "some title".into(),
            vec![],
            Style::new(vec![Font::new(
                "my-wonderful-font".into(),
                "some_path".into(),
                500,
                false
            )])
        )
    );

    parser_test!(
        slide_after_style,
        vec![
            Token::KeywordMetadata,
            Token::OpeningBrace,
            Token::KeywordTitle,
            Token::String("some title".into()),
            Token::ClosingBrace,
            Token::KeywordStyle,
            Token::OpeningBrace,
            Token::KeywordFont,
            Token::OpeningBrace,
            Token::KeywordPath,
            Token::String("some_path".into()),
            Token::Comma,
            Token::KeywordName,
            Token::Name("my-wonderful-font".into()),
            Token::Comma,
            Token::KeywordWeight,
            Token::Integer(500),
            Token::Comma,
            Token::ClosingBrace,
            Token::ClosingBrace,
            Token::KeywordSlide,
            Token::String("some slide".into()),
            Token::OpeningBrace,
            Token::ClosingBrace
        ],
        Presentation::new(
            "some title".into(),
            vec![Slide::new("some slide".into())],
            Style::new(vec![Font::new(
                "my-wonderful-font".into(),
                "some_path".into(),
                500,
                false
            )])
        )
    );

    parser_test_fail!(
        fails_on_unexpected_token_in_font_definition,
        vec![
            Token::KeywordMetadata,
            Token::OpeningBrace,
            Token::KeywordTitle,
            Token::String("some title".into()),
            Token::ClosingBrace,
            Token::KeywordStyle,
            Token::OpeningBrace,
            Token::KeywordFont,
            Token::OpeningBrace,
            Token::Name("invalid".into()),
            Token::String("some_path".into()),
            Token::ClosingBrace,
            Token::ClosingBrace
        ],
        ParserError::UnexpectedToken {
            actual: "Name(\"invalid\")".into(),
            expected: "KeywordName, KeywordPath, KeywordWeight or ClosingBrace".into(),
            location: SourceLocationRange::new_single(SourceLocation::new(0, 0))
        }
    );

    #[test]
    pub fn passes_tokenization_failure_through() {
        let mut results = vec![TokenizerResult::Err(TokenizerFailure::new(
            SourceLocation::new(0, 0),
            TokenizerFailureKind::UnclosedString,
        ))];
        let mut stream = MockTokenStream::new(&mut results);
        let mut parser = Parser::new(&mut stream);

        assert_eq!(
            parser.parse(),
            Err(ParserError::TokenizerFailure(TokenizerFailure::new(
                SourceLocation::new(0, 0),
                TokenizerFailureKind::UnclosedString
            )))
        );
    }
}
