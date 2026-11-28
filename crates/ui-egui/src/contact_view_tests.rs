// SPDX-FileCopyrightText: MailCraft contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

use egui::{PointerButton, accesskit::Role};
use egui_kittest::kittest::Queryable;
use serde_json::json;

use crate::people_view::testing;

#[test]
fn contact_view_renders_list_and_chat_timeline_in_light_and_dark() {
    let mut w = testing::window("view.mail");
    w.state_mut().execute("ui.dismissReminder", json!({"all": true})).unwrap();
    w.state_mut().execute("view.contactConversations", json!({"value": true})).unwrap();
    w.run_steps(3);

    // Select Tomás Rivera (who has both incoming messages and an outgoing reply in the sample mailbox).
    w.state_mut().execute("mail.selectCorrespondent", json!({"key": "tomas@lumen.example"})).unwrap();
    w.run_steps(3);

    assert!(w.state().session.view.contact_conversations);
    assert_eq!(w.state().session.view.selected_correspondent.as_deref(), Some("tomas@lumen.example"));

    let row_id = crate::ribbon::hit_id("tomas@lumen.example");
    assert!(w.state().hit.rect("correspondentRow", row_id).is_some(), "correspondent row should be drawn");
    assert!(w.state().hit.rect("contactMerge", 0).is_some(), "merge button should be drawn in chat header");

    // Verify per-message Reply button works on a visible message and opens compose addressed to Tomás.
    let reply_pos = w
        .state()
        .session
        .mailbox
        .messages
        .values()
        .filter(|m| m.from.email.eq_ignore_ascii_case("tomas@lumen.example"))
        .max_by_key(|m| m.date)
        .and_then(|m| w.state().hit.rect("chatReply", m.id))
        .expect("chatReply button drawn for latest Tomás message")
        .center();
    testing::click(&mut w, reply_pos, PointerButton::Primary, 1);
    w.run_steps(3);
    assert!(w.state().session.compose.is_some());
    w.state_mut().execute("compose.close", json!({"discard": true})).unwrap();
    w.run_steps(2);

    // Switch to dark theme and render without panic.
    w.state_mut().execute("ui.theme", json!({"value": "dark"})).unwrap();
    w.run_steps(3);
    assert!(w.state().hit.rect("correspondentRow", row_id).is_some());
}

#[test]
fn merge_correspondents_dialog_combines_addresses() {
    let mut w = testing::window("view.mail");
    w.state_mut().execute("ui.dismissReminder", json!({"all": true})).unwrap();
    w.state_mut().execute("view.contactConversations", json!({"value": true})).unwrap();
    w.state_mut().execute("mail.selectCorrespondent", json!({"key": "tomas@lumen.example"})).unwrap();
    w.run_steps(2);

    let merge_pos = w.state().hit.rect("contactMerge", 0).expect("merge button").center();
    testing::click(&mut w, merge_pos, PointerButton::Primary, 1);
    w.run_steps(2);
    assert_eq!(testing::dialog(&w), Some("mergeCorrespondents"));
    if let Some(crate::dialogs::Dialog::MergeCorrespondents(dlg)) = &w.state().dialog {
        assert!(dlg.selected_candidate.is_none(), "opening MergeDialog must not preselect a candidate");
    } else {
        panic!("expected MergeCorrespondents dialog");
    }

    // Enter a target address into "Or add email address:" and click Merge.
    let addr_box = w.get_by_role_and_label(Role::TextInput, tl!("Or add email address:")).rect();
    testing::click(&mut w, addr_box.center(), PointerButton::Primary, 1);
    testing::text(&mut w, "maya@lumen.example");
    w.run_steps(2);
    w.get_by_label(tl!("Merge")).click();
    w.run_steps(3);

    assert_eq!(testing::dialog(&w), None, "Merge button should submit and close dialog");
    assert_eq!(w.state().session.mailbox.correspondent_aliases.len(), 1);
    assert_eq!(w.state().session.mailbox.correspondent_aliases[0].addresses, vec!["maya@lumen.example", "tomas@lumen.example"]);
    let merged_id = crate::ribbon::hit_id("maya@lumen.example");
    assert!(w.state().hit.rect("correspondentRow", merged_id).is_some());
}

