use iced::widget::text;
use iced::Theme;

pub fn subtle(theme: &Theme) -> text::Style {
    text::Style {
        color: Some(theme.extended_palette().background.strongest.color),
    }
}