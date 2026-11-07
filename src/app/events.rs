use crate::core::events::CoreEvent;

#[derive(Debug, Clone)]
pub enum AppEvent {
    Core(CoreEvent),
    ThemeChanged(bool),
}