#[test]
fn chat_bubbles_render_html_cid_images_cc_and_click_to_read() {
    use mailcraft_model::{Address, Attachment, Body, FolderKind, Message};

    let mut mb = mailcraft_engine::sample::mailbox(testing::now(), 0);
    let account = mb.accounts[0].id;
    let inbox = mb.folder_of_kind(account, FolderKind::Inbox).unwrap();

    // Minimal valid 1x1 PNG for testing inline `cid:` rendering in chat bubbles.
    const TINY_PNG: &[u8] = &[
        0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A, 0x00, 0x00, 0x00, 0x0D, 0x49, 0x48, 0x44, 0x52, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00,
        0x01, 0x08, 0x06, 0x00, 0x00, 0x00, 0x1F, 0x15, 0xC4, 0x89, 0x00, 0x00, 0x00, 0x0A, 0x49, 0x44, 0x41, 0x54, 0x78, 0x9C, 0x63, 0x00, 0x01,
        0x00, 0x00, 0x05, 0x00, 0x01, 0x0D, 0x0A, 0x2D, 0xB4, 0x00, 0x00, 0x00, 0x00, 0x49, 0x45, 0x4E, 0x44, 0xAE, 0x42, 0x60, 0x82,
    ];
    let mut inline_pic = Attachment::new("pixel.png", "image/png", TINY_PNG.to_vec());
    inline_pic.content_id = "hero-1".into();
    inline_pic.inline = true;

    let older_unread = mb
        .add_message(Message {
            folder: inbox,
            message_id: "alice-older-unread@partner.example".into(),
            from: Address::new("Alice Chen", "alice@partner.example"),
            to: vec![Address::new("Avery Quinn", "avery@lumen.example")],
            subject: "Quick check-in".into(),
            date: testing::now() - 3_600,
            received: testing::now() - 3_600,
            body: Body { text: "Are we still good for 3pm?".into(), html: None },
            read: false,
            ..Default::default()
        })
        .unwrap();

    let long_paras: String = (0..40).map(|i| format!("<p>Detailed release section line {i}: status review notes.</p>")).collect();
    let rich_id = mb
        .add_message(Message {
            folder: inbox,
            message_id: "rich-chat-test@lumen.example".into(),
            from: Address::new("Alice Chen", "alice@partner.example"),
            to: vec![Address::new("Avery Quinn", "avery@lumen.example")],
            cc: vec![Address::new("Tomás Rivera", "tomas@lumen.example"), Address::new("Priya Nair", "priya@lumen.example")],
            subject: "Design update with HTML and image".into(),
            date: testing::now(),
            received: testing::now(),
            body: Body {
                text: "Fallback plain text".into(),
                html: Some(format!("<h3>Mockup Preview</h3><p><img src=\"cid:hero-1\" width=\"48\" height=\"48\"></p>{long_paras}")),
            },
            attachments: vec![inline_pic],
            read: false,
            ..Default::default()
        })
        .unwrap();

    let mut w = testing::window_on(mb, "view.mail");
    w.state_mut().execute("ui.dismissReminder", json!({"all": true})).unwrap();
    w.state_mut().execute("view.contactConversations", json!({"value": true})).unwrap();
    w.run_steps(3);

    // Click Alice's row (Option B): latest unread becomes read, older unread stays unread.
    let alice_row = w.state().hit.rect("correspondentRow", crate::ribbon::hit_id("alice@partner.example")).expect("Alice row").center();
    testing::click(&mut w, alice_row, PointerButton::Primary, 1);
    w.run_steps(4);

    assert!(w.state().session.mailbox.message(rich_id).unwrap().read, "latest unread marked read on row click");
    assert!(!w.state().session.mailbox.message(older_unread).unwrap().read, "older unread stays unread under Option B");

    // HTML + inline image layout was built under Place::Chat(rich_id), and tall HTML is clamped below half-screen ceiling.
    let laid_h = crate::reading::view::laid_size(&w.ctx, crate::reading::Place::Chat(rich_id), rich_id).map(|s| s.y).unwrap_or(0.0);
    assert!(laid_h > 460.0, "expected tall HTML body > 460pt, got {laid_h}");
    let bubble_rect = w.state().hit.rect("chatBubble", rich_id).expect("rich bubble rect");
    assert!(bubble_rect.height() < laid_h, "tall HTML bubble should be collapsed, bubble_h={} laid_h={laid_h}", bubble_rect.height());
    assert!(
        crate::reading::view::has_picture_item(&w.ctx, crate::reading::Place::Chat(rich_id), rich_id),
        "CID inline image must resolve as a picture item"
    );

    // Verify `Cc:` line and fixture recipient are rendered directly in the chat pane.
    assert!(w.query_all_by_label_contains("Cc: ").any(|_| true), "Cc recipient line should be rendered");
    assert!(w.query_all_by_label_contains("Tomás Rivera").any(|_| true), "Cc fixture recipient should be rendered");
    assert!(w.state().hit.rect("contactMarkAllRead", 0).is_some(), "Mark All Read button shown when unread messages remain");

    // Clicking the older unread bubble marks it read too.
    let older_pos = w.state().hit.rect("chatBubble", older_unread).expect("older bubble").center();
    testing::click(&mut w, older_pos, PointerButton::Primary, 1);
    w.run_steps(3);
    assert!(w.state().session.mailbox.message(older_unread).unwrap().read, "clicking unread bubble marks it read");

    // Clicking envelope toggle marks it unread again and keeps it unread (not immediately re-read on same frame).
    let toggle_pos = w.state().hit.rect("chatToggleRead", older_unread).expect("toggle read button").center();
    testing::click(&mut w, toggle_pos, PointerButton::Primary, 1);
    w.run_steps(3);
    assert!(!w.state().session.mailbox.message(older_unread).unwrap().read, "envelope toggle leaves message unread");

    // Clicking the prominent Mark All Read button clears remaining unread count.
    let mark_all_pos = w.state().hit.rect("contactMarkAllRead", 0).expect("Mark All Read button").center();
    testing::click(&mut w, mark_all_pos, PointerButton::Primary, 1);
    w.run_steps(3);
    assert!(w.state().session.mailbox.message(older_unread).unwrap().read);
    assert!(w.state().hit.rect("contactMarkAllRead", 0).is_none());

    // Clicking blank space in the right-hand chat pane clears all bubble selection.
    assert!(!w.state().session.sel.messages.is_empty());
    let rich_rect = w.state().hit.rect("chatBubble", rich_id).expect("rich bubble rect");
    let blank_pos = egui::pos2(rich_rect.max.x + 24.0, rich_rect.center().y);
    testing::click(&mut w, blank_pos, PointerButton::Primary, 1);
    w.run_steps(2);
    assert!(w.state().session.sel.messages.is_empty(), "clicking blank area clears bubble selection");
}

