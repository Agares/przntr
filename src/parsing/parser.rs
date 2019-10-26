use super::token_stream::{
    PeekableTokenStream, Token, TokenStream, TokenizerFailure, TokenizerResult,
};
use crate::parsing::token_stream::SourceLocationRange;
use crate::presentation::{Font, Presentation, Slide, Style, StyleError};

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
    InvalidStyleDefinition(StyleError),
}

impl From<StyleError> for ParserError {
    fn from(style_error: StyleError) -> Self {
        ParserError::InvalidStyleDefinition(style_error)
    }
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
    ($self:expr, $($expected:pat => $action:expr),+) => {
        match $self.token_stream.next() {
            $(
                TokenizerResult::Ok($expected, _) => $action,
            )+
            result => {
                return Self::handle_invalid_result(
                    &result,
                    [$(stringify!($expected),)+].iter().map(|name| name.to_string().replace("Token::", "")).collect::<Vec<String>>().join(", ")
                )
            }
        }
    }
}

macro_rules! peek_decide {
    ($self:expr, $($expected:pat => $action:expr),+) => {
        peek_decide!(
            $self,
            $( $expected => $action ),+
            ; return Self::handle_invalid_result(
                    &TokenizerResult::End,
                    [$(stringify!($expected),)+].iter().map(|name| name.to_string().replace("Token::", "")).collect::<Vec<String>>().join(", ")
            )
        );
    };
    ($self:expr, $($expected:pat => $action:expr),+;$end_action:expr) => {
        match $self.token_stream.peek() {
            None|Some(TokenizerResult::End) => $end_action,
            $(
                Some(TokenizerResult::Ok($expected, _)) => $action,
            )+
            Some(result) => {
                return Self::handle_invalid_result(
                    &result,
                    [$(stringify!($expected),)+].iter().map(|name| name.to_string().replace("Token::", "")).collect::<Vec<String>>().join(", ")
                )
            }
        }
    }
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
            peek_decide!(
                self,
                Token::KeywordSlide => slides.push(self.parse_slide()?),
                Token::KeywordStyle => style = Some(self.parse_style()?)
                ;break
            );
        }

        Ok(Presentation::new(
            title,
            slides,
            style.unwrap_or_else(Style::empty),
        ))
    }

    fn parse_slide(&mut self) -> Result<Slide, ParserError> {
        consume!(self, Token::KeywordSlide);
        let slide_name = consume!(self, Token::String(slide_name) => slide_name);
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

        loop {
            peek_decide!(
                self,
                Token::KeywordFont => fonts.push(self.parse_font()?),
                Token::ClosingBrace => { consume!(self, Token::ClosingBrace); break }
            );
        }

        Ok(Style::new(fonts)?)
    }

    fn parse_font(&mut self) -> Result<Font, ParserError> {
        let mut italic = false;
        let mut name: Option<String> = None;
        let mut path: Option<String> = None;
        let mut weight: Option<i128> = None;

        consume!(self, Token::KeywordFont);
        consume!(self, Token::OpeningBrace);

        loop {
            consume!(
                self,
                Token::KeywordName => name = consume!(self, Token::Name(font_name) => Some(font_name)),
                Token::KeywordPath => path = consume!(self, Token::String(font_path) => Some(font_path)),
                Token::KeywordWeight => weight = consume!(self, Token::Integer(font_weight) => Some(font_weight)),
                Token::KeywordItalic => italic = true,
                Token::ClosingBrace => break
            );

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
    use crate::parsing::tokenizer::Tokenizer;
    use crate::presentation::Font;

    macro_rules! parser_test_fail {
        ($test_name:ident, $results:expr, $expected_error:expr) => {
            #[test]
            pub fn $test_name() {
                let mut tokenizer = Tokenizer::new($results);
                let mut parser = Parser::new(&mut tokenizer);

                let error: ParserError = $expected_error;
                assert_eq!(parser.parse(), Err(error));
            }
        };
    }

    macro_rules! parser_test {
        ($test_name:ident, $results:expr, $expected_presentation:expr) => {
            #[test]
            pub fn $test_name() {
                let mut tokenizer = Tokenizer::new($results);
                let mut parser = Parser::new(&mut tokenizer);

                let parsed = parser.parse().unwrap();

                assert_eq!(parsed, $expected_presentation);
            }
        };
    }

    parser_test_fail!(
        fails_on_slide_before_metadata,
        "slide \"some slide\" {}",
        ParserError::UnexpectedToken {
            actual: "KeywordSlide".into(),
            expected: "KeywordMetadata".into(),
            location: SourceLocationRange::new(
                SourceLocation::new(0, 1),
                SourceLocation::new(0, 6)
            )
        }
    );

    parser_test!(
        can_parse_metadata_block,
        "metadata { title \"some title\" }",
        Presentation::new("some title".into(), vec![], Style::new(vec![]).unwrap())
    );

    parser_test!(
        can_parse_slide_after_metadata,
        "metadata { title \"some title\" } slide \"first slide\" {}",
        Presentation::new(
            "some title".into(),
            vec![Slide::new("first slide".into())],
            Style::new(vec![]).unwrap()
        )
    );

    parser_test_fail!(
        fails_if_block_type_is_not_slide,
        "metadata { title \"some title\" } notslide \"some slide\" {}",
        ParserError::UnexpectedToken {
            actual: "Name(\"notslide\")".into(),
            expected: "KeywordSlide, KeywordStyle".into(),
            location: SourceLocationRange::new(
                SourceLocation::new(0, 33),
                SourceLocation::new(0, 41)
            )
        }
    );

    parser_test_fail!(
        fails_on_missing_braces,
        "metadata { title \"some title\" } slide \"some slide\"",
        ParserError::UnexpectedEndOfStream {
            expected: "OpeningBrace".into()
        }
    );

    parser_test_fail!(
        fails_on_unexpected_token_after_slide_name,
        "metadata { title \"some title\" } slide \"some slide\" }",
        ParserError::UnexpectedToken {
            actual: "ClosingBrace".into(),
            expected: "OpeningBrace".into(),
            location: SourceLocationRange::new_single(SourceLocation::new(0, 52))
        }
    );

    parser_test_fail!(
        fails_on_unexpected_token_after_slide_opening_brace,
        "metadata { title \"some title\" } slide \"some slide\" {{",
        ParserError::UnexpectedToken {
            actual: "OpeningBrace".into(),
            expected: "ClosingBrace".into(),
            location: SourceLocationRange::new_single(SourceLocation::new(0, 53))
        }
    );

    parser_test!(
        can_parse_single_font,
        "metadata { title \"some title\" } style { font { path \"some_path\", name my-wonderful-font, weight 500,}}",
        Presentation::new(
            "some title".into(),
            vec![],
            Style::new(vec![Font::new(
                "my-wonderful-font".into(),
                "some_path".into(),
                500,
                false
            )]).unwrap()
        )
    );

    parser_test!(
        can_parse_italic_font,
        "metadata { title \"some title\" } style { font { path \"some_path\", name my-wonderful-font, weight 500, italic, } }",
        Presentation::new(
            "some title".into(),
            vec![],
            Style::new(vec![Font::new(
                "my-wonderful-font".into(),
                "some_path".into(),
                500,
                true
            )]).unwrap()
        )
    );

    parser_test!(
        slide_after_style,
        "metadata { title \"some title\" } style { font { path \"some_path\", name my-wonderful-font, weight 500, } } slide \"some slide\" {}",
        Presentation::new(
            "some title".into(),
            vec![Slide::new("some slide".into())],
            Style::new(vec![Font::new(
                "my-wonderful-font".into(),
                "some_path".into(),
                500,
                false
            )]).unwrap()
        )
    );

    parser_test!(
        style_with_multiple_fonts,
        "metadata { title \"some title\" } \n\
        style { \n\
            font { path \"path1\", name font-1, weight 500, } \n\
            font { path \"path2\", name font-1, weight 500, italic, } \n\
        }",
        Presentation::new(
            "some title".into(),
            vec![],
            Style::new(vec![
                Font::new("font-1".into(), "path1".into(), 500, false),
                Font::new("font-1".into(), "path2".into(), 500, true)
            ]).unwrap()
        )
    );

    parser_test_fail!(
        fails_on_unexpected_token_in_font_definition,
        "metadata { title \"some title\" } style { font { invalid \"some_path\" } }",
        ParserError::UnexpectedToken {
            actual: "Name(\"invalid\")".into(),
            expected: "KeywordName, KeywordPath, KeywordWeight, KeywordItalic, ClosingBrace".into(),
            location: SourceLocationRange::new(
                SourceLocation::new(0, 48),
                SourceLocation::new(0, 55)
            )
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
