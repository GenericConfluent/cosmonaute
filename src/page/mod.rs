pub mod docs;
pub mod home;

pub enum Page {
    Home(home::ViewModel),
    Docs(docs::ViewModel),
}