#[test]
fn prefs_restore_contact_view_flags_and_merge_dialog_limits_remove_to_manual_groups() {
    let mut w = testing::window("view.mail");
    w.state_mut().execute("ui.dismissReminder", json!({"all": true})).unwrap();
    w.state_mut().execute("view.contactConversations", json!({"value": true, "allAccounts": true})).unwrap();
    w.state_mut().execute("mail.selectCorrespondent", json!({"key": "tomas@lumen.example"})).unwrap();
    let saved = w.state().prefs();

    let mut fresh = testing::window("view.mail");
    fresh.state_mut().apply_prefs(saved);
    assert!(fresh.state().session.view.contact_conversations);
    assert!(fresh.state().session.view.contact_all_accounts);
    assert!(!fresh.state().session.view.conversations);
    assert!(fresh.state().session.view.selected_correspondent.is_none());

    // Contact-only links without a manual CorrespondentGroup have empty manual_addresses.
    let dlg = crate::contact_view::MergeDialog::open(
        fresh.state(),
        "tomas@lumen.example",
        "Tomás",
        &["tomas@lumen.example".into(), "tomas.alt@lumen.example".into()],
    );
    assert!(dlg.manual_addresses.is_empty());

    // After manual merge, both addresses appear in manual_addresses.
    fresh.state_mut().execute("mail.mergeCorrespondents", json!({"addresses": ["tomas@lumen.example", "maya@lumen.example"]})).unwrap();
    let dlg2 = crate::contact_view::MergeDialog::open(
        fresh.state(),
        "maya@lumen.example",
        "Maya",
        &["maya@lumen.example".into(), "tomas@lumen.example".into()],
    );
    assert!(dlg2.manual_addresses.contains("tomas@lumen.example"));
    assert!(dlg2.manual_addresses.contains("maya@lumen.example"));
}

