// SPDX-FileCopyrightText: MailCraft contributors
// SPDX-License-Identifier: MIT OR Apache-2.0
//! Which message is selected after the selected one leaves the list (#1604): the one below, as
//! before, or, when the owner chose it in File › Options › Mail, the one above.

use mailcraft_model::{FolderKind, Id};
use mailcraft_time::{Date, DateTime};
use serde_json::{Value, json};

use crate::{AfterRemoval, Session, sample};

/// Wednesday 2026-10-14 16:00 UTC.
fn now() -> i64 {
    DateTime::new(Date::new(2026, 10, 14).unwrap(), 16, 0, 0).unwrap().to_timestamp(0)
}

fn run(s: &mut Session, id: &str, params: Value) -> Value {
    s.run(id, &params).unwrap_or_else(|e| panic!("{id} {params}: {e}"))
}

/// The sample's Inbox, and its rows from the top.
fn inbox() -> (Session, Vec<Id>) {
    let mut s = Session::at(sample::mailbox(now(), 0), now(), 0);
    run(&mut s, "folder.select", json!({"kind": "inbox"}));
    let rows: Vec<Id> = s.rows().iter().map(|r| r.id).collect();
    assert!(rows.len() >= 5, "the sample's Inbox has rows to walk");
    (s, rows)
}

#[test]
fn by_default_the_message_below_is_selected() {
    let (mut s, rows) = inbox();
    assert_eq!(s.prefs.after_removal, AfterRemoval::Below);
    run(&mut s, "mail.select", json!({"id": rows[2]}));
    run(&mut s, "mail.delete", json!({}));
    assert_eq!(s.sel.messages, [rows[3]]);
}

#[test]
fn with_above_chosen_delete_and_move_walk_up_the_list() {
    let (mut s, rows) = inbox();
    run(&mut s, "file.options", json!({"afterRemoval": "above"}));
    assert_eq!(s.prefs.after_removal, AfterRemoval::Above);
    run(&mut s, "mail.select", json!({"id": rows[3]}));
    run(&mut s, "mail.delete", json!({}));
    assert_eq!(s.sel.messages, [rows[2]], "Delete goes to the message above");
    let account = s.mailbox.accounts[0].id;
    let archive = s.mailbox.folder_of_kind(account, FolderKind::Archive).unwrap();
    run(&mut s, "mail.move", json!({"folder": archive}));
    assert_eq!(s.sel.messages, [rows[1]], "Move goes up too");
}

#[test]
fn with_above_chosen_several_messages_go_to_the_one_above_the_first() {
    let (mut s, rows) = inbox();
    run(&mut s, "file.options", json!({"afterRemoval": "above"}));
    run(&mut s, "mail.select", json!({"ids": [rows[2], rows[3], rows[4]]}));
    run(&mut s, "mail.delete", json!({}));
    assert_eq!(s.sel.messages, [rows[1]]);
}

#[test]
fn with_above_chosen_the_top_message_goes_to_the_new_top() {
    let (mut s, rows) = inbox();
    run(&mut s, "file.options", json!({"afterRemoval": "above"}));
    run(&mut s, "mail.select", json!({"id": rows[0]}));
    run(&mut s, "mail.delete", json!({}));
    assert_eq!(s.sel.messages, [rows[1]], "nothing above: the first row left");
}

#[test]
fn with_above_chosen_a_conversation_goes_to_the_row_above() {
    let (mut s, _) = inbox();
    run(&mut s, "view.conversations", json!({"value": true}));
    run(&mut s, "file.options", json!({"afterRemoval": "above"}));
    let rows: Vec<Id> = s.rows().iter().map(|r| r.id).collect();
    run(&mut s, "mail.select", json!({"id": rows[2]}));
    run(&mut s, "mail.delete", json!({}));
    assert_eq!(s.sel.messages, [rows[1]]);
}

#[test]
fn the_choice_is_kept_and_a_wrong_one_changes_nothing() {
    let (mut s, _) = inbox();
    run(&mut s, "file.options", json!({"afterRemoval": "above"}));
    assert!(s.run("file.options", &json!({"afterRemoval": "sideways"})).is_err());
    assert!(s.run("file.options", &json!({"afterRemoval": true})).is_err());
    assert_eq!(s.prefs.after_removal, AfterRemoval::Above);
    let saved = serde_json::to_value(&s.prefs).unwrap();
    assert_eq!(saved["afterRemoval"], "above");
    let back: crate::Prefs = serde_json::from_value(saved).unwrap();
    assert_eq!(back.after_removal, AfterRemoval::Above);
    // Settings saved before this was kept read as today's behaviour.
    let older: crate::Prefs = serde_json::from_value(json!({"markReadOnOpen": true})).unwrap();
    assert_eq!(older.after_removal, AfterRemoval::Below);
    run(&mut s, "file.options", json!({"afterRemoval": "below"}));
    assert_eq!(s.prefs.after_removal, AfterRemoval::Below);
}
