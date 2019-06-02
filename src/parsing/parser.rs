use super::tokenizer::{Token, TokenStream, TokenizerResult};

#[derive(Debug, Eq, PartialEq)]
pub enum ParserError {
    UnexpectedToken, // todo add information about the actual/expected token
    InvalidSectionName { actual: String, expected: String },
    UnexpectedEndOfStream, // todo add information about the expected token
}

#[derive(Debug, Eq, PartialEq)]
pub struct Slide {
    name: String,
}

#[derive(Debug, Eq, PartialEq)]
pub struct Presentation {
    slides: Vec<Slide>,
}

impl Slide {
    pub fn new(name: String) -> Self {
        Slide { name }
    }
}

impl Presentation {
    pub fn new(slides: Vec<Slide>) -> Self {
        Presentation { slides }
    }
}

struct PeekableTokenStream<'a, T: TokenStream> {
    token_stream: &'a mut T,
    peeked: Option<TokenizerResult<'a>>,
}

pub struct Parser<'a, T: TokenStream> {
    token_stream: &'a mut T,
}

const NAME_SLIDE: &str = "slide";

impl<'a, T: TokenStream> Parser<'a, T> {
    pub fn new(token_stream: &'a mut T) -> Self {
        Parser { token_stream }
    }

    pub fn parse(&mut self) -> Result<Presentation, ParserError> {
        let slide = self.parse_block()?;

        Ok(Presentation::new(vec![slide]))
    }

    fn parse_block(&mut self) -> Result<Slide, ParserError> {
        let name = match self.token_stream.next() {
            TokenizerResult::Ok(Token::Name(name)) => Ok(name),
            _ => Err(ParserError::UnexpectedToken),
        }?;

        if name != NAME_SLIDE {
            return Err(ParserError::InvalidSectionName {
                actual: name.into(),
                expected: NAME_SLIDE.into(),
            });
        }

        self.parse_slide_name_and_contents()
    }

    fn parse_slide_name_and_contents(&mut self) -> Result<Slide, ParserError> {
        let slide_name = match self.token_stream.next() {
            TokenizerResult::Ok(Token::String(slide_name)) => Ok(slide_name),
            _ => Err(ParserError::UnexpectedToken),
        }?;

        match self.token_stream.next() {
            TokenizerResult::Ok(Token::OpeningBrace) => {}
            TokenizerResult::Ok(_) => return Err(ParserError::UnexpectedToken),
            _ => return Err(ParserError::UnexpectedEndOfStream),
        }

        match self.token_stream.next() {
            TokenizerResult::Ok(Token::ClosingBrace) => {}
            TokenizerResult::Ok(_) => return Err(ParserError::UnexpectedToken),
            _ => return Err(ParserError::UnexpectedEndOfStream),
        }

        Ok(Slide::new(slide_name))
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use ::std::vec::Drain;

    struct MockTokenStream<'a> {
        iter: Drain<'a, TokenizerResult<'a>>,
    }

    impl<'a> MockTokenStream<'a> {
        pub fn new(results: &'a mut Vec<TokenizerResult<'a>>) -> Self {
            MockTokenStream {
                iter: results.drain(..),
            }
        }
    }

    impl<'a> TokenStream for MockTokenStream<'a> {
        fn next(&mut self) -> TokenizerResult {
            if let Some(x) = self.iter.next() {
                x
            } else {
                TokenizerResult::End
            }
        }
    }

    #[test]
    pub fn can_parse_slide_block() {
        let mut tokens = vec![
            TokenizerResult::Ok(Token::Name("slide")),
            TokenizerResult::Ok(Token::String("some slide".into())),
            TokenizerResult::Ok(Token::OpeningBrace),
            TokenizerResult::Ok(Token::ClosingBrace),
        ];
        let mut stream = MockTokenStream::new(&mut tokens);

        let mut parser = Parser::<MockTokenStream>::new(&mut stream);

        assert_eq!(
            parser.parse().unwrap(),
            Presentation::new(vec![Slide::new("some slide".into())])
        );
    }

    macro_rules! parser_test_fail {
        ($test_name:ident, $results:expr, $expected_error:expr) => {
            #[test]
            pub fn $test_name() {
                let mut tokens = $results;
                let mut stream = MockTokenStream::new(&mut tokens);
                let mut parser = Parser::new(&mut stream);

                assert_eq!(parser.parse(), Err($expected_error));
            }
        };
    }

    parser_test_fail!(
        fails_if_block_type_is_not_slide,
        vec![
            TokenizerResult::Ok(Token::Name("notslide")),
            TokenizerResult::Ok(Token::String("some slide".into())),
            TokenizerResult::Ok(Token::OpeningBrace),
            TokenizerResult::Ok(Token::ClosingBrace),
        ],
        ParserError::InvalidSectionName {
            actual: "notslide".into(),
            expected: "slide".into()
        }
    );

    parser_test_fail!(
        fails_on_missing_braces,
        vec![
            TokenizerResult::Ok(Token::Name("slide")),
            TokenizerResult::Ok(Token::String("some slide".into())),
        ],
        ParserError::UnexpectedEndOfStream
    );

    parser_test_fail!(
        fails_on_unexpected_token_after_slide_name,
        vec![
            TokenizerResult::Ok(Token::Name("slide")),
            TokenizerResult::Ok(Token::String("some slide".into())),
            TokenizerResult::Ok(Token::ClosingBrace)
        ],
        ParserError::UnexpectedToken
    );

    parser_test_fail!(
        fails_on_unexpected_token_after_slide_opening_brace,
        vec![
            TokenizerResult::Ok(Token::Name("slide")),
            TokenizerResult::Ok(Token::String("some slide".into())),
            TokenizerResult::Ok(Token::OpeningBrace),
            TokenizerResult::Ok(Token::OpeningBrace),
        ],
        ParserError::UnexpectedToken
    );
}