#[test]
fn unread_tab_keeps_selected_conversation_and_mailbox_replace_closes_merge_dialog() {
    use mailcraft_model::{Address, Body, FolderKind, Message};

    let mut mb = mailcraft_model::Mailbox::with_local_account("Avery", "avery@example.org");
    let inbox = mb.folder_of_kind(mb.accounts[0].id, FolderKind::Inbox).unwrap();
    let msg_id = mb
        .add_message(Message {
            folder: inbox,
            from: Address::new("Solo Unread", "solo@example.org"),
            to: vec![Address::new("Avery", "avery@example.org")],
            subject: "Only unread".into(),
            date: testing::now(),
            received: testing::now(),
            body: Body { text: "Please keep open".into(), html: None },
            read: false,
            ..Default::default()
        })
        .unwrap();

    let mut w = testing::window_on(mb, "view.mail");
    w.state_mut().execute("ui.dismissReminder", json!({"all": true})).unwrap();
    w.state_mut().execute("view.contactConversations", json!({"value": true})).unwrap();
    w.state_mut().execute("view.filter", json!({"value": "unread"})).unwrap();
    w.run_steps(3);

    // Clicking Solo's row marks its only unread message read, removing it from Unread list,
    // yet the conversation stays selected and visible in the reading pane.
    let solo_row = w.state().hit.rect("correspondentRow", crate::ribbon::hit_id("solo@example.org")).expect("Solo row").center();
    testing::click(&mut w, solo_row, PointerButton::Primary, 1);
    w.run_steps(4);
    assert!(w.state().session.mailbox.message(msg_id).unwrap().read);
    assert_eq!(w.state().session.view.selected_correspondent.as_deref(), Some("solo@example.org"));
    assert!(w.state().hit.rect("chatBubble", msg_id).is_some(), "reading pane stays open after marking last unread read");

    // Replacing the mailbox clears an open MergeCorrespondents dialog (M9).
    let merge_pos = w.state().hit.rect("contactMerge", 0).expect("merge button").center();
    testing::click(&mut w, merge_pos, PointerButton::Primary, 1);
    w.run_steps(2);
    assert_eq!(testing::dialog(&w), Some("mergeCorrespondents"));
    w.state_mut().session.replace_mailbox(mailcraft_model::Mailbox::with_local_account("Other", "other@example.org"));
    w.run_steps(2);
    assert_eq!(testing::dialog(&w), None, "mailbox replacement must close MergeCorrespondents dialog");
}

