use crate::fl;
use cosmic::app::context_drawer;
use cosmic::cosmic_config::{self, CosmicConfigEntry};
use cosmic::iced::alignment::{Horizontal, Vertical};
use cosmic::iced::{self, Alignment, Length, Subscription};
use cosmic::prelude::*;
use cosmic::widget::{self, icon, menu, nav_bar};
use cosmic::{cosmic_theme, theme};

pub struct ViewModel {}

#[derive(Debug, Clone)]
pub enum Message {}

impl From<Message> for crate::app::Message {
    fn from(value: Message) -> Self {
        crate::app::Message::Docs(value)
    }
}

impl ViewModel {
    pub fn view(&self) -> Element<Message> {
        // Placeholder for the view implementation
        widget::text::title1("Welcome to Cosmonaute").into()
    }

    pub fn update(&mut self, message: Message) -> Task<cosmic::Action<Message>> {
        Task::none()
    }
}
