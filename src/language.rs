// SPDX-FileCopyrightText: 2024 Simon Repp
// SPDX-License-Identifier: AGPL-3.0-or-later

use chrono::Locale as ChronoLocale;

use crate::Translations;

#[derive(Clone, Debug)]
pub struct Language {
    pub chrono_locale: ChronoLocale,
    pub code: String,
    pub default: bool,
    pub text_direction: TextDirection,
    pub translations: Translations
}

#[derive(Clone, Debug)]
pub enum TextDirection {
    Ltr,
    Rtl
}

impl Language {
    pub fn default() -> Language {
        Language {
            chrono_locale: ChronoLocale::en_US,
            code: String::from("en"),
            default: true,
            text_direction: TextDirection::Ltr,
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

    pub fn is_rtl(&self) -> bool {
        match self.text_direction {
            TextDirection::Ltr => false,
            TextDirection::Rtl => true
        }
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
            text_direction: TextDirection::from_code(code),
            translations
        }
    }
}

impl TextDirection {
    /// Language codes compiled based on these (slightly diverging) lists:
    /// - https://meta.wikimedia.org/wiki/Template:List_of_language_names_ordered_by_code
    /// - https://localizejs.com/articles/localizing-for-right-to-left-languages-the-issues-to-consider/
    /// - https://lingohub.com/blog/right-to-left-vs-left-to-right
    /// - https://localizely.com/iso-639-1-list/
    pub fn from_code(code: &str) -> TextDirection {
        match code {
            "ar" |
            "arc" |
            "arz" |
            "ckb" |
            "dv" |
            "fa" |
            "ha" |
            "he" |
            "khw" |
            "ks" |
            "ku" |
            "ps" |
            "sd" |
            "ur" |
            "uz_AF" |
            "yi" => TextDirection::Rtl,
            _ => TextDirection::Ltr
        }
    }
}