#[test]
fn global_search_reveals_older_matched_message_and_preview_renders_in_contact_mode() {
    use mailcraft_model::{Address, Attachment, Body, FolderKind, Message};
    use std::sync::mpsc;

    let mut mb = mailcraft_model::Mailbox::with_local_account("Avery", "avery@example.org");
    let inbox = mb.folder_of_kind(mb.accounts[0].id, FolderKind::Inbox).unwrap();

    // Create 55 messages so the oldest sits outside the default 50-message page.
    let oldest = mb
        .add_message(Message {
            folder: inbox,
            from: Address::new("Jordan Lee", "jordan@example.org"),
            to: vec![Address::new("Avery", "avery@example.org")],
            subject: "Ancient needle subject".into(),
            date: testing::now() - 10_000,
            received: testing::now() - 10_000,
            body: Body { text: "Very old matching email".into(), html: None },
            attachments: vec![Attachment::new("brief.pdf", "application/pdf", b"%PDF-1.4\n%%EOF".to_vec())],
            read: true,
            ..Default::default()
        })
        .unwrap();
    for i in 1..55_i64 {
        mb.add_message(Message {
            folder: inbox,
            from: Address::new("Jordan Lee", "jordan@example.org"),
            to: vec![Address::new("Avery", "avery@example.org")],
            subject: format!("Routine check-in {i}"),
            date: testing::now() - 10_000 + i * 60,
            received: testing::now() - 10_000 + i * 60,
            body: Body { text: format!("Regular note {i}"), html: None },
            read: true,
            ..Default::default()
        })
        .unwrap();
    }

    let mut w = testing::window_on(mb, "view.mail");
    let host_seen = std::sync::Arc::new(std::sync::Mutex::new(false));
    let seen_flag = host_seen.clone();
    w.state_mut().services.attachment_preview = Some(crate::attachment_preview::Host {
        pdf: true,
        images: false,
        render: Box::new(move |req| {
            *seen_flag.lock().unwrap() = true;
            let _ = req.load();
            let (tx, rx) = mpsc::channel();
            let _ = tx.send(Ok(crate::attachment_preview::Page { image: egui::ColorImage::filled([4, 4], egui::Color32::WHITE), pages: 1 }));
            rx
        }),
    });
    w.state_mut().ui.attachment_previews = true;
    w.state_mut().execute("ui.dismissReminder", json!({"all": true})).unwrap();
    w.state_mut().execute("view.contactConversations", json!({"value": true})).unwrap();

    // Global Mail search for "needle" surfaces the oldest matched message immediately in chat view (M7).
    w.state_mut().execute("view.search", json!({"query": "needle"})).unwrap();
    w.run_steps(4);
    assert!(w.state().hit.rect("chatBubble", oldest).is_some(), "oldest matching message must be revealed without loading 55+ history first");

    // Requesting attachment preview on that message renders the preview view in contact mode (M8).
    let viewport = w.ctx.viewport_id();
    crate::attachment_preview::open(w.state_mut(), viewport, oldest, 0).unwrap();
    w.run_steps(4);
    assert!(*host_seen.lock().unwrap());
    assert!(w.state().hit.rect("attachmentPreviewBack", oldest).is_some(), "attachment preview Back button must render in contact mode");

    // Global search by correspondent name ("Jordan") shows the normal full conversation without hiding latest messages.
    w.state_mut().execute("ui.showAttachmentMessage", json!({})).unwrap();
    w.state_mut().execute("view.search", json!({"query": "Jordan"})).unwrap();
    w.run_steps(4);
    assert!(w.state().hit.rect("chatBubble", oldest).is_none(), "name-only search shows newest 50 page as usual");
}

