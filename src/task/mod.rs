pub mod view;

use serde::{Deserialize, Serialize};
use uuid::Uuid;
use iced::Element;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Task {
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
    pub fn new(description: String) -> Self {
        Task {
            id: Uuid::new_v4(),
            description,
            completed: false,
            state: TaskState::Idle,
        }
    }

    pub fn id(&self) -> Uuid {
        self.id
    }

    pub fn completed(&self) -> bool {
        self.completed
    }

    pub fn text_input_id(i: usize) -> iced::widget::text_input::Id {
        iced::widget::text_input::Id::new(format!("task-{i}"))
    }

    pub fn update(&mut self, message: TaskMessage) {
        match message {
            TaskMessage::Completed(completed) => {
                self.completed = completed;
                if completed {
                    crate::audio::play_done_sound();
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

    pub fn view(&self, index: usize) -> Element<'_, TaskMessage> {
        view::task_view(self, index)
    }

    pub fn description(&self) -> &str {
        &self.description
    }

    pub fn state(&self) -> &TaskState {
        &self.state
    }
}