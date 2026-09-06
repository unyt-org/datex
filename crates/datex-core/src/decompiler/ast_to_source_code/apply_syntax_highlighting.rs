use crate::{dxb_parser::body::DXBParserError, prelude::*};
#[cfg(feature = "syntax_highlighting_legacy")]
use syntect::{
    easy::HighlightLines,
    highlighting::{Style, Theme, ThemeSet},
    parsing::{SyntaxDefinition, SyntaxSetBuilder},
    util::{LinesWithEndings, as_24_bit_terminal_escaped},
};

#[cfg(not(feature = "syntax_highlighting_legacy"))]
pub fn apply_syntax_highlighting(
    datex_script: String,
) -> Result<String, DXBParserError> {
    // skip syntax highlighting
    Ok(datex_script)
}

#[cfg(all(feature = "std", feature = "syntax_highlighting_legacy"))]
pub fn apply_syntax_highlighting(
    datex_script: String,
) -> Result<String, DXBParserError> {
    use binrw::io::Cursor;
    use core::fmt::Write;

    let mut output = String::new();

    // load datex syntax + custom theme
    static DATEX_SCRIPT_DEF: &str = include_str!(
        "../../../datex-language/datex.tmbundle/Syntaxes/datex.sublime-text"
    );
    static DATEX_THEME_DEF: &str =
        include_str!("../../../datex-language/themes/datex-dark.tmTheme");
    let mut builder = SyntaxSetBuilder::new();
    let syntax = SyntaxDefinition::load_from_str(DATEX_SCRIPT_DEF, true, None)
        .expect("Failed to load syntax definition");
    builder.add(syntax);
    let theme: Theme =
        ThemeSet::load_from_reader(&mut Cursor::new(DATEX_THEME_DEF))
            .expect("Failed to load theme");

    let ps = builder.build();
    let syntax = ps.find_syntax_by_extension("dx").unwrap();
    let mut h = HighlightLines::new(syntax, &theme);

    for line in LinesWithEndings::from(&datex_script) {
        let ranges: Vec<(Style, &str)> = h.highlight_line(line, &ps).unwrap();
        let escaped = as_24_bit_terminal_escaped(&ranges[..], false);
        core::write!(output, "{escaped}")?;
    }
    // reset style
    core::write!(output, "\x1b[0m")?;
    Ok(output)
}

#[cfg(all(not(feature = "std"), feature = "syntax_highlighting_legacy"))]
pub fn apply_syntax_highlighting(
    datex_script: String,
) -> Result<String, DXBParserError> {
    // no_std fallback: no highlighting
    Ok(datex_script)
}