#[test]
fn offscreen_remote_images_not_fetched_and_long_history_cache_bounded() {
    use mailcraft_model::{Address, Attachment, Body, FolderKind, Message};
    use std::sync::{Arc, Mutex, mpsc};

    let mut mb = mailcraft_model::Mailbox::with_local_account("Avery", "avery@example.org");
    let inbox = mb.folder_of_kind(mb.accounts[0].id, FolderKind::Inbox).unwrap();
    let mut ids = Vec::new();
    for i in 0..150_i64 {
        let mut msg = Message {
            folder: inbox,
            from: Address::new("Pat", "pat@example.org"),
            to: vec![Address::new("Avery", "avery@example.org")],
            subject: format!("Msg {i}"),
            date: testing::now() - 8_000 + i * 60,
            received: testing::now() - 8_000 + i * 60,
            body: Body { text: format!("Line {i}"), html: Some(format!("<p>Item {i}</p><img src=\"https://img.example/{i}.png\">")) },
            read: true,
            ..Default::default()
        };
        if i % 10 == 0 {
            msg.attachments.push(Attachment::new("empty.txt", "text/plain", b"hi".to_vec()));
        }
        ids.push(mb.add_message(msg).unwrap());
    }

    let mut w = testing::window_on(mb, "view.mail");
    let fetched: Arc<Mutex<Vec<String>>> = Arc::new(Mutex::new(Vec::new()));
    let sink = fetched.clone();
    w.state_mut().services.pictures = Some(Box::new(move |url| {
        sink.lock().unwrap().push(url);
        let (_tx, rx) = mpsc::channel();
        rx
    }));
    w.state_mut().ui.download_pictures = true;
    w.state_mut().execute("ui.dismissReminder", json!({"all": true})).unwrap();
    w.state_mut().execute("view.contactConversations", json!({"value": true})).unwrap();
    w.state_mut().execute("mail.selectCorrespondent", json!({"key": "pat@example.org"})).unwrap();
    w.run_steps(3);
    w.state_mut().chat_pane.shown_limit = 150;
    w.state_mut().chat_pane.scroll_to_bottom = true;
    w.run_steps(6);

    // Off-screen early messages at the top of the 150-message list must NOT trigger remote fetches (M10).
    let urls = fetched.lock().unwrap().clone();
    assert!(!urls.iter().any(|u| u == "https://img.example/0.png"), "off-screen top bubble must not queue remote image fetch: {urls:?}");

    // Viewport virtualization keeps cached chat entries well below KEPT_IN_CHAT = 64 and stable across redraws (M12b).
    let begun_before = crate::reading::view::layouts_begun(&w.ctx);
    w.run_steps(5);
    let begun_after = crate::reading::view::layouts_begun(&w.ctx);
    assert_eq!(begun_before, begun_after, "redraws must not re-layout or thrash cache on 150-message conversation");
    assert!(crate::reading::view::kept_count(&w.ctx) <= 64);

    // Disabling automatic pictures immediately clears cached remote permission instead of keeping deferred remote layout (m7).
    w.state_mut().ui.download_pictures = false;
    w.run_steps(3);
    assert_eq!(crate::reading::remote::access(w.state(), w.state().session.mailbox.message(ids[149]).unwrap()), crate::reading::Access::Blocked);
}

