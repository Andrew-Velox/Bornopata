//! Edit sessions: TSF does not let a text service change a document directly; it asks for an edit session and gets
//! an edit cookie in the callback. Writing the text and the word being typed is in [`update`], where the cursor is on
//! the screen is in [`anchor`].

mod anchor;
mod update;

pub(crate) use self::anchor::{anchor_rect, caret_rect};
pub(crate) use self::update::request_update;
