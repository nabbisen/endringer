use gix::ObjectId;

pub mod repository;
pub mod types;
mod util;

/// convert commit id to short string id
pub fn commit_id_to_short_id(commit_id: ObjectId) -> String {
    util::commit_id_to_short_id(commit_id)
}
