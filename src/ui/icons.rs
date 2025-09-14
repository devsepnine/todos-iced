use iced::widget::{text, Text};
use iced::{Center, Font};

pub fn icon(unicode: char) -> Text<'static> {
    text(unicode.to_string())
        .font(Font::with_name("Iced-Todos-Icons"))
        .width(20)
        .align_x(Center)
        .shaping(text::Shaping::Basic)
}

pub fn edit_icon() -> Text<'static> {
    icon('\u{F303}')
}

pub fn delete_icon() -> Text<'static> {
    icon('\u{F1F8}')
}

pub fn plus_icon() -> Text<'static> {
    text("+").size(20).style(|theme: &iced::Theme| text::Style {
        color: Some(theme.extended_palette().background.strong.text),
    })
}