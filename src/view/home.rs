use crate::fl;
use cosmic::app::context_drawer;
use cosmic::cosmic_config::{self, CosmicConfigEntry};
use cosmic::iced::alignment::{Horizontal, Vertical};
use cosmic::iced::{self, Alignment, Length, Subscription};
use cosmic::prelude::*;
use cosmic::widget::{self, icon, menu, nav_bar};
use cosmic::{cosmic_theme, theme};

#[derive(Default)]
pub struct ViewModel {
    search: String
}

#[derive(Debug, Clone)]
pub enum Message {
    Settings,
    Help,
    Add
}

impl From<Message> for crate::app::Message {
    fn from(value: Message) -> Self {
        crate::app::Message::Home(value)
    }
}

impl ViewModel {
    pub fn view(&self) -> Element<Message> {
        let action_bar = iced::widget::row![
            widget::button::icon(crate::app::icondata_svg(icondata::AiSettingOutlined))
                .on_press(Message::Settings),
            widget::button::icon(crate::app::icondata_svg(icondata::AiQuestionCircleOutlined))
                .on_press(Message::Help),
            widget::button::icon(crate::app::icondata_svg(icondata::AiPlusOutlined))
                .on_press(Message::Add),
        ];

        let left = iced::widget::column![
            widget::svg("logo.svg"),
            widget::text::title1("Cosmonaute"),
            action_bar
        ]
            .apply(widget::container)
            .center(Length::Fill);

        let right = iced::widget::column![
            widget::text::title1("Documentation"),
            widget::search_input("Search", &self.search),
            widget::scrollable::vertical(widget::text("TODO: Docset list"))
            .width(Length::Fill)
            .height(Length::Fill)
        ];

        widget::row()
            .push(left)
            .push(right)
            .into()
    }

    pub fn update(&mut self, message: Message) -> Task<cosmic::Action<Message>> {
        Task::none()
    }
}