use std::io::{self, Write};


#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Colour {
    Purple,
    Orange,
    Red,
    Green,
    Blue,
    Yellow,
    Cyan,
    Magenta,
    Black,
    White,
    Rgb(u8, u8, u8),
    Reset,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Style {
    pub bold: bool,
    pub underline: bool,
    pub dim: bool,
}


impl Style {
    pub const fn new() -> Self {
        Self { bold: false, underline: false, dim: false }
    }

    pub const fn bold() -> Self {
        Self { bold: true, underline: false, dim: false }
    }

     pub const fn underline() -> Self {
        Self { bold: false, underline: true, dim: false }
    }

    pub const fn dim() -> Self {
        Self { bold: false, underline: false, dim: true }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Coloured<'a> {
    pub colour: Colour,
    pub style: Style,
    pub text: &'a str,
}

impl<'a> Coloured<'a> {
    pub fn new(text: &'a str, colour: Colour) -> Self {
        Self { colour, style: Style::new(), text }
    }

    pub fn with_style(text: &'a str, colour: Colour, style: Style) -> Self {
        Self { colour, style, text }
    }

    pub fn write_to<W: Write>(&self, w: &mut W) -> io::Result<()> {
        write_colour_prefix(w, self.colour, self.style)?;
        w.write_all(self.text.as_bytes())?;
        write_reset(w)
    }
}

pub fn write_coloured<W: Write>(
    w: &mut W,
    text: &str,
    colour: Colour,
    style: Style,
) -> io::Result<()> {
   Coloured::with_style(text, colour, style).write_to(w) 
}

pub fn write_colour_prefix<W: Write>(
   w: &mut W,
   colour: Colour,
   style: Style, 
) -> io::Result<()> {
    w.write_all(b"\x1b[")?;
    let mut first = true;

    if style.bold {
        if !first { w.write_all(b";")?; }
        w.write_all(b"1")?;
        first = false;
    }
    if style.dim {
        if !first { w.write_all(b";")?; }
        w.write_all(b"2")?;
        first = false;
    }
    if style.underline {
        if !first { w.write_all(b";")?; }
        w.write_all(b"4")?;
        first = false;
    }

    match colour {
        Colour::Purple => {
            if !first { w.write_all(b";")?; }
            w.write_all(b"35")?;
        }
        Colour::Orange => {
            if !first { w.write_all(b";")?; }
            w.write_all(b"38;5;208")?;
        }
        Colour::Red => {
            if !first { w.write_all(b";")?; }
            w.write_all(b"31")?;
        }
        Colour::Green => {
            if !first { w.write_all(b";")?; }
            w.write_all(b"32")?;
        }
        Colour::Blue => {
            if !first { w.write_all(b";")?; }
            w.write_all(b"34")?;
        }
         Colour::Yellow => {
            if !first { w.write_all(b";")?; }
            w.write_all(b"33")?;
        }
        Colour::Cyan => {
            if !first { w.write_all(b";")?; }
            w.write_all(b"36")?;
        }
        Colour::Magenta => {
            if !first { w.write_all(b";")?; }
            w.write_all(b"35")?;
        }
        Colour::Black => {
            if !first { w.write_all(b";")?; }
            w.write_all(b"30")?;
        }
        Colour::White => {
            if !first { w.write_all(b";")?; }
            w.write_all(b"37")?;
        }
        Colour::Rgb(r, g, b) => {
            if !first { w.write_all(b";")?; }
            write!(w, "38;2;{};{};{}", r, g, b)?;
        }
        Colour::Reset => {
            w.write_all(b"0")?;
        }
    }

    w.write_all(b"m")
}

pub fn write_reset<W: Write>(w: &mut W) -> io::Result<()> {
    w.write_all(b"\x1b[0m")
}

pub fn purple_bold<'a>(text: &'a str) -> Coloured<'a> {
    Coloured::with_style(text, Colour::Purple, Style::bold())
}

pub fn orange_bold<'a>(text: &'a str) -> Coloured<'a> {
    Coloured::with_style(text, Colour::Orange, Style::bold())
}

pub fn red_bold<'a>(text: &'a str) -> Coloured<'a> {
    Coloured::with_style(text, Colour::Red, Style::bold())
}

pub fn white_bold<'a>(text: &'a str) -> Coloured<'a> {
    Coloured::with_style(text, Colour::White, Style::bold())
}

pub fn black_bold<'a>(text: &'a str) -> Coloured<'a> {
    Coloured::with_style(text, Colour::Black, Style::bold())
}