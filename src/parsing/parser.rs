use super::token_stream::{
    PeekableTokenStream, Token, TokenStream, TokenizerFailure, TokenizerResult,
};

#[derive(Debug, Eq, PartialEq)]
pub enum ParserError {
    UnexpectedToken,       // todo add information about the actual/expected token
    UnexpectedEndOfStream, // todo add information about the expected token
    TokenizerFailure(TokenizerFailure),
}

#[derive(Debug, Eq, PartialEq)]
pub struct Slide {
    name: String,
}

#[derive(Debug, Eq, PartialEq)]
pub struct Presentation {
    title: String,
    slides: Vec<Slide>,
}

impl Slide {
    pub fn new(name: String) -> Self {
        Slide { name }
    }
}

impl Presentation {
    pub fn new(title: String, slides: Vec<Slide>) -> Self {
        Presentation { title, slides }
    }
}

pub struct Parser<'a, T: TokenStream> {
    token_stream: PeekableTokenStream<'a, T>,
}

macro_rules! consume {
    ($self:expr, $expected:pat) => {
        match $self.token_stream.next() {
            TokenizerResult::Ok($expected) => {}
            result => return Self::handle_invalid_result(&result),
        }
    };
    ($self:expr, $expected:pat => $action:expr) => {
        match $self.token_stream.next() {
            TokenizerResult::Ok($expected) => $action,
            result => return Self::handle_invalid_result(&result),
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
        let mut title: String = "".into();

        match self.token_stream.peek() {
            None => {}
            Some(TokenizerResult::End) => {}
            Some(TokenizerResult::Ok(Token::KeywordSlide)) => slides.push(self.parse_slide()?),
            Some(TokenizerResult::Ok(Token::KeywordMetadata)) => title = self.parse_metadata()?,
            Some(result) => return Self::handle_invalid_result(result),
        }

        Ok(Presentation::new(title, slides))
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

    fn handle_invalid_result<TOk>(result: &TokenizerResult) -> Result<TOk, ParserError> {
        Err(match result {
            TokenizerResult::Ok(_) => ParserError::UnexpectedToken,
            TokenizerResult::Err(error) => ParserError::TokenizerFailure(*error),
            TokenizerResult::End => ParserError::UnexpectedEndOfStream,
        })
    }
}

#[cfg(test)]
mod test {
    use super::super::token_stream::{MockTokenStream, SourceLocation, TokenizerFailureKind};
    use super::*;

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

    macro_rules! parser_test {
        ($test_name:ident, $results:expr, $expected_presentation:expr) => {
            #[test]
            pub fn $test_name() {
                let mut tokens = $results;
                let mut stream = MockTokenStream::new(&mut tokens);
                let mut parser = Parser::new(&mut stream);

                assert_eq!(parser.parse().unwrap(), $expected_presentation);
            }
        };
    }

    // todo are presentations without metadata allowed?
    parser_test!(
        can_parse_slide_block,
        vec![
            TokenizerResult::Ok(Token::KeywordSlide),
            TokenizerResult::Ok(Token::String("some slide".into())),
            TokenizerResult::Ok(Token::OpeningBrace),
            TokenizerResult::Ok(Token::ClosingBrace),
        ],
        Presentation::new("".into(), vec![Slide::new("some slide".into())])
    );

    parser_test!(
        can_parse_metadata_block,
        vec![
            TokenizerResult::Ok(Token::KeywordMetadata),
            TokenizerResult::Ok(Token::OpeningBrace),
            TokenizerResult::Ok(Token::KeywordTitle),
            TokenizerResult::Ok(Token::String("some title".into())),
            TokenizerResult::Ok(Token::ClosingBrace)
        ],
        Presentation::new("some title".into(), vec![])
    );

    parser_test_fail!(
        fails_if_block_type_is_not_slide,
        vec![
            TokenizerResult::Ok(Token::Name("notslide".into())),
            TokenizerResult::Ok(Token::String("some slide".into())),
            TokenizerResult::Ok(Token::OpeningBrace),
            TokenizerResult::Ok(Token::ClosingBrace),
        ],
        ParserError::UnexpectedToken
    );

    parser_test_fail!(
        fails_on_missing_braces,
        vec![
            TokenizerResult::Ok(Token::KeywordSlide),
            TokenizerResult::Ok(Token::String("some slide".into())),
        ],
        ParserError::UnexpectedEndOfStream
    );

    parser_test_fail!(
        fails_on_unexpected_token_after_slide_name,
        vec![
            TokenizerResult::Ok(Token::KeywordSlide),
            TokenizerResult::Ok(Token::String("some slide".into())),
            TokenizerResult::Ok(Token::ClosingBrace)
        ],
        ParserError::UnexpectedToken
    );

    parser_test_fail!(
        fails_on_unexpected_token_after_slide_opening_brace,
        vec![
            TokenizerResult::Ok(Token::KeywordSlide),
            TokenizerResult::Ok(Token::String("some slide".into())),
            TokenizerResult::Ok(Token::OpeningBrace),
            TokenizerResult::Ok(Token::OpeningBrace),
        ],
        ParserError::UnexpectedToken
    );

    parser_test_fail!(
        passes_tokenization_failure_through,
        vec![TokenizerResult::Err(TokenizerFailure::new(
            SourceLocation::new(0, 0),
            TokenizerFailureKind::UnclosedString
        ))],
        ParserError::TokenizerFailure(TokenizerFailure::new(
            SourceLocation::new(0, 0),
            TokenizerFailureKind::UnclosedString
        ))
    );
}
