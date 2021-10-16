#![feature(vec_into_raw_parts)]

use std::ffi::{CStr, CString};
use syntect::{
    easy::HighlightLines, highlighting::ThemeSet, parsing::SyntaxSet, util::LinesWithEndings,
};

#[repr(C)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

#[repr(C)]
pub struct Style {
    pub foreground: Color,
    pub background: Color,
    pub font_style: u8,
}

#[repr(C)]
pub struct StyledString {
    pub style: Style,
    pub string: *const i8,
}

#[repr(C)]
pub struct Highlighted {
    lines: *const StyledString,
    count: usize,
}

#[no_mangle]
#[allow(clippy::not_unsafe_ptr_arg_deref)]
pub extern "C" fn highlight_string(content: *const i8, file_ext: *const i8) -> Highlighted {
    let content = unsafe { CStr::from_ptr(content) }.to_str().unwrap();
    let file_ext = unsafe { CStr::from_ptr(file_ext) }.to_str().unwrap();
    let mut sb = SyntaxSet::load_defaults_newlines().into_builder();
    sb.add_from_folder(".", true).unwrap();
    let ss = sb.build();
    let syntax = ss
        .find_syntax_by_token(file_ext)
        .unwrap_or_else(|| ss.find_syntax_plain_text());
    let themes: ThemeSet = ThemeSet::load_defaults();
    let mut h: HighlightLines = HighlightLines::new(syntax, &themes.themes["base16-ocean.dark"]);
    let mut highlighted: Vec<StyledString> = Vec::new();
    for line in LinesWithEndings::from(content) {
        let mut styled_strings: Vec<StyledString> = h
            .highlight(line, &ss)
            .iter()
            .map(|&(s, str)| {
                let fg = s.foreground;
                let bg = s.background;
                StyledString {
                    style: Style {
                        foreground: Color {
                            r: fg.r,
                            g: fg.g,
                            b: fg.b,
                            a: fg.a,
                        },
                        background: Color {
                            r: bg.r,
                            g: bg.g,
                            b: bg.b,
                            a: bg.a,
                        },
                        font_style: s.font_style.bits(),
                    },
                    string: CString::new(str).unwrap().into_raw(),
                }
            })
            .collect();
        highlighted.append(&mut styled_strings);
    }
    let count = highlighted.len();
    Highlighted {
        lines: highlighted.into_raw_parts().0,
        count,
    }
}
