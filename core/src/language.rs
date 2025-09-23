// SPDX-FileCopyrightText: 2024-2025 Simon Repp
// SPDX-License-Identifier: AGPL-3.0-or-later

use chrono::Locale as ChronoLocale;
use icu_collator::{Collator, CollatorBorrowed};
use icu_collator::options::{CollatorOptions, Strength};
use icu_locale::Locale as IcuLocale;
use icu_locale::locale as icu_locale;
use log::warn;
use serde_derive::{Deserialize, Serialize};

use hyper8_translations::Translations;

pub const AVAILABLE_LANGUAGES: [AvailableLanguage; 2] = [
    AvailableLanguage { code: "de", name: "Deutsch" },
    AvailableLanguage { code: "en", name: "English" }
];

pub struct AvailableLanguage {
    pub code: &'static str,
    pub name: &'static str
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(from = "&str", into = "String")]
pub struct Language {
    pub chrono_locale: ChronoLocale,
    pub code: String,
    pub default: bool,
    pub icu_collator: CollatorBorrowed<'static>,
    pub icu_locale: IcuLocale,
    pub text_direction: TextDirection,
    pub translations: Translations
}

#[derive(Clone, Debug)]
pub enum TextDirection {
    Ltr,
    Rtl
}

impl Language {
    /// Utility function to obtain an ICU collator from an ICU locale,
    /// which we need to do both on initializing and cloning [Language]
    /// because ICU Collator does not implement Clone.
    fn icu_collator(icu_locale: IcuLocale) -> CollatorBorrowed<'static> {
        let mut collator_options = CollatorOptions::default();
        collator_options.strength = Some(Strength::Primary);

        match Collator::try_new(icu_locale.clone().into(), collator_options) {
            Ok(collator) => collator,
            Err(err) => {
                warn!("No ICU collator could be initialized from ICU locale {icu_locale} ({err}).");

                // Try to fall back to the default collator
                Collator::try_new(Default::default(), collator_options).unwrap()
            }
        }
    }

    pub fn default() -> Language {
        Language::new(
            ChronoLocale::en_US,
            String::from("en"),
            true,
            icu_locale!("en"),
            TextDirection::Ltr,
            Translations::EN
        )
    }

    /// As in "specified explicitly in the manifest by the user",
    /// so we know we need to persist this to the manifest.
    fn explicit(
        chrono_locale: ChronoLocale,
        code: String,
        icu_locale: IcuLocale,
        text_direction: TextDirection,
        translations: Translations
    ) -> Language {
        Language::new(
            chrono_locale,
            code,
            false,
            icu_locale,
            text_direction,
            translations
        )
    }

    pub fn from_code(code: &str) -> Language {
        match code {
            "de" =>  {
                Language::explicit(
                    ChronoLocale::de_AT_euro,
                    String::from("de"),
                    icu_locale!("de"),
                    TextDirection::Ltr,
                    Translations::DE
                )
            }
            "en" => {
                 Language::explicit(
                    ChronoLocale::en_US,
                    String::from("en"),
                    icu_locale!("en"),
                    TextDirection::Ltr,
                    Translations::EN
                )
            }
            _ => {
                let icu_locale = code.parse::<IcuLocale>()
                    .unwrap_or_else(|err| {
                        warn!("Language code {code} could not be parsed as an ICU locale ({err}).");
                        icu_locale!("en")
                    });

                Language::explicit(
                    ChronoLocale::en_US,
                    code.to_string(),
                    icu_locale,
                    TextDirection::from_code(code),
                    Translations::EN
                )
            }
        }
    }

    pub fn is_rtl(&self) -> bool {
        match self.text_direction {
            TextDirection::Ltr => false,
            TextDirection::Rtl => true
        }
    }

    fn new(
        chrono_locale: ChronoLocale,
        code: String,
        default: bool,
        icu_locale: IcuLocale,
        text_direction: TextDirection,
        translations: Translations
    ) -> Language {
        let icu_collator = Language::icu_collator(icu_locale.clone());

        Language {
            chrono_locale,
            code: code.to_string(),
            default,
            icu_collator,
            icu_locale,
            text_direction,
            translations
        }
    }
}

impl Clone for Language {
    /// ICU's Collator/CollatorBorrowed does not implement clone, therefore
    /// for cloning [Language] we must recreate the collactor manually from
    /// the (cloneable) ICU Locale, just as we do on intializing [Language].
    fn clone(&self) -> Language {
        let chrono_locale = self.chrono_locale;
        let code = self.code.clone();
        let default = self.default;
        let icu_collator = Language::icu_collator(self.icu_locale.clone());
        let icu_locale = self.icu_locale.clone();
        let text_direction = self.text_direction.clone();
        let translations = self.translations.clone();

        Language {
            chrono_locale,
            code,
            default,
            icu_collator,
            icu_locale,
            text_direction,
            translations,
        }
    }
}

impl From<&str> for Language {
    fn from(code: &str) -> Language {
        Language::from_code(code)
    }
}

impl Into<String> for Language {
    fn into(self) -> String {
        self.code.to_string()
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
