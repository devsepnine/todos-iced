pub mod filter;
pub mod persistence;

use crate::i18n::Language;
use crate::task::Task;
pub use filter::Filter;

#[derive(Debug, Default)]
pub struct State {
    pub input_value: String,
    pub filter: Filter,
    pub tasks: Vec<Task>,
    pub dirty: bool,
    pub saving: bool,
    pub input_hovered: bool,
    pub language: Language,
}