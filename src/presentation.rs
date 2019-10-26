use std::collections::HashMap;
use std::hash::Hash;

#[derive(Debug, Eq, PartialEq)]
pub enum StyleError {
    DuplicateFont(FontDescriptor),
}

#[derive(Debug, Eq, PartialEq)]
pub struct Slide {
    name: String,
}

impl Slide {
    pub fn new(name: String) -> Self {
        Slide { name }
    }
}

#[derive(Debug, Eq, PartialEq, Hash, Clone)]
pub struct FontDescriptor {
    name: String,
    weight: u32,
    italic: bool,
}

#[derive(Debug, Eq, PartialEq)]
pub struct Font {
    path: String,
    descriptor: FontDescriptor,
}

impl Font {
    pub fn new(name: String, path: String, weight: u32, italic: bool) -> Self {
        Self {
            path,
            descriptor: FontDescriptor {
                name,
                weight,
                italic,
            },
        }
    }

    pub fn path(&self) -> &String {
        &self.path
    }
}

#[derive(Debug, Eq, PartialEq)]
pub struct Style {
    fonts: HashMap<FontDescriptor, Font>,
}

impl Style {
    pub fn new(fonts_input: Vec<Font>) -> Result<Self, StyleError> {
        let mut fonts = HashMap::new();
        for font in fonts_input {
            if let Some(font) = fonts.insert(font.descriptor.clone(), font) {
                return Err(StyleError::DuplicateFont(font.descriptor));
            }
        }

        Ok(Self { fonts })
    }

    pub fn empty() -> Self {
        Self {
            fonts: HashMap::new(),
        }
    }

    pub fn fonts(&self) -> Vec<&Font> {
        self.fonts.values().collect()
    }
}

#[derive(Debug, Eq, PartialEq)]
pub struct Presentation {
    name: String,
    slides: Vec<Slide>,
    style: Style,
}

impl Presentation {
    pub fn new(name: String, slides: Vec<Slide>, style: Style) -> Self {
        Presentation {
            name,
            slides,
            style,
        }
    }

    pub fn style(&self) -> &Style {
        &self.style
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    pub fn style_conflicting_fonts() {
        Style::new(vec![
            Font::new("some-font".into(), "/some/path/1".into(), 500, false),
            Font::new("some-font".into(), "/some/path/2".into(), 500, false),
        ])
        .expect_err("Expected error from identical font definitions");
    }
}
