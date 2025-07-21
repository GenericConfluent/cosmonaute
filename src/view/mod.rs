pub mod home;
pub mod docs;

pub enum View {
    Home(home::ViewModel),
    Docs(docs::ViewModel),
}
