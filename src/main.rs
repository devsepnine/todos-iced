#![windows_subsystem = "windows"]

use iced::keyboard::{self, key};
use iced::widget::{
  self, button, center, center_x, checkbox, column, container, keyed_column, mouse_area, row,
  scrollable, text, text_input, Text,
};
use iced::{window, Center, Element, Fill, Font, Function, Subscription, Task as Command, Theme};
use rodio::{Decoder, OutputStreamBuilder, Sink};
use serde::{Deserialize, Serialize};
use std::io::Cursor;
use uuid::Uuid;

use rust_embed::RustEmbed;

#[derive(RustEmbed)]
#[folder = "i18n"]
struct Localizations;

use i18n_embed::{
  fluent::{fluent_language_loader, FluentLanguageLoader},
  DesktopLanguageRequester,
};
use i18n_embed_fl::fl;
use std::sync::LazyLock;
use i18n_embed::unic_langid::LanguageIdentifier;

pub static LANGUAGE_LOADER: LazyLock<FluentLanguageLoader> = LazyLock::new(|| {
    let loader = fluent_language_loader!();
    let requested_languages = DesktopLanguageRequester::requested_languages();
    let _result = i18n_embed::select(&loader, &Localizations, &requested_languages);
    loader
});

// Simple fallback until we implement proper runtime locale switching
fn translate(key: &str, language: Language) -> String {
    // Special case for language toggle - show the OTHER language
    if key == "language-toggle" {
        return match language {
            Language::Korean => "En".to_string(),
            Language::English => "Ko".to_string(),
        };
    }

    // For other keys, use fl! macro (which uses system locale, not the language parameter)
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

fn translate_tasks_left(count: usize, _language: Language) -> String {
    fl!(LANGUAGE_LOADER, "tasks-left", count = count)
}

fn main() -> iced::Result {
    #[cfg(not(target_arch = "wasm32"))]
    tracing_subscriber::fmt::init();

    // Initialize i18n by accessing the lazy static
    LazyLock::force(&LANGUAGE_LOADER);
    init_audio();

    iced::application(Todos::new, Todos::update, Todos::view)
        .subscription(Todos::subscription)
        .title(Todos::title)
        .font(Todos::ICON_FONT)
        .window(window::Settings {
            size: (500.0, 800.0).into(),
            min_size: Some((500.0, 600.0).into()),
            ..window::Settings::default()
        })
        .run()
}

fn init_audio() {
    match OutputStreamBuilder::open_default_stream() {
        Ok(_stream_handle) => {
            println!("Audio stream initialized successfully");
        }
        Err(e) => {
            eprintln!("Failed to initialize audio stream: {:?}", e);
        }
    }
}

#[derive(Debug)]
enum Todos {
    Loading,
    Loaded(State),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Language {
    English,
    Korean,
}

impl Default for Language {
    fn default() -> Self {
        // Use system locale detection
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

#[derive(Debug, Default)]
struct State {
    input_value: String,
    filter: Filter,
    tasks: Vec<Task>,
    dirty: bool,
    saving: bool,
    input_hovered: bool,
    language: Language,
}

#[derive(Debug, Clone)]
enum Message {
    Loaded(Result<SavedState, LoadError>),
    Saved(Result<(), SaveError>),
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
    const ICON_FONT: &'static [u8] = include_bytes!("../fonts/icons.ttf");

    fn new() -> (Self, Command<Message>) {
        println!("Data saved at: {:?}", SavedState::path());

        (
            Self::Loading,
            Command::perform(SavedState::load(), Message::Loaded),
        )
    }

    fn title(&self) -> String {
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

    fn update(&mut self, message: Message) -> Command<Message> {
        match self {
            Todos::Loading => {
                match message {
                    Message::Loaded(Ok(state)) => {
                        *self = Todos::Loaded(State {
                            input_value: state.input_value,
                            filter: state.filter,
                            tasks: state.tasks,
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
                            widget::focus_previous()
                        } else {
                            widget::focus_next()
                        }
                    }
                    Message::ToggleFullscreen(mode) => {
                        window::latest().and_then(move |window| window::set_mode(window, mode))
                    }
                    Message::LanguageChanged(language) => {
                        state.language = language;
                        
                        // Update the language loader immediately
                        let lang_ids = match language {
                            Language::Korean => vec!["ko-KR".parse::<LanguageIdentifier>().unwrap()],
                            Language::English => vec!["en-US".parse::<LanguageIdentifier>().unwrap()],
                        };
                        let _result = i18n_embed::select(&*LANGUAGE_LOADER, &Localizations, &lang_ids);
                        
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

    fn view(&self) -> Element<'_, Message> {
        match self {
            Todos::Loading => loading_message(Language::default()),
            Todos::Loaded(State {
                input_value,
                filter,
                tasks,
                input_hovered,
                language,
                ..
            }) => {
                let input = text_input(&translate("add-task-placeholder", *language), input_value)
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
                    .width(Fill);

                let plus_icon = text("+").size(20).style(|theme: &Theme| text::Style {
                    color: Some(theme.extended_palette().background.strong.text),
                });

                let input_container = container(row![plus_icon, input].spacing(8).align_y(Center))
                    .padding(iced::Padding {
                        top: 8.0,
                        left: 16.0,
                        bottom: 8.0,
                        right: 4.0,
                    })
                    .style(move |theme| {
                        let background_color = if *input_hovered {
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
                    .width(Fill);

                let controls = view_controls(tasks, *filter, *language);
                let filtered_tasks = tasks.iter().filter(|task| filter.matches(task));

                let tasks: Element<_> = if filtered_tasks.count() > 0 {
                    keyed_column(
                        tasks
                            .iter()
                            .enumerate()
                            .filter(|(_, task)| filter.matches(task))
                            .map(|(i, task)| {
                                (task.id, task.view(i).map(Message::TaskMessage.with(i)))
                            }),
                    )
                    .spacing(10)
                    .height(Fill)
                    .into()
                } else {
                    {
                        let key = match filter {
                            Filter::All => "empty-no-tasks",
                            Filter::Active => "empty-all-done",
                            Filter::Completed => "empty-no-completed",
                        };
                        empty_message(key, *language)
                    }
                };

                let header = controls;
                let scrollable_tasks = scrollable(tasks).height(Fill);
                let footer_input = mouse_area(input_container)
                    .on_enter(Message::InputHovered)
                    .on_exit(Message::InputUnhovered);
                let content = column![header, scrollable_tasks, footer_input]
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
        }
    }

    fn subscription(&self) -> Subscription<Message> {
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
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Task {
    #[serde(default = "Uuid::new_v4")]
    id: Uuid,
    description: String,
    completed: bool,

    #[serde(skip)]
    state: TaskState,
}

#[derive(Debug, Clone)]
pub enum TaskState {
    Idle,
    Editing,
}

impl Default for TaskState {
    fn default() -> Self {
        Self::Idle
    }
}

#[derive(Debug, Clone)]
pub enum TaskMessage {
    Completed(bool),
    Edit,
    DescriptionEdited(String),
    FinishEdition,
    Delete,
}

impl Task {
    fn text_input_id(i: usize) -> text_input::Id {
        text_input::Id::new(format!("task-{i}"))
    }

    fn new(description: String) -> Self {
        Task {
            id: Uuid::new_v4(),
            description,
            completed: false,
            state: TaskState::Idle,
        }
    }

    fn update(&mut self, message: TaskMessage) {
        match message {
            TaskMessage::Completed(completed) => {
                self.completed = completed;

                if completed {
                    play_done_sound();
                }
            }
            TaskMessage::Edit => {
                self.state = TaskState::Editing;
            }
            TaskMessage::DescriptionEdited(new_description) => {
                self.description = new_description;
            }
            TaskMessage::FinishEdition => {
                if !self.description.is_empty() {
                    self.state = TaskState::Idle;
                }
            }
            TaskMessage::Delete => {}
        }
    }

    fn view(&self, i: usize) -> Element<'_, TaskMessage> {
        let content = match &self.state {
            TaskState::Idle => {
                let checkbox = checkbox(&self.description, self.completed)
                    .on_toggle(TaskMessage::Completed)
                    .width(Fill)
                    .size(18)
                    .text_shaping(text::Shaping::Advanced);

                row![
                    checkbox,
                    button(edit_icon())
                        .on_press(TaskMessage::Edit)
                        .padding(4)
                        .style(button::text),
                ]
                .spacing(20)
                .align_y(Center)
            }
            TaskState::Editing => {
                let text_input = text_input(
                    &fl!(LANGUAGE_LOADER, "describe-task-placeholder"),
                    &self.description,
                )
                .id(Self::text_input_id(i))
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
            }
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
}

fn view_controls(
    tasks: &[Task],
    current_filter: Filter,
    language: Language,
) -> Element<'_, Message> {
    let tasks_left = tasks.iter().filter(|task| !task.completed).count();

    let filter_button = |key, filter, current_filter| {
        let label = text(translate(key, language));

        let button = button(label).style(if filter == current_filter {
            button::primary
        } else {
            button::text
        });

        button
            .on_press(Message::FilterChanged(filter))
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
                .on_press(Message::LanguageChanged(match language {
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum Filter {
    #[default]
    All,
    Active,
    Completed,
}

impl Filter {
    fn matches(self, task: &Task) -> bool {
        match self {
            Filter::All => true,
            Filter::Active => !task.completed,
            Filter::Completed => task.completed,
        }
    }
}

fn loading_message<'a>(language: Language) -> Element<'a, Message> {
    center(
        text(translate("loading", language))
            .width(Fill)
            .align_x(Center)
            .size(50),
    )
    .into()
}

fn empty_message(key: &str, language: Language) -> Element<'_, Message> {
    center(
        text(translate(key, language))
            .width(Fill)
            .size(25)
            .align_x(Center)
            .style(subtle),
    )
    .height(200)
    .into()
}

fn icon(unicode: char) -> Text<'static> {
    text(unicode.to_string())
        .font(Font::with_name("Iced-Todos-Icons"))
        .width(20)
        .align_x(Center)
        .shaping(text::Shaping::Basic)
}

fn edit_icon() -> Text<'static> {
    icon('\u{F303}')
}

fn delete_icon() -> Text<'static> {
    icon('\u{F1F8}')
}

fn subtle(theme: &Theme) -> text::Style {
    text::Style {
        color: Some(theme.extended_palette().background.strongest.color),
    }
}

const DONE_SOUND: &[u8] = include_bytes!("../assets/done.wav");

fn play_done_sound() {
    std::thread::spawn(|| {
        let wav_data = DONE_SOUND;
        match OutputStreamBuilder::open_default_stream() {
            Ok(stream_handler) => {
                let sink = Sink::connect_new(&stream_handler.mixer());
                let cursor = Cursor::new(wav_data.as_ref());
                match Decoder::new(cursor) {
                    Ok(source) => {
                        sink.append(source);
                        sink.sleep_until_end();
                    }
                    Err(e) => eprintln!("Failed to decode audio: {}", e),
                }
            }
            Err(e) => eprintln!("Failed to open audio stream: {}", e),
        }
    });
}

// Persistence
#[derive(Debug, Clone, Serialize, Deserialize)]
struct SavedState {
    input_value: String,
    filter: Filter,
    tasks: Vec<Task>,
}

#[derive(Debug, Clone)]
enum LoadError {
    File,
    Format,
}

#[derive(Debug, Clone)]
enum SaveError {
    Write,
    Format,
}

#[cfg(not(target_arch = "wasm32"))]
impl SavedState {
    fn path() -> std::path::PathBuf {
        let mut path =
            if let Some(project_dirs) = directories::ProjectDirs::from("rs", "Iced", "Todos") {
                project_dirs.data_dir().into()
            } else {
                std::env::current_dir().unwrap_or_default()
            };

        path.push("todos.json");

        path
    }

    async fn load() -> Result<SavedState, LoadError> {
        let contents = tokio::fs::read_to_string(Self::path())
            .await
            .map_err(|_| LoadError::File)?;

        serde_json::from_str(&contents).map_err(|_| LoadError::Format)
    }

    async fn save(self) -> Result<(), SaveError> {
        let json = serde_json::to_string_pretty(&self).map_err(|_| SaveError::Format)?;

        let path = Self::path();

        if let Some(dir) = path.parent() {
            tokio::fs::create_dir_all(dir)
                .await
                .map_err(|_| SaveError::Write)?;
        }

        tokio::fs::write(path, json.as_bytes())
            .await
            .map_err(|_| SaveError::Write)?;

        tokio::time::sleep(std::time::Duration::from_secs(2)).await;

        Ok(())
    }
}

#[cfg(target_arch = "wasm32")]
impl SavedState {
    fn storage() -> Option<web_sys::Storage> {
        let window = web_sys::window()?;

        window.local_storage().ok()?
    }

    async fn load() -> Result<SavedState, LoadError> {
        let storage = Self::storage().ok_or(LoadError::File)?;

        let contents = storage
            .get_item("state")
            .map_err(|_| LoadError::File)?
            .ok_or(LoadError::File)?;

        serde_json::from_str(&contents).map_err(|_| LoadError::Format)
    }

    async fn save(self) -> Result<(), SaveError> {
        let storage = Self::storage().ok_or(SaveError::Write)?;

        let json = serde_json::to_string_pretty(&self).map_err(|_| SaveError::Format)?;

        storage
            .set_item("state", &json)
            .map_err(|_| SaveError::Write)?;

        wasmtimer::tokio::sleep(std::time::Duration::from_secs(2)).await;

        Ok(())
    }
}
