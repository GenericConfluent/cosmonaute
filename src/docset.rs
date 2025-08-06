use cargo_metadata::TargetKind;
use cosmic::iced::futures::channel::mpsc;
use cosmic::iced_futures::futures::SinkExt;
use derive_more::{Error, From};
use std::path::PathBuf;
use tokio::{io::AsyncBufReadExt, process::Command};
use rustdoc_types::Id;

static DOCDIR: &'static str = "/home/generic/.cosmonaute/";

pub struct Library {
    path: PathBuf,
}

pub enum DocKind {
    RustCrate,
    TomlBook,
}

pub enum Protocol {
    File,
}

pub struct DocSource {
    pub protocol: Protocol,
    pub kind: DocKind,
    pub path: PathBuf,
}

impl DocSource {}

pub struct DocSetHandle {
    name: String,
    version: String,
    language: String,
}

pub struct Documentation {
    inner: rustdoc_types::Crate,
    handle: DocSetHandle,
}

pub struct Module {
    
}


impl Documentation {
    fn module_items<T: AsRef<str>>(&self, path: &[T]) -> Vec<Id> {
        todo!();
    }
    
    // PERF: This is really bad. We don't need to allocate strings to compare
    // case insensitive.
    fn search(&self, query: &str) -> Vec<Id> {
        let mut results = Vec::new();


        for (id, item) in self.inner.index {

        }
        
        ///
        for (id, summary) in self.inner.paths {

                if item.name.to_lowercase().contains(query.to_lowercase()) {
                
            }

        }
        
        results
    }
    
}

#[derive(Debug, Error, From)]
pub enum DocsetError {
    CommandFailed,
    ParseError,
    NetworkIssue,
    FailedToFindDocOutput,
    Utf8Error(std::str::Utf8Error),
    JoinError(tokio::task::JoinError),
    IoError(std::io::Error),
    JsonError(serde_json::Error),
}

impl std::fmt::Display for DocsetError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DocsetError::CommandFailed => write!(f, "Command failed"),
            DocsetError::ParseError => write!(f, "Parse error"),
            DocsetError::NetworkIssue => write!(f, "Network issue"),
            DocsetError::FailedToFindDocOutput => write!(f, "Failed to find documentation output"),
            DocsetError::Utf8Error(e) => write!(f, "UTF-8 error: {}", e),
            DocsetError::JoinError(e) => write!(f, "Join error: {}", e),
            DocsetError::IoError(e) => write!(f, "IO error: {}", e),
            DocsetError::JsonError(e) => write!(f, "JSON error: {}", e),
        }
    }
}

type Result<T> = std::result::Result<T, DocsetError>;

#[derive(Debug, Clone)]
pub enum Message {
    CurrentProgress { completed: usize, total: usize },
    CompilerMessage(cargo_metadata::CompilerMessage),
    ImportComplete { success: bool },
    ReadPackageMetadata,
}

pub struct DocumentationManager {
    /// Path to the documentation directory.
    store_path: PathBuf,
    /// Optional sender to allow for progress monitoring
    listener: Option<mpsc::Sender<Message>>,
}

pub async fn query_crate_features(path: impl AsRef<std::path::Path>) -> Vec<String> {
    todo!()
}

pub async fn crate_metadata(path: impl AsRef<std::path::Path>) -> Result<cargo_metadata::Metadata> {
    let mut cmd = Command::new("cargo");
    cmd.args(&[
        "metadata",
        "--format-version",
        "1",
        "--manifest-path",
        path.as_ref().to_str().unwrap(),
    ]);

    cmd.stdout(std::process::Stdio::piped());
    let output = cmd.output().await?;
    let json = str::from_utf8(&output.stdout).map_err(DocsetError::Utf8Error)?;

    if !output.status.success() {
        return Err(DocsetError::CommandFailed);
    }

    // NOTE: Could just expect on the utf8 conversion, but throwing is probably
    // better for the future when working with tools which may not be written in Rust.
    serde_json::from_str::<cargo_metadata::Metadata>(json).map_err(DocsetError::JsonError)
}

