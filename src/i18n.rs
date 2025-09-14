use rust_embed::RustEmbed;
use i18n_embed::{
    fluent::{fluent_language_loader, FluentLanguageLoader},
    DesktopLanguageRequester,
};
use i18n_embed_fl::fl;
use std::sync::LazyLock;
use i18n_embed::unic_langid::LanguageIdentifier;

#[derive(RustEmbed)]
#[folder = "i18n"]
struct Localizations;

pub static LANGUAGE_LOADER: LazyLock<FluentLanguageLoader> = LazyLock::new(|| {
    let loader = fluent_language_loader!();
    let requested_languages = DesktopLanguageRequester::requested_languages();
    let _result = i18n_embed::select(&loader, &Localizations, &requested_languages);
    loader
});

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Language {
    English,
    Korean,
}

impl Default for Language {
    fn default() -> Self {
        let requested_languages = DesktopLanguageRequester::requested_languages();
        if requested_languages
            .iter()
            .any(|lang| lang.language.as_str() == "ko")
        {
            Language::Korean
        } else {
            Language::English
        }
    }
}

pub fn translate(key: &str, language: Language) -> String {
    if key == "language-toggle" {
        return match language {
            Language::Korean => "En".to_string(),
            Language::English => "Ko".to_string(),
        };
    }

    match key {
        "app-title" => fl!(LANGUAGE_LOADER, "app-title"),
        "loading" => fl!(LANGUAGE_LOADER, "loading"),
        "add-task-placeholder" => fl!(LANGUAGE_LOADER, "add-task-placeholder"),
        "describe-task-placeholder" => fl!(LANGUAGE_LOADER, "describe-task-placeholder"),
        "filter-all" => fl!(LANGUAGE_LOADER, "filter-all"),
        "filter-active" => fl!(LANGUAGE_LOADER, "filter-active"),
        "filter-completed" => fl!(LANGUAGE_LOADER, "filter-completed"),
        "empty-no-tasks" => fl!(LANGUAGE_LOADER, "empty-no-tasks"),
        "empty-all-done" => fl!(LANGUAGE_LOADER, "empty-all-done"),
        "empty-no-completed" => fl!(LANGUAGE_LOADER, "empty-no-completed"),
        _ => key.to_string(),
    }
}

pub fn translate_tasks_left(count: usize, _language: Language) -> String {
    fl!(LANGUAGE_LOADER, "tasks-left", count = count)
}

pub fn update_language(language: Language) {
    let lang_ids = match language {
        Language::Korean => vec!["ko-KR".parse::<LanguageIdentifier>().unwrap()],
        Language::English => vec!["en-US".parse::<LanguageIdentifier>().unwrap()],
    };
    let _result = i18n_embed::select(&*LANGUAGE_LOADER, &Localizations, &lang_ids);
}