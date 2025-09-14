use iced::keyboard::{self, key};
use iced::widget::{center_x, column, keyed_column, mouse_area, scrollable, text_input};
use iced::{window, Element, Fill, Function, Subscription, Task as Command, Theme};

use crate::i18n::{translate, Language};
use crate::state::{Filter, State};
use crate::task::{Task, TaskMessage};
use crate::ui::{controls::view_controls, styles::subtle};

#[derive(Debug)]
pub enum Todos {
    Loading,
    Loaded(State),
}

#[derive(Debug, Clone)]
pub enum Message {
    Loaded(Result<crate::state::persistence::SavedState, crate::state::persistence::LoadError>),
    Saved(Result<(), crate::state::persistence::SaveError>),
    InputChanged(String),
    InputHovered,
    InputUnhovered,
    CreateTask,
    FilterChanged(Filter),
    TaskMessage(usize, TaskMessage),
    TabPressed { shift: bool },
    ToggleFullscreen(window::Mode),
    LanguageChanged(Language),
}

impl Todos {
    pub const ICON_FONT: &'static [u8] = include_bytes!("../fonts/icons.ttf");

    pub fn new() -> (Self, Command<Message>) {
        use crate::state::persistence::SavedState;
        
        println!("Data saved at: {:?}", SavedState::path());

        (
            Self::Loading,
            Command::perform(SavedState::load(), Message::Loaded),
        )
    }

    pub fn title(&self) -> String {
        let (dirty, language) = match self {
            Todos::Loading => (false, Language::default()),
            Todos::Loaded(state) => (state.dirty, state.language),
        };

        format!(
            "{}{}",
            translate("app-title", language),
            if dirty { "..." } else { "" }
        )
    }

    pub fn update(&mut self, message: Message) -> Command<Message> {
        match self {
            Todos::Loading => {
                match message {
                    Message::Loaded(Ok(saved_state)) => {
                        *self = Todos::Loaded(State {
                            input_value: saved_state.input_value,
                            filter: saved_state.filter,
                            tasks: saved_state.tasks,
                            ..State::default()
                        });
                    }
                    Message::Loaded(Err(_)) => {
                        *self = Todos::Loaded(State::default());
                    }
                    _ => {}
                }

                text_input::focus("new-task")
            }
            Todos::Loaded(state) => {
                let mut saved = false;

                let command = match message {
                    Message::InputChanged(value) => {
                        state.input_value = value;
                        Command::none()
                    }
                    Message::InputHovered => {
                        state.input_hovered = true;
                        Command::none()
                    }
                    Message::InputUnhovered => {
                        state.input_hovered = false;
                        Command::none()
                    }
                    Message::CreateTask => {
                        if !state.input_value.is_empty() {
                            state.tasks.push(Task::new(state.input_value.clone()));
                            state.input_value.clear();
                        }
                        Command::none()
                    }
                    Message::FilterChanged(filter) => {
                        state.filter = filter;
                        Command::none()
                    }
                    Message::TaskMessage(i, TaskMessage::Delete) => {
                        state.tasks.remove(i);
                        Command::none()
                    }
                    Message::TaskMessage(i, task_message) => {
                        if let Some(task) = state.tasks.get_mut(i) {
                            let should_focus = matches!(task_message, TaskMessage::Edit);

                            task.update(task_message);

                            if should_focus {
                                let id = Task::text_input_id(i);
                                Command::batch(vec![
                                    text_input::focus(id.clone()),
                                    text_input::select_all(id),
                                ])
                            } else {
                                Command::none()
                            }
                        } else {
                            Command::none()
                        }
                    }
                    Message::Saved(_result) => {
                        state.saving = false;
                        saved = true;
                        Command::none()
                    }
                    Message::TabPressed { shift } => {
                        if shift {
                            iced::widget::focus_previous()
                        } else {
                            iced::widget::focus_next()
                        }
                    }
                    Message::ToggleFullscreen(mode) => {
                        window::latest().and_then(move |window| window::set_mode(window, mode))
                    }
                    Message::LanguageChanged(language) => {
                        state.language = language;
                        crate::i18n::update_language(language);
                        Command::none()
                    }
                    Message::Loaded(_) => Command::none(),
                };

                if !saved {
                    state.dirty = true;
                }

                let save = if state.dirty && !state.saving {
                    state.dirty = false;
                    state.saving = true;

                    use crate::state::persistence::SavedState;
                    Command::perform(
                        SavedState {
                            input_value: state.input_value.clone(),
                            filter: state.filter,
                            tasks: state.tasks.clone(),
                        }
                        .save(),
                        Message::Saved,
                    )
                } else {
                    Command::none()
                };

                Command::batch(vec![command, save])
            }
        }
    }