#[test]
fn unmerge_dialog_preserves_contact_link_and_responsive_html_fits() {
    use mailcraft_model::{Address, Attachment, Body, Contact, FolderKind, Labeled, Message};

    let mut mb = mailcraft_model::Mailbox::with_local_account("Avery", "avery@example.org");
    let acc = mb.accounts[0].id;
    let inbox = mb.folder_of_kind(acc, FolderKind::Inbox).unwrap();
    mb.add_contact(Contact {
        first: "Cara".into(),
        emails: vec![Labeled::new("work", "cara@example.org"), Labeled::new("home", "cara.alt@example.org")],
        ..Default::default()
    });
    // Also create a manual group linking those same two addresses, plus a responsive HTML message & non-inline CID image.
    mb.merge_correspondents(&["cara@example.org".into(), "cara.alt@example.org".into()], Some("Cara Merged")).unwrap();
    let mut non_inline = Attachment::new("badge.png", "image/png", vec![0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A]);
    non_inline.content_id = "badge-1".into();
    non_inline.inline = false;
    let resp_id = mb
        .add_message(Message {
            folder: inbox,
            from: Address::new("Cara", "cara@example.org"),
            to: vec![Address::new("Avery", "avery@example.org")],
            subject: "Responsive media fit".into(),
            date: testing::now() - 60,
            received: testing::now() - 60,
            body: Body {
                text: String::new(),
                html: Some(
                    r#"<style>@media (min-width: 900px){ .body{ display:none; } }</style><div class="body"><p>Shown in bubble</p></div>"#.into(),
                ),
            },
            read: true,
            ..Default::default()
        })
        .unwrap();
    let cid_id = mb
        .add_message(Message {
            folder: inbox,
            from: Address::new("Cara", "cara@example.org"),
            to: vec![Address::new("Avery", "avery@example.org")],
            subject: "Non-inline CID image".into(),
            date: testing::now(),
            received: testing::now(),
            body: Body { text: String::new(), html: Some(r#"<p>Inline badge:</p><img src="cid:badge-1" width="16" height="16">"#.into()) },
            attachments: vec![non_inline],
            read: true,
            ..Default::default()
        })
        .unwrap();

    let mut w = testing::window_on(mb, "view.mail");
    w.state_mut().execute("ui.dismissReminder", json!({"all": true})).unwrap();
    w.state_mut().execute("view.contactConversations", json!({"value": true})).unwrap();
    w.run_steps(4);

    // M11: Responsive body hidden at >=900px fits to bubble width and still shows.
    let doc = crate::reading::view::current_doc(&w.ctx, crate::reading::Place::Chat(resp_id), resp_id).expect("fitted doc");
    assert!(!doc.blocks.is_empty(), "responsive body must fit to bubble width before emptiness check");
    // m6: `note_drawn` marks `badge.png` drawn on the first chat render frame so it isn't duplicated as a chip.
    assert_eq!(w.state().session.mailbox.message(cid_id).unwrap().listed_attachments().count(), 0);

    // M12a: Empty-body chat entries (e.g. attachment-only messages) expire cleanly rather than pinning >64 entries.
    for idx in 1..=70_u64 {
        let empty_msg = std::sync::Arc::new(Message {
            id: 10_000 + idx,
            folder: inbox,
            from: Address::new("Cara", "cara@example.org"),
            subject: format!("Attachment only {idx}"),
            body: Body { text: String::new(), html: None },
            attachments: vec![Attachment::new("doc.txt", "text/plain", b"x".to_vec())],
            ..Default::default()
        });
        let _ = crate::reading::view::doc_fitted(&w.ctx, &empty_msg, crate::reading::Place::Chat(empty_msg.id), false, 360.0, false);
        w.step();
    }
    assert!(crate::reading::view::kept_count(&w.ctx) <= 64, "empty-body chat entries must expire cleanly");

    // m4: Removing manual link when a Contact card also links the two addresses keeps Contact-linked pair in dialog.
    let merge_pos = w.state().hit.rect("contactMerge", 0).expect("merge button").center();
    testing::click(&mut w, merge_pos, PointerButton::Primary, 1);
    w.run_steps(2);
    assert_eq!(testing::dialog(&w), Some("mergeCorrespondents"));
    w.get_all_by_label(tl!("Remove")).next().expect("Remove button").click();
    w.run_steps(2);
    if let Some(crate::dialogs::Dialog::MergeCorrespondents(dlg)) = &w.state().dialog {
        assert_eq!(dlg.existing_addresses, vec!["cara.alt@example.org", "cara@example.org"]);
        assert!(!dlg.error.is_empty(), "should explain that Contact card still links the address");
    } else {
        panic!("expected MergeCorrespondents dialog to stay open");
    }
}

#[test]
fn unread_and_focused_chat_bubbles_use_distinct_colors() {
    for t in [crate::theme::Tokens::light(), crate::theme::Tokens::dark()] {
        let (read_fill, read_stroke) = crate::chat_view::bubble_style(false, false, false, None, None, &t);
        let (unread_fill, unread_stroke) = crate::chat_view::bubble_style(false, true, false, None, None, &t);
        let (focus_fill, focus_stroke) = crate::chat_view::bubble_style(false, false, true, None, None, &t);

        assert_ne!(unread_fill, focus_fill, "unread and focused bubble fills must differ");
        assert_ne!(unread_stroke.color, focus_stroke.color, "unread and focused borders must differ");
        assert_ne!(unread_fill, read_fill, "unread bubble fill must differ from plain read fill");
        assert_ne!(focus_fill, read_fill, "focused bubble fill must differ from plain read fill");
        assert_ne!(unread_stroke.color, read_stroke.color, "unread border must differ from plain read border");
        assert_ne!(focus_stroke.color, read_stroke.color, "focused border must differ from plain read border");

        // Custom colours replace the theme's own, for unread and for selected alike.
        let green = egui::Color32::from_rgb(0x10, 0xC0, 0x40);
        let purple = egui::Color32::from_rgb(0x90, 0x30, 0xD0);
        let (_, unread_custom) = crate::chat_view::bubble_style(false, true, false, Some(green), Some(purple), &t);
        let (_, focus_custom) = crate::chat_view::bubble_style(false, false, true, Some(green), Some(purple), &t);
        assert_eq!(unread_custom.color, green);
        assert_eq!(focus_custom.color, purple);
        // A message that is both keeps the selection's look.
        let (_, both) = crate::chat_view::bubble_style(false, true, true, Some(green), Some(purple), &t);
        assert_eq!(both.color, purple);
    }
}

#[test]
fn chat_bubble_colors_are_configurable_in_options_and_persisted() {
    let mut w = testing::window("view.mail");
    w.state_mut().execute("ui.dismissReminder", json!({"all": true})).unwrap();

    // The command sets, reads back and clears each colour.
    assert_eq!(w.state().ui.chat_unread_color, None);
    let set = w.state_mut().execute("ui.chatColors", json!({"unread": "#10C040", "selected": [144, 48, 208]})).unwrap();
    assert_eq!(set["unread"], "#10C040");
    assert_eq!(set["selected"], "#9030D0");
    assert_eq!(w.state().ui.chat_unread_color, Some(egui::Color32::from_rgb(0x10, 0xC0, 0x40)));
    assert_eq!(w.state().ui.chat_selected_color, Some(egui::Color32::from_rgb(0x90, 0x30, 0xD0)));

    // A value that doesn't read changes nothing at all, not even the other colour.
    assert!(w.state_mut().execute("ui.chatColors", json!({"unread": "#FFFFFF", "selected": "nonsense"})).is_err());
    assert_eq!(w.state().ui.chat_unread_color, Some(egui::Color32::from_rgb(0x10, 0xC0, 0x40)));

    // It is kept with the window's settings and read back.
    let saved = serde_json::to_value(&w.state().ui).unwrap();
    let back: crate::UiState = serde_json::from_value(saved).unwrap();
    assert_eq!(back.chat_unread_color, w.state().ui.chat_unread_color);
    assert_eq!(back.chat_selected_color, w.state().ui.chat_selected_color);

    // null clears one; reset clears both.
    w.state_mut().execute("ui.chatColors", json!({"unread": null})).unwrap();
    assert_eq!(w.state().ui.chat_unread_color, None);
    assert!(w.state().ui.chat_selected_color.is_some());
    w.state_mut().execute("ui.chatColors", json!({"unread": "#112233", "reset": true})).unwrap();
    assert_eq!(w.state().ui.chat_selected_color, None);
    assert_eq!(w.state().ui.chat_unread_color, Some(egui::Color32::from_rgb(0x11, 0x22, 0x33)));
    w.state_mut().execute("ui.chatColors", json!({"reset": true})).unwrap();
    assert_eq!(w.state().ui.chat_unread_color, None);

    // File › Options › Mail has a picker for each; a colour typed in its hex field applies at once.
    w.state_mut().execute("ui.backstage", json!({"value": true, "page": "options"})).unwrap();
    w.run_steps(5);
    w.get_by_role_and_label(Role::Button, tl!("Unread bubble colour")).click();
    w.run_steps(4);
    w.get_by_role_and_label(Role::TextInput, tl!("Unread bubble colour as hex")).set_text_value_accesskit("#00FF88");
    w.run_steps(4);
    assert_eq!(w.state().ui.chat_unread_color, Some(egui::Color32::from_rgb(0x00, 0xFF, 0x88)));
    assert_eq!(w.state().ui.chat_selected_color, None);
    testing::key(&mut w, egui::Key::Escape);
    w.run_steps(3);

    // Reset gives the theme's own colour back.
    w.get_by_label(tl!("Reset")).click();
    w.run_steps(3);
    assert_eq!(w.state().ui.chat_unread_color, None);
}
