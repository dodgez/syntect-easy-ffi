#![feature(box_into_inner)]
#![feature(vec_into_raw_parts)]

use std::ffi::{CStr, CString};
use std::path::Path;
use syntect::{
    easy::HighlightLines,
    highlighting::{Color as SyntectColor, Style as SyntectStyle, ThemeSet},
    parsing::SyntaxSet,
    util::LinesWithEndings,
};

#[repr(C)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

impl From<SyntectColor> for Color {
    fn from(c: SyntectColor) -> Color {
        Color {
            r: c.r,
            g: c.g,
            b: c.b,
            a: c.a,
        }
    }
}

#[repr(C)]
pub struct OptionColor {
    pub present: u8,
    pub color: Color,
}

impl From<Option<SyntectColor>> for OptionColor {
    fn from(c: Option<SyntectColor>) -> OptionColor {
        match c {
            Some(c) => OptionColor {
                present: 1,
                color: c.into(),
            },
            _ => OptionColor {
                present: 0,
                color: Color {
                    r: 0,
                    g: 0,
                    b: 0,
                    a: 0,
                },
            },
        }
    }
}

#[repr(C)]
pub struct Style {
    pub foreground: Color,
    pub background: Color,
    pub font_style: u8,
}

impl From<SyntectStyle> for Style {
    fn from(s: SyntectStyle) -> Style {
        Style {
            foreground: s.foreground.into(),
            background: s.background.into(),
            font_style: s.font_style.bits(),
        }
    }
}

#[repr(C)]
pub struct StyledString {
    pub style: Style,
    pub string: *const i8,
}

impl From<(SyntectStyle, String)> for StyledString {
    fn from(styled_string: (SyntectStyle, String)) -> StyledString {
        StyledString {
            style: styled_string.0.into(),
            string: CString::new(styled_string.1).unwrap().into_raw(),
        }
    }
}

#[repr(C)]
pub struct Highlighted {
    lines: *const StyledString,
    count: usize,
}

impl From<Vec<(SyntectStyle, String)>> for Highlighted {
    fn from(lines: Vec<(SyntectStyle, String)>) -> Highlighted {
        let converted_lines: Vec<StyledString> = lines.iter().map(|s| s.clone().into()).collect();
        Highlighted {
            count: converted_lines.len(),
            lines: converted_lines.into_raw_parts().0,
        }
    }
}

#[no_mangle]
#[allow(clippy::not_unsafe_ptr_arg_deref)]
pub extern "C" fn load_default_syntaxes(syntaxes_folder: *const i8) -> *mut SyntaxSet {
    let syntaxes_folder = unsafe { CStr::from_ptr(syntaxes_folder) }.to_str().unwrap();
    let mut sb = SyntaxSet::load_defaults_newlines().into_builder();
    if Path::new(syntaxes_folder).exists() {
        sb.add_from_folder(syntaxes_folder, true).unwrap();
    }
    let ss = sb.build();
    Box::into_raw(Box::new(ss))
}

#[no_mangle]
#[allow(clippy::not_unsafe_ptr_arg_deref)]
pub extern "C" fn load_default_themes(themes_folder: *const i8) -> *mut ThemeSet {
    let themes_folder = unsafe { CStr::from_ptr(themes_folder) }.to_str().unwrap();
    let mut themes = ThemeSet::load_defaults();
    if Path::new(themes_folder).exists() {
        themes.add_from_folder(themes_folder).unwrap();
    }
    Box::into_raw(Box::new(themes))
}

#[no_mangle]
#[allow(clippy::not_unsafe_ptr_arg_deref)]
pub extern "C" fn get_theme_setting(
    themes_folder: *const i8,
    theme: *const i8,
    theme_setting: *const i8,
) -> OptionColor {
    let theme_set: ThemeSet =
        unsafe { Box::into_inner(Box::from_raw(load_default_themes(themes_folder))) };
    let theme = unsafe { CStr::from_ptr(theme) }.to_str().unwrap();
    let theme = &theme_set.themes[theme].settings;
    let theme_setting = unsafe { CStr::from_ptr(theme_setting) }.to_str().unwrap();
    match theme_setting {
        "foreground" => theme.foreground.into(),
        "background" => theme.background.into(),
        _ => None.into(),
    }
}

#[no_mangle]
#[allow(clippy::not_unsafe_ptr_arg_deref)]
pub extern "C" fn highlight_string(
    content: *const i8,
    file_ext: *const i8,
    theme: *const i8,
    syntaxes: *const SyntaxSet,
    themes: *const ThemeSet,
) -> Highlighted {
    let content = unsafe { CStr::from_ptr(content) }.to_str().unwrap();
    let file_ext = unsafe { CStr::from_ptr(file_ext) }.to_str().unwrap();
    let theme = unsafe { CStr::from_ptr(theme) }.to_str().unwrap();
    let ss: &SyntaxSet = unsafe { &*syntaxes };
    let syntax = ss
        .find_syntax_by_token(file_ext)
        .unwrap_or_else(|| ss.find_syntax_plain_text());
    let themes: &ThemeSet = unsafe { &*themes };
    let mut h: HighlightLines = HighlightLines::new(syntax, &themes.themes[theme]);
    let mut highlighted: Vec<(SyntectStyle, &str)> = Vec::new();
    for line in LinesWithEndings::from(content) {
        highlighted.append(&mut h.highlight(line, &ss));
    }
    highlighted
        .iter()
        .map(|&(style, str)| (style, str.to_owned()))
        .collect::<Vec<(SyntectStyle, String)>>()
        .into()
}
