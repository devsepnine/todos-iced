use iced::widget::{button, row, text};
use iced::{Center, Element, Fill};

use crate::i18n::{translate, translate_tasks_left, Language};
use crate::state::Filter;
use crate::task::Task;

pub fn view_controls<'a>(
    tasks: &[Task],
    current_filter: Filter,
    language: Language,
) -> Element<'a, crate::app::Message> {
    let tasks_left = tasks.iter().filter(|task| !task.completed()).count();

    let filter_button = |key, filter, current_filter| {
        let label = text(translate(key, language));

        let button = button(label).style(if filter == current_filter {
            button::primary
        } else {
            button::text
        });

        button
            .on_press(crate::app::Message::FilterChanged(filter))
            .padding(iced::Padding {
                top: 5.0,
                left: 16.0,
                bottom: 5.0,
                right: 16.0,
            })
    };

    row![
        text(translate_tasks_left(tasks_left, language)).width(Fill),
        row![
            filter_button("filter-all", Filter::All, current_filter),
            filter_button("filter-active", Filter::Active, current_filter),
            filter_button("filter-completed", Filter::Completed, current_filter),
            button(text(translate("language-toggle", language)).size(12))
                .on_press(crate::app::Message::LanguageChanged(match language {
                    Language::Korean => Language::English,
                    Language::English => Language::Korean,
                }))
                .padding(iced::Padding {
                    top: 5.0,
                    left: 8.0,
                    bottom: 5.0,
                    right: 8.0,
                })
                .style(button::text),
        ]
        .spacing(10)
        .align_y(Center)
    ]
    .spacing(20)
    .align_y(Center)
    .into()
}