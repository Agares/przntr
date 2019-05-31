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

pub struct Parser<'a, T: TokenStream> {
    token_stream: &'a mut T,
}

const NAME_SLIDE:&str = "slide";

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
            _ => return Err(ParserError::UnexpectedEndOfStream), // todo this can also be unexpected token
        }

        match self.token_stream.next() {
            TokenizerResult::Ok(Token::ClosingBrace) => {}
            _ => return Err(ParserError::UnexpectedEndOfStream), // todo this can also be unexpected token
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

    #[test]
    pub fn fails_if_block_type_is_not_slide() {
        let mut tokens = vec![
            TokenizerResult::Ok(Token::Name("notslide")),
            TokenizerResult::Ok(Token::String("some slide".into())),
            TokenizerResult::Ok(Token::OpeningBrace),
            TokenizerResult::Ok(Token::ClosingBrace),
        ];
        let mut stream = MockTokenStream::new(&mut tokens);

        let mut parser = Parser::<MockTokenStream>::new(&mut stream);

        assert_eq!(
            parser.parse(),
            Err(ParserError::InvalidSectionName {
                actual: "notslide".into(),
                expected: "slide".into()
            })
        );
    }

    #[test]
    pub fn fails_on_missing_braces() {
        let mut tokens = vec![
            TokenizerResult::Ok(Token::Name("slide")),
            TokenizerResult::Ok(Token::String("some slide".into())),
        ];
        let mut stream = MockTokenStream::new(&mut tokens);

        let mut parser = Parser::<MockTokenStream>::new(&mut stream);

        assert_eq!(parser.parse(), Err(ParserError::UnexpectedEndOfStream));
    }
}
