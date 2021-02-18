pub use nu_data::config::NuConfig;
use nu_data::primitive::lookup_ansi_color_style;
use nu_protocol::Value;
use nu_table::{table_theme::TableTheme, text_style::TextStyle, Alignment};
use std::fmt::Debug;

pub trait ConfigExtensions: Debug + Send {
    fn table_mode(&self) -> TableTheme;
    fn disabled_indexes(&self) -> bool;
    fn header_style(&self) -> TextStyle;
}

pub fn header_alignment_from_value(align_value: Option<&Value>) -> nu_table::Alignment {
    match align_value {
        Some(v) => match v
            .as_string()
            .unwrap_or_else(|_| "none".to_string())
            .as_ref()
        {
            "l" | "left" => nu_table::Alignment::Left,
            "c" | "center" => nu_table::Alignment::Center,
            "r" | "right" => nu_table::Alignment::Right,
            _ => nu_table::Alignment::Center,
        },
        _ => nu_table::Alignment::Center,
    }
}

pub fn get_color_from_key_and_subkey(config: &NuConfig, key: &str, subkey: &str) -> Option<Value> {
    let vars = &config.vars;

    if let Some(config_vars) = vars.get(key) {
        for (kee, value) in config_vars.row_entries() {
            if kee == subkey {
                return Some(value.clone());
            }
        }
    }

    None
}

pub fn header_bold_from_value(bold_value: Option<&Value>) -> bool {
    bold_value
        .map(|x| x.as_bool().unwrap_or(true))
        .unwrap_or(true)
}

pub fn table_mode(config: &NuConfig) -> TableTheme {
    let vars = &config.vars;

    vars.get("table_mode")
        .map_or(TableTheme::compact(), |mode| match mode.as_string() {
            Ok(m) if m == "basic" => TableTheme::basic(),
            Ok(m) if m == "compact" => TableTheme::compact(),
            Ok(m) if m == "light" => TableTheme::light(),
            Ok(m) if m == "thin" => TableTheme::thin(),
            Ok(m) if m == "with_love" => TableTheme::with_love(),
            Ok(m) if m == "compact_double" => TableTheme::compact_double(),
            Ok(m) if m == "rounded" => TableTheme::rounded(),
            Ok(m) if m == "reinforced" => TableTheme::reinforced(),
            Ok(m) if m == "heavy" => TableTheme::heavy(),
            Ok(m) if m == "none" => TableTheme::none(),
            _ => TableTheme::compact(),
        })
}

pub fn disabled_indexes(config: &NuConfig) -> bool {
    let vars = &config.vars;

    vars.get("disable_table_indexes")
        .map_or(false, |x| x.as_bool().unwrap_or(false))
}

impl ConfigExtensions for NuConfig {
    fn header_style(&self) -> TextStyle {
        // FIXME: I agree, this is the long way around, please suggest and alternative.
        let head_color = get_color_from_key_and_subkey(self, "color_config", "header_color");
        let head_color_style = match head_color {
            Some(s) => {
                lookup_ansi_color_style(s.as_string().unwrap_or_else(|_| "green".to_string()))
            }
            None => ansi_term::Color::Green.normal(),
        };
        let head_bold = get_color_from_key_and_subkey(self, "color_config", "header_bold");
        let head_bold_bool = match head_bold {
            Some(b) => header_bold_from_value(Some(&b)),
            None => true,
        };
        let head_align = get_color_from_key_and_subkey(self, "color_config", "header_align");
        let head_alignment = match head_align {
            Some(a) => header_alignment_from_value(Some(&a)),
            None => Alignment::Center,
        };

        TextStyle::new()
            .alignment(head_alignment)
            .bold(Some(head_bold_bool))
            .fg(head_color_style
                .foreground
                .unwrap_or(ansi_term::Color::Green))
    }

    fn table_mode(&self) -> TableTheme {
        table_mode(self)
    }

    fn disabled_indexes(&self) -> bool {
        disabled_indexes(self)
    }
}
