use std::path::PathBuf;

use crate::docset;
use crate::fl;
use cosmic::app::context_drawer;
use cosmic::cosmic_config::{self, CosmicConfigEntry};
use cosmic::iced::alignment::{Horizontal, Vertical};
use cosmic::iced::{self, Alignment, Length, Subscription};
use cosmic::prelude::*;
use cosmic::widget::{self, icon, menu, nav_bar};
use cosmic::{cosmic_theme, theme};
use derive_more::From;

#[derive(Default)]
pub struct ViewModel {
    search: String,
}

#[derive(Debug, Clone, From)]
pub enum Message {
    SearchInput(String),
    DocsetMessage(crate::docset::Message),
    Settings,
    Help,
    Add,
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
                .on_press(Message::Settings)
                .medium(),
            widget::button::icon(crate::app::icondata_svg(icondata::AiQuestionCircleOutlined))
                .on_press(Message::Help)
                .medium(),
            widget::button::icon(crate::app::icondata_svg(icondata::AiPlusOutlined))
                .on_press(Message::Add)
                .medium(),
        ];

        let left = iced::widget::column![
            widget::svg("logo_export.svg")
                .width(Length::Fill)
                .height(Length::Fill)
                .apply(widget::container)
                .align_bottom(Length::FillPortion(3))
                .center_x(Length::Fill)
                .padding(iced::Padding::default().top(30.0)),
            iced::widget::column![widget::text::title1("Cosmonaute"), action_bar]
                .width(Length::Fill)
                .height(Length::FillPortion(2))
        ]
        .padding(iced::Padding::from([0.0, 20.0]))
        .width(Length::Fill)
        .apply(widget::container)
        .center(Length::Fill);

        let right = iced::widget::column![
            widget::text::title1("Documentation"),
            widget::search_input("Search", &self.search).on_input(Message::SearchInput),
            widget::scrollable::vertical(widget::text("TODO: Docset list"))
                .width(Length::Fill)
                .height(Length::Fill)
        ];

        widget::row().push(left).push(right).into()
    }

    pub fn update(&mut self, message: Message) -> Task<cosmic::Action<Message>> {
        match message {
            Message::SearchInput(what) => {
                self.search = what;
            }

            Message::Add => {
                return cosmic::Task::stream(cosmic::iced_futures::stream::channel(
                    3,
                    |tx| async move {
                        if let Err(why) = crate::docset::import_docset(
                            tx,
                            crate::docset::DocSource {
                                protocol: docset::Protocol::File,
                                kind: docset::DocKind::RustCrate,
                                path: PathBuf::from(
                                    "/home/generic/git/generic/libcosmic/Cargo.toml",
                                ),
                            },
                        )
                        .await
                        {
                            println!("Import error: {}", why)
                        } else {
                            println!("Import successful");
                        }
                    },
                ))
                .map(cosmic::Action::App)
                .map(crate::app::map_action);
            }

            Message::DocsetMessage(msg) => match msg {
                docset::Message::CurrentProgress { completed, total } => {
                    println!("Progress: {}/{}", completed, total);
                }
                docset::Message::ImportComplete { success } => {
                    if success {
                        println!("Import completed successfully");
                    } else {
                        println!("Import failed");
                    }
                }
                docset::Message::CompilerMessage(compiler_msg) => {
                    println!("Compiler message: {:?}", compiler_msg);
                }
                docset::Message::ReadPackageMetadata => {
                    println!("Reading package metadata");
                }
            },

            _ => {}
        }
        Task::none()
    }
}
