use chrono::Locale as ChronoLocale;

use crate::Translations;
use crate::util::is_rtl_language;

#[derive(Clone, Debug)]
pub struct Language {
    pub chrono_locale: ChronoLocale,
    pub code: String,
    pub default: bool,
    /// Whether this is a right-to-left language
    pub rtl: bool,
    pub translations: Translations
}

impl Language {
    pub fn default() -> Language {
        Language {
            chrono_locale: ChronoLocale::en_US,
            code: String::from("en"),
            default: true,
            rtl: false,
            translations: Translations::en()
        }
    }

    pub fn from_code(code: &str) -> Language {
        let (chrono_locale, translations) = match code {
            "de" => (ChronoLocale::de_AT_euro, Translations::de()),
            "en" => (ChronoLocale::en_US, Translations::en()),
            _ => (ChronoLocale::en_US, Translations::en())
        };

        Language::manual(chrono_locale, code, translations)
    }

    pub fn manual(
        chrono_locale: ChronoLocale,
        code: &str,
        translations: Translations
    ) -> Language {
        Language {
            chrono_locale,
            code: code.to_string(),
            default: false,
            rtl: is_rtl_language(code),
            translations
        }
    }
}