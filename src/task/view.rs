use iced::widget::{button, checkbox, container, row, text_input};
use iced::{Center, Element, Fill, Theme};

use crate::i18n::LANGUAGE_LOADER;
use crate::ui::icons::{delete_icon, edit_icon};
use i18n_embed_fl::fl;

use super::{Task, TaskMessage, TaskState};

pub fn task_view(task: &Task, index: usize) -> Element<'_, TaskMessage> {
    let content = match task.state() {
        TaskState::Idle => idle_view(task),
        TaskState::Editing => editing_view(task, index),
    };

    container(content)
        .padding(12)
        .style(|theme| container::Style {
            background: Some(theme.extended_palette().background.weakest.color.into()),
            border: iced::Border {
                color: theme.extended_palette().background.weakest.color,
                width: 1.0,
                radius: 8.0.into(),
            },
            ..Default::default()
        })
        .into()
}

fn idle_view(task: &Task) -> Element<'_, TaskMessage> {
    let checkbox = checkbox(task.description(), task.completed())
        .on_toggle(TaskMessage::Completed)
        .width(Fill)
        .size(18)
        .text_shaping(iced::widget::text::Shaping::Advanced);

    row![
        checkbox,
        button(edit_icon())
            .on_press(TaskMessage::Edit)
            .padding(4)
            .style(button::text),
    ]
    .spacing(20)
    .align_y(Center)
    .into()
}

fn editing_view(task: &Task, index: usize) -> Element<'_, TaskMessage> {
    let text_input = text_input(
        &fl!(LANGUAGE_LOADER, "describe-task-placeholder"),
        task.description(),
    )
    .id(Task::text_input_id(index))
    .on_input(TaskMessage::DescriptionEdited)
    .on_submit(TaskMessage::FinishEdition)
    .padding(10)
    .style(|theme: &Theme, status| {
        let default_style = text_input::default(theme, status);

        text_input::Style {
            background: iced::Color::TRANSPARENT.into(),
            border: iced::Border {
                color: iced::Color::TRANSPARENT,
                width: 0.0,
                radius: 0.0.into(),
            },
            icon: default_style.icon,
            placeholder: default_style.placeholder,
            value: default_style.value,
            selection: default_style.selection,
        }
    });

    row![
        text_input,
        button(row![delete_icon()].spacing(10).align_y(Center))
            .on_press(TaskMessage::Delete)
            .padding(10)
            .style(button::danger)
    ]
    .spacing(20)
    .align_y(Center)
    .into()
}