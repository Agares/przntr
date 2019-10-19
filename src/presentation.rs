#[derive(Debug, Eq, PartialEq)]
pub struct Slide {
    name: String,
}

impl Slide {
    pub fn new(name: String) -> Self {
        Slide { name }
    }
}

#[derive(Debug, Eq, PartialEq)]
pub struct Font {
    name: String,
    path: String,
    weight: u32,
    italic: bool,
}

impl Font {
    pub fn new(name: String, path: String, weight: u32, italic: bool) -> Self {
        Self {
            name,
            path,
            weight,
            italic,
        }
    }

    pub fn path(&self) -> &String {
        &self.path
    }
}

#[derive(Debug, Eq, PartialEq)]
pub struct Style {
    fonts: Vec<Font>,
}

impl Style {
    pub fn new(fonts: Vec<Font>) -> Self {
        // todo validate fonts (no repeats with the same name, weight and italicness)
        Self { fonts }
    }

    pub fn fonts(&self) -> &Vec<Font> {
        &self.fonts
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
