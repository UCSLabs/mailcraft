// SPDX-FileCopyrightText: MailCraft contributors
// SPDX-License-Identifier: MIT OR Apache-2.0
//! Norwegian Bokmal: its name in the picker, its locale, plural rule, and date conventions.
use crate::Lang;
use crate::date::{Day, long, numeric, time};
use crate::plural::Rule;

#[test]
fn norwegian_is_offered_by_its_own_name_and_chosen_for_its_locales() {
    if cfg!(target_arch = "wasm32") {
        return;
    }
    assert!(Lang::available().contains(&Lang::Norwegian));
    assert_eq!(Lang::Norwegian.picker_name(), "Norsk bokmål (forhåndsvisning)");
    // `no` is the macrolanguage and `nb` the subtag Windows, macOS and browsers send.
    assert_eq!(Lang::from_pref("auto", Some("nb_NO.UTF-8")), Lang::Norwegian);
    assert_eq!(Lang::from_pref("auto", Some("no_NO.UTF-8")), Lang::Norwegian);
    assert_eq!(Lang::from_pref("auto", Some("nb-NO")), Lang::Norwegian);
    assert_eq!(Lang::from_pref("no", None), Lang::Norwegian);
    assert_eq!(Lang::from_pref("nb", None), Lang::Norwegian);
    // Nynorsk is a written language of its own: it is not served Bokmal.
    assert_eq!(Lang::from_pref("auto", Some("nn_NO.UTF-8")), Lang::English);
    assert_eq!(Lang::from_pref("nn", None), Lang::English);
}

#[test]
fn norwegian_has_englishs_two_plural_forms() {
    let no = Rule::from_header("nplurals=2; plural=(n != 1);");
    let forms: Vec<usize> = [1, 0, 2, 3, 11, 21, 100].iter().map(|n| no.index(*n)).collect();
    assert_eq!(forms, [0, 1, 1, 1, 1, 1, 1]);
    assert_eq!(no.forms(), 2);
    assert_eq!(Lang::Norwegian.tn(1, "{n} message", "{n} messages"), "{n} melding");
    assert_eq!(Lang::Norwegian.tn(3, "{n} message", "{n} messages"), "{n} meldinger");
}

#[test]
fn norwegian_writes_dates_its_own_way() {
    const SUN: Day = Day { year: 2026, month: 10, day: 11, weekday: 6 };
    const OLD: Day = Day { year: 2025, month: 10, day: 3, weekday: 4 };
    let c = Lang::Norwegian.conventions();
    assert_eq!((c.twelve_hour, c.week_start, c.pad_numeric, c.day_first), (false, 0, true, true));
    let lang = Lang::Norwegian;
    assert_eq!(
        (long(&lang, SUN), numeric(&lang, OLD), time(&lang, 21, 5, false)),
        ("søndag, 11. oktober 2026".into(), "03.10.2025".into(), "21:05".into())
    );
}
