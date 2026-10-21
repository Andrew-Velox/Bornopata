//! The word being typed, shown inline in the document as a TSF composition (the counterpart of Fcitx5's client preedit).
//! Only the plain text of the Bangla word goes there; the candidates are drawn by the Server. Every write happens in an
//! asynchronous read and write edit session (see `edit::update` for why).

mod shared;
mod sink;

use std::mem::ManuallyDrop;
use std::rc::Rc;

use windows::Win32::UI::TextServices::{
    INSERT_TEXT_AT_SELECTION_FLAGS, ITfComposition, ITfCompositionSink, ITfContext,
    ITfContextComposition, ITfInsertAtSelection, ITfRange, TF_AE_END, TF_ANCHOR_END,
    TF_IAS_QUERYONLY, TF_SELECTION, TF_SELECTIONSTYLE,
};
use windows::core::{Interface, Result};

pub(crate) use self::shared::Shared;
use self::sink::CompositionSink;
use super::edit::{anchor_rect, caret_rect};
use super::service::SharedClient;

/// Runs inside an edit session (holding the write lock `ec`): first settles `commit`, then starts, changes or ends the
/// composition so that it shows `preedit`, and last tells the Server where the cursor is.
pub(crate) fn apply(
    shared: &Rc<Shared>,
    engine: &SharedClient,
    context: &ITfContext,
    ec: u32,
    commit: Option<&str>,
    preedit: &str,
) -> Result<()> {
    if let Some(text) = commit {
        commit_text(shared, context, ec, text)?;
    }
    if preedit.is_empty() {
        end_composition(shared, ec)?;
    } else {
        update_preedit(shared, context, ec, preedit)?;
    }
    report_cursor(shared, engine, context, ec);
    Ok(())
}

/// While a word is being typed the cursor is reported, so the Server's window follows it. When the word is over the
/// Server hides the window by itself, because its reply to the key had an empty frame.
fn report_cursor(shared: &Shared, engine: &SharedClient, context: &ITfContext, ec: u32) {
    let rect = match shared.composition() {
        Some(composition) => {
            let Ok(range) = (unsafe { composition.GetRange() }) else {
                return;
            };
            anchor_rect(context, ec, &range)
        }
        // No composition range but a word being typed (the application ended it): measure the insertion point.
        None if shared.composing() => caret_rect(context, ec),
        None => return,
    };
    // The client being borrowed elsewhere is rare; skipping one report keeps the window where it was.
    if let Ok(mut guard) = engine.try_borrow_mut()
        && let Some(client) = guard.as_mut()
        && let Err(error) = client.cursor(rect)
    {
        super::log::log(&format!("could not report the cursor: {error}"));
    }
}

/// With a composition: replaces its text with `text` and ends it. Without: inserts `text` at the selection.
fn commit_text(shared: &Shared, context: &ITfContext, ec: u32, text: &str) -> Result<()> {
    let utf16: Vec<u16> = text.encode_utf16().collect();
    match shared.composition() {
        Some(composition) => {
            let range = unsafe { composition.GetRange()? };
            unsafe { range.SetText(ec, 0, &utf16)? };
            move_selection_to_end(context, ec, &range)?;
            unsafe { composition.EndComposition(ec)? };
            shared.set_composition(None);
        }
        None => {
            let insert: ITfInsertAtSelection = context.cast()?;
            // Not NOQUERY: it returns no range, and windows-rs takes the NULL for a failure.
            let range = unsafe {
                insert.InsertTextAtSelection(ec, INSERT_TEXT_AT_SELECTION_FLAGS(0), &utf16)?
            };
            // Without moving the cursor the next insertion would land in the same place and the text would pile up from
            // right to left.
            move_selection_to_end(context, ec, &range)?;
        }
    }
    Ok(())
}

fn update_preedit(shared: &Rc<Shared>, context: &ITfContext, ec: u32, preedit: &str) -> Result<()> {
    let composition = match shared.composition() {
        Some(composition) => composition,
        None => start_composition(shared, context, ec)?,
    };
    let utf16: Vec<u16> = preedit.encode_utf16().collect();
    let range = unsafe { composition.GetRange()? };
    unsafe { range.SetText(ec, 0, &utf16)? };
    super::display_attribute::mark(context, ec, &range);
    move_selection_to_end(context, ec, &range)
}

/// Starts an empty composition at the selection; the framework keeps the sink.
fn start_composition(shared: &Rc<Shared>, context: &ITfContext, ec: u32) -> Result<ITfComposition> {
    let insert: ITfInsertAtSelection = context.cast()?;
    let range = unsafe { insert.InsertTextAtSelection(ec, TF_IAS_QUERYONLY, &[])? };
    let context_composition: ITfContextComposition = context.cast()?;
    let sink: ITfCompositionSink = CompositionSink::new(shared.clone()).into();
    let composition = unsafe { context_composition.StartComposition(ec, &range, &sink)? };
    shared.set_composition(Some(composition.clone()));
    Ok(composition)
}

/// Clears the composition's text before ending it, so nothing of the word stays behind.
fn end_composition(shared: &Shared, ec: u32) -> Result<()> {
    if let Some(composition) = shared.take_composition() {
        let range = unsafe { composition.GetRange()? };
        unsafe { range.SetText(ec, 0, &[])? };
        unsafe { composition.EndComposition(ec)? };
    }
    Ok(())
}

fn move_selection_to_end(context: &ITfContext, ec: u32, range: &ITfRange) -> Result<()> {
    let end = unsafe { range.Clone()? };
    unsafe { end.Collapse(ec, TF_ANCHOR_END)? };
    let selection = TF_SELECTION {
        range: ManuallyDrop::new(Some(end)),
        style: TF_SELECTIONSTYLE {
            ase: TF_AE_END,
            fInterimChar: false.into(),
        },
    };
    // `SetSelection` does not take ownership of the range; it is released here.
    let result = unsafe { context.SetSelection(ec, std::slice::from_ref(&selection)) };
    drop(ManuallyDrop::into_inner(selection.range));
    result
}
