#[derive(Debug, Eq, PartialEq)]
pub struct Slide {
    name: String,
}

#[derive(Debug, Eq, PartialEq)]
pub struct Presentation {
    name: String,
    slides: Vec<Slide>,
}

impl Slide {
    pub fn new(name: String) -> Self {
        Slide { name }
    }
}

impl Presentation {
    pub fn new(name: String, slides: Vec<Slide>) -> Self {
        Presentation { name, slides }
    }
}