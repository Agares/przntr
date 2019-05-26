use std::str::CharIndices;

#[derive(Eq, PartialEq, Debug)]
enum TokenizerState {
    None,
    ReadingName { start_index: usize }
}

struct Tokenizer<'a> {
    iter: CharIndices<'a>,
    data: &'a str
}

impl<'a> Tokenizer<'a> {
    pub fn new(data: &'a str) -> Self {
        Tokenizer {
            iter: data.char_indices(),
            data
        }
    }
}

impl<'a> Iterator for Tokenizer<'a> {
    type Item = Token<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        let mut state = TokenizerState::None;

        while let Some((index, character)) = self.iter.next() {
            match state {
                TokenizerState::None if character.is_ascii_alphabetic() => { state = TokenizerState::ReadingName { start_index: index } },
                TokenizerState::ReadingName { .. } if character.is_ascii_alphabetic() => {},
                TokenizerState::ReadingName { start_index } => { return Some(Token::Name(&self.data[start_index..index])) },
                TokenizerState::None => unimplemented!()
            }
        }

        match state {
            TokenizerState::ReadingName { start_index } => Some(Token::Name(&self.data[start_index..])),
            TokenizerState::None => None
        }
    }
}

#[derive(Eq, PartialEq, Debug)]
enum Token<'a> {
    Name(&'a str)
}

fn main() {

}

#[cfg(test)]
mod tests {
    use crate::{Token, Tokenizer};

    macro_rules! tokenizer_test {
        ( $test_name: ident, $test_string: expr, $($expected_token:expr),+ ) => {
            #[test]
            pub fn $test_name() {
                let mut tokenizer = Tokenizer::new($test_string);

                $(
                    assert_eq!($expected_token, tokenizer.next().unwrap());
                )*

                assert_eq!(None, tokenizer.next());
            }
        };
    }

    tokenizer_test!(can_tokenize_a_name, "a", Token::Name("a"));
    tokenizer_test!(can_tokenize_a_multicharacter_name, "test", Token::Name("test"));
    tokenizer_test!(can_tokenize_multiple_names, "something else", Token::Name("something"), Token::Name("else"));
    tokenizer_test!(can_read_names_separated_by_unix_newlines, "first\nsecond", Token::Name("first"), Token::Name("second"));
}