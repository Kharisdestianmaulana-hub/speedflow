use uuid::Uuid;

#[derive(Debug, Clone)]
pub enum AppCommand {
    StartDownload { url: String, destination_path: String },
    PauseTask(Uuid),
    ResumeTask(Uuid),
    CancelTask(Uuid),
    OpenFile(Uuid),
    RevealInFolder(Uuid),
}
