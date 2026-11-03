//! TM AutoClip yerel yaması — açılış kapısı (11 Eki 2026; canlı uyumu 3 Kas 2026). Oturum (`TMAC_OTURUM` +
//! `TMAC_BELIRTEC`) hiç verilmediyse program serbest açılır (canlıda TM AutoClip menüsünden düz açılış); oturum verildiyse
//! ve geçersizse ileti gösterip kapanır. Denetim
//! `filmcraft_ui_egui::tm::oturum` içinde (üç yamalı programda ortak); ayrıntı TM AutoClip deposunda
//! `harici/craft-yamalar/YAMALAR.md`.

/// Verilen oturum geçerli değilse kullanıcıya söyler ve süreci 3 koduyla kapatır; oturum yoksa bir şey yapmaz.
pub fn kapi() {
    use filmcraft_ui_egui::tm::{PROGRAM, oturum};
    if let Err(sebep) = oturum::kapi_karari(PROGRAM) {
        log::warn!("TM AutoClip oturumu reddedildi: {sebep}");
        let metin = format!("{}\n\n({sebep})", oturum::RET_ILETISI);
        eprintln!("filmcraft: {metin}");
        let _ = rfd::MessageDialog::new().set_level(rfd::MessageLevel::Warning).set_title("FilmCraft — TM AutoClip").set_description(&metin).show();
        std::process::exit(3);
    }
}

/// TM AutoClip yerel yaması — arayüz dili (4 Kas 2026): TM AutoClip Craft'ı açarken kendi arayüz dilini `TMAC_DIL` ortam
/// değişkeniyle verir ("tr", "en"…). Ayarlar ▸ Genel ▸ Arayüz dili "System Language" (varsayılan) iken bu dil sistemin dillerinin
/// önüne geçer; FilmCraft'ta o dil yoksa sistemin dilleri, o da yoksa İngilizce (upstream `Language::from_locales`). Kullanıcının
/// elle seçtiği dil korunur. Değişken yoksa ya da boşsa davranış upstream'le aynı.
pub fn sistem_dilleri(sistem: Vec<String>) -> Vec<String> {
    dilleri_sirala(std::env::var("TMAC_DIL").ok(), sistem)
}

fn dilleri_sirala(tm: Option<String>, sistem: Vec<String>) -> Vec<String> {
    match tm.map(|dil| dil.trim().to_owned()).filter(|dil| !dil.is_empty() && dil.len() <= 32 && dil.bytes().all(|c| c.is_ascii_alphanumeric() || c == b'-' || c == b'_')) {
        Some(dil) => std::iter::once(dil).chain(sistem).collect(),
        None => sistem,
    }
}

#[cfg(test)]
mod tests {
    use super::dilleri_sirala;

    #[test]
    fn tm_dili_sistem_dillerinin_onune_gecer() {
        let sistem = vec!["de-DE".to_owned()];
        assert_eq!(dilleri_sirala(Some("tr".to_owned()), sistem.clone()), vec!["tr".to_owned(), "de-DE".to_owned()]);
        assert_eq!(dilleri_sirala(None, sistem.clone()), sistem);
        assert_eq!(dilleri_sirala(Some("  ".to_owned()), sistem.clone()), sistem);
        assert_eq!(dilleri_sirala(Some("tr;rm -rf".to_owned()), sistem.clone()), sistem);
    }
}