    pub fn view(&self) -> Element<'_, Message> {
        match self {
            Todos::Loading => self.loading_view(),
            Todos::Loaded(state) => self.loaded_view(state),
        }
    }

    pub fn subscription(&self) -> Subscription<Message> {
        keyboard::on_key_press(|key, modifiers| {
            let keyboard::Key::Named(key) = key else {
                return None;
            };

            match (key, modifiers) {
                (key::Named::Tab, _) => Some(Message::TabPressed {
                    shift: modifiers.shift(),
                }),
                (key::Named::ArrowUp, keyboard::Modifiers::SHIFT) => {
                    Some(Message::ToggleFullscreen(window::Mode::Fullscreen))
                }
                (key::Named::ArrowDown, keyboard::Modifiers::SHIFT) => {
                    Some(Message::ToggleFullscreen(window::Mode::Windowed))
                }
                _ => None,
            }
        })
    }

    fn loading_view(&self) -> Element<'_, Message> {
        iced::widget::center(
            iced::widget::text(translate("loading", Language::default()))
                .width(Fill)
                .align_x(iced::Center)
                .size(50),
        )
        .into()
    }

    fn loaded_view<'a>(&'a self, state: &'a State) -> Element<'a, Message> {
        let input = self.create_input(&state.input_value, state.language);
        let input_container = self.create_input_container(input, state.input_hovered);
        let controls = view_controls(&state.tasks, state.filter, state.language);
        let tasks_view = self.create_tasks_view(&state.tasks, state.filter, state.language);

        let footer_input = mouse_area(input_container)
            .on_enter(Message::InputHovered)
            .on_exit(Message::InputUnhovered);

        let content = column![controls, tasks_view, footer_input]
            .spacing(20)
            .height(Fill);

        center_x(content)
            .padding(iced::Padding {
                top: 24.0,
                left: 16.0,
                bottom: 32.0,
                right: 16.0,
            })
            .into()
    }

    fn create_input(&self, input_value: &str, language: Language) -> Element<'_, Message> {
        text_input(&translate("add-task-placeholder", language), input_value)
            .id("new-task")
            .on_input(Message::InputChanged)
            .on_submit(Message::CreateTask)
            .padding(iced::Padding {
                top: 8.0,
                left: 0.0,
                bottom: 8.0,
                right: 0.0,
            })
            .size(16)
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
            })
            .width(Fill)
            .into()
    }

    fn create_input_container<'a>(&self, input: Element<'a, Message>, is_hovered: bool) -> Element<'a, Message> {
        use iced::widget::{container, row};
        use crate::ui::icons::plus_icon;

        let input_row = row![plus_icon(), input].spacing(8).align_y(iced::Center);

        container(input_row)
            .padding(iced::Padding {
                top: 8.0,
                left: 16.0,
                bottom: 8.0,
                right: 4.0,
            })
            .style(move |theme| {
                let background_color = if is_hovered {
                    theme.extended_palette().background.strong.color
                } else {
                    theme.extended_palette().background.weak.color
                };

                container::Style {
                    background: Some(background_color.into()),
                    border: iced::Border {
                        color: background_color,
                        width: 1.0,
                        radius: 12.0.into(),
                    },
                    ..Default::default()
                }
            })
            .width(Fill)
            .into()
    }

    fn create_tasks_view<'a>(&'a self, tasks: &'a [Task], filter: Filter, language: Language) -> Element<'a, Message> {
        let filtered_tasks = tasks.iter().filter(|task| filter.matches(task));

        if filtered_tasks.count() > 0 {
            let tasks_column = keyed_column(
                tasks
                    .iter()
                    .enumerate()
                    .filter(|(_, task)| filter.matches(task))
                    .map(|(i, task)| {
                        (task.id(), task.view(i).map(Message::TaskMessage.with(i)))
                    }),
            )
            .spacing(10)
            .height(Fill);

            scrollable(tasks_column).height(Fill).into()
        } else {
            let key = match filter {
                Filter::All => "empty-no-tasks",
                Filter::Active => "empty-all-done",
                Filter::Completed => "empty-no-completed",
            };
            self.empty_message(key, language)
        }
    }

    fn empty_message<'a>(&'a self, key: &str, language: Language) -> Element<'a, Message> {
        iced::widget::center(
            iced::widget::text(translate(key, language))
                .width(Fill)
                .size(25)
                .align_x(iced::Center)
                .style(subtle),
        )
        .height(Fill)
        .into()
    }
}