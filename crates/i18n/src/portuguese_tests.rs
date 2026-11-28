// SPDX-FileCopyrightText: MailCraft contributors
// SPDX-License-Identifier: MIT OR Apache-2.0
//! Brazilian and European Portuguese: their names in the picker, their locales, plural rule, and
//! date conventions.
use crate::Lang;
use crate::date::{Day, long, numeric, time};
use crate::plural::Rule;

#[test]
fn brazilian_portuguese_is_offered_by_its_own_name_and_chosen_for_its_locales() {
    if cfg!(target_arch = "wasm32") {
        return;
    }
    assert_eq!(Lang::PortugueseBrazil.code(), "pt_BR");
    assert!(Lang::available().contains(&Lang::PortugueseBrazil));
    assert_eq!(Lang::PortugueseBrazil.picker_name(), "Português do Brasil (pré-visualização)");
    assert_eq!(Lang::from_pref("auto", Some("pt_BR.UTF-8")), Lang::PortugueseBrazil);
    assert_eq!(Lang::from_pref("auto", Some("pt-BR")), Lang::PortugueseBrazil);
    assert_eq!(Lang::from_pref("pt_BR", None), Lang::PortugueseBrazil);
    // A bare `pt` is the Brazilian catalogue, as it was before European Portuguese existed.
    assert_eq!(Lang::from_pref("pt", None), Lang::PortugueseBrazil);
    assert_eq!(Lang::from_code("pt-AO"), Some(Lang::PortugueseBrazil));
    assert_eq!(Lang::from_code("pt-BR-x-pt"), Some(Lang::PortugueseBrazil));
}

#[test]
fn european_portuguese_is_offered_by_its_own_name_and_chosen_for_portugal() {
    if cfg!(target_arch = "wasm32") {
        return;
    }
    assert_eq!(Lang::PortuguesePortugal.code(), "pt_PT");
    assert!(Lang::available().contains(&Lang::PortuguesePortugal));
    assert_eq!(Lang::PortuguesePortugal.picker_name(), "Português europeu (pré-visualização)");
    assert_eq!(Lang::from_pref("auto", Some("pt_PT.UTF-8")), Lang::PortuguesePortugal);
    assert_eq!(Lang::from_pref("auto", Some("pt-PT")), Lang::PortuguesePortugal);
    assert_eq!(Lang::from_pref("pt-PT", None), Lang::PortuguesePortugal);
    assert_eq!(Lang::from_code("pt_PT"), Some(Lang::PortuguesePortugal));
    assert_eq!(Lang::from_code("pt-Latn-PT"), Some(Lang::PortuguesePortugal));
}

#[test]
fn portuguese_has_two_plural_forms() {
    let br = Rule::from_header("nplurals=2; plural=(n > 1);");
    assert_eq!(br, Rule::AboveOne);
    assert_eq!((br.index(0), br.index(1), br.index(2), br.forms()), (0, 0, 1, 2));
    assert_eq!(Lang::PortugueseBrazil.tn(0, "{n} message", "{n} messages"), "{n} mensagem");
    assert_eq!(Lang::PortugueseBrazil.tn(1, "{n} message", "{n} messages"), "{n} mensagem");
    assert_eq!(Lang::PortugueseBrazil.tn(2, "{n} message", "{n} messages"), "{n} mensagens");

    let pt = Rule::from_header("nplurals=2; plural=(n != 1);");
    assert_eq!(pt, Rule::NotOne);
    assert_eq!((pt.index(0), pt.index(1), pt.index(2), pt.forms()), (1, 0, 1, 2));
    assert_eq!(Lang::PortuguesePortugal.tn(0, "{n} message", "{n} messages"), "{n} mensagens");
    assert_eq!(Lang::PortuguesePortugal.tn(1, "{n} message", "{n} messages"), "{n} mensagem");
    assert_eq!(Lang::PortuguesePortugal.tn(2, "{n} message", "{n} messages"), "{n} mensagens");
}

#[test]
fn brazilian_portuguese_writes_dates_its_own_way() {
    const SUN: Day = Day { year: 2026, month: 10, day: 11, weekday: 6 };
    const OLD: Day = Day { year: 2025, month: 10, day: 3, weekday: 4 };
    let c = Lang::PortugueseBrazil.conventions();
    assert_eq!((c.twelve_hour, c.week_start, c.pad_numeric, c.day_first), (false, 6, true, true));
    let lang = Lang::PortugueseBrazil;
    assert_eq!(
        (long(&lang, SUN), numeric(&lang, OLD), time(&lang, 21, 5, false)),
        ("domingo, 11 de outubro de 2026".into(), "03/10/2025".into(), "21:05".into())
    );
}

#[test]
fn european_portuguese_starts_the_week_on_monday() {
    const SUN: Day = Day { year: 2026, month: 10, day: 11, weekday: 6 };
    const OLD: Day = Day { year: 2025, month: 10, day: 3, weekday: 4 };
    let c = Lang::PortuguesePortugal.conventions();
    assert_eq!((c.twelve_hour, c.week_start, c.pad_numeric, c.day_first), (false, 0, true, true));
    let lang = Lang::PortuguesePortugal;
    assert_eq!(
        (long(&lang, SUN), numeric(&lang, OLD), time(&lang, 21, 5, false)),
        ("domingo, 11 de outubro de 2026".into(), "03/10/2025".into(), "21:05".into())
    );
}
