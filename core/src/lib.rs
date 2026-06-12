mod note;
mod project;
pub mod sync;
mod timeline;
pub mod utils;

mod error;

pub use error::CoreError;
pub use note::error::NoteError;
pub use note::{
    NoteSummary, SearchHit, create_draft_note, delete_note, extract_wikilinks, list_backlinks,
    list_mentions, list_notes, read_note, read_note_by_filename, resolve_wikilink, search_notes,
    update_note,
};
pub use project::error::ProjectError;
pub use project::{
    ProjectActivitySummary, ProjectSummary, TaskSummary, complete_task, create_project,
    create_task, delete_task, get_project_activity_summary, list_active_tasks, list_done_tasks,
    list_projects, read_project, update_task,
};
pub use timeline::error::TimelineError;
pub use timeline::{
    TimelineSearchHit, list_timeline_dates, read_timeline, save_timeline_entry, search_timeline,
};
pub use utils::device::Context as DeviceContext;
pub use utils::frontmatter;
pub use utils::validated::{Filename, NoteFilename, Slug};