/// Fetch documentation set frm a source. This may be as simple as grabbing the file
/// from some URI and bundling it into a [`DocSetHandle`], or it may involve running
/// a command to generate the documentation.
///
/// Currently only supports rustdoc, but could be extended to support other tools.
pub async fn import_docset(
    mut tx: mpsc::Sender<Message>,
    source: DocSource,
) -> Result<DocSetHandle> {
    let metadata = tokio::spawn(async move { crate_metadata(&source.path).await });
    tx.send(Message::ReadPackageMetadata).await;

    let metadata = metadata.await??;
    dbg!("METADATA TASK JOIN");
    let total = metadata.packages.len();

    let mut cmd = Command::new("cargo");
    cmd.args(&[
        "rustdoc",
        // TODO: These need to be import options
        // "--all-features",
        // "--keep-going",
        "--manifest-path",
        metadata.root_package().unwrap().manifest_path.as_str(),
        "--message-format",
        "json",
        "-Z",
        "unstable-options",
        "--output-format",
        "json",
    ]);
    cmd.stdout(std::process::Stdio::piped());

    let mut child = cmd.spawn()?;
    let stdout = child.stdout.take().expect("Failed to capture stdout");
    let mut reader = tokio::io::BufReader::new(stdout).lines();

    let success = tokio::spawn(async move {
        let status = child.wait().await.expect("child process wasn't running");
        // cargo rustdoc returns 101 on failure.
        // Could write this line nicer but I don't know how
        // .success and .exit_ok() evaluate.
        status.code() == Some(0)
    });

    let mut completed = 0;
    // FIXME:
    // This is actually really screwed up. We don't account for feature flags
    //and others.
    tx.send(Message::CurrentProgress { completed, total }).await;

    // WARN: I assume cargo writes one json message per line.
    while let Some(line) = reader.next_line().await? {
        // If this is a message we need to observe the `reason` to figure out what kind it is.
        let Ok(message) = serde_json::from_str::<serde_json::Value>(&line) else {
            continue;
        };

        match &message {
            serde_json::Value::Object(object) => {
                match object.get("reason").and_then(serde_json::Value::as_str) {
                    // Maybe pass these along in the future
                    Some("compiler-message") => {
                        if let Ok(compiler_message) =
                            serde_json::from_value::<cargo_metadata::CompilerMessage>(message)
                        {
                            tx.send(Message::CompilerMessage(compiler_message)).await;
                        }
                    }
                    // We can count these to track progress
                    Some("compiler-artifact") => {
                        completed += 1;
                        tx.send(Message::CurrentProgress { completed, total }).await;
                    }
                    // Not much use for this. It only contains a success bool.
                    // We already have that from the exit code.
                    Some("build-finished") => {
                        break;
                    }
                    // This is basically useless
                    Some("build-script-executed") | _ => {}
                }
            }
            _ => {}
        }
    }

    if !success.await.map_err(DocsetError::JoinError)? {
        return Err(DocsetError::CommandFailed);
    } else {
        tx.send(Message::ImportComplete { success: true }).await;
    }

    // By this point there is a .json file in `target/doc/`

    // ERROR: The root package name is often but not always the name of the
    // target we want to document. I assume that cargo rustdoc will generate
    // single .json for whichever library target happens to show up first in
    // the root package.
    let Some(name) = metadata
        .root_package()
        .unwrap()
        .targets
        .iter()
        .find(|target| {
            target.doc
                && (target.kind.contains(&TargetKind::Lib)
                    || target.kind.contains(&TargetKind::DyLib)
                    || target.kind.contains(&TargetKind::ProcMacro))
        })
        .map(|target| &target.name)
    else {
        return Err(DocsetError::FailedToFindDocOutput);
    };

    let json_source = metadata
        .target_directory
        .join("doc")
        .join(format!("{name}.json"));

    let json_content = tokio::fs::read_to_string(&json_source).await?;
    let crate_docs: rustdoc_types::Crate = serde_json::from_str(&json_content)?;

    // This may be pretty big, probably want to clean it up before allocating
    // more for the more compact serailization.
    drop(json_content);
    drop(json_source);

    let bitcode_content = bitcode::serialize(&crate_docs).expect("Failed to serialize crate docs");

    // FIXME: We need to make sure cosmonaute is setup before trying to write
    let bitcode_destination = PathBuf::from(DOCDIR).join(format!("{name}.bitcode"));
    tokio::fs::write(&bitcode_destination, bitcode_content).await?;

    Ok(DocSetHandle {
        name: name.to_string(),
        version: crate_docs
            .crate_version
            .unwrap_or_else(|| "???".to_string()),
        language: "rust".to_string(),
    })
}
