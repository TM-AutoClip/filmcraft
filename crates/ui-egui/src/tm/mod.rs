//! TM AutoClip yerel yaması — FilmCraft (11 Eki 2026). Upstream'e dokunuş: `lib.rs` → `pub mod tm;`, `app.rs` → her karede
//! `tm::kare`, `menus.rs` → menü öğeleri + çalıştırma, `panels/export_mode/preview.rs` → düğme, `apps/filmcraft/src/main.rs` → kapı.
//!
//! - Akış kipinde Dışa Aktar penceresinde "Bitir ve TM AutoClip'e gönder": geçerli dışa aktarma ayarlarıyla (görünür altyazı izleri
//!   her zaman yakılarak) oturumun beklenen çıktı yoluna (uzantı ayarın biçimine göre) dışa aktarır; iş bitince `teslim.json` yazar ve programı kapatır.
//!   Dışa aktarmadan ÖNCE proje kaydedilir (adım 6b, çift yön: TM AutoClip'in Düzenle'si projeyi okuyup kullanıcının burada yaptıklarını
//!   kilitler ve korur; kaydedilmezse düzenlemeler kurtarma günlüğünde kalıp TM AutoClip'e görünmezdi).
//! - Düz kipte düğme GİZLİ. `craft-marka.json` linkleri (yalnız https) Yardım menüsünde Discord öğesinin YANINA eklenir.

pub mod oturum;

use std::sync::Mutex;

use egui::{Align2, Color32, Rect, Sense, pos2, vec2};
use serde_json::{Value, json};

use crate::FilmcraftApp;
use crate::menus::MenuItem;
use crate::theme::Tokens;

/// Bu programın oturumdaki adı.
pub const PROGRAM: &str = "filmcraft";
/// Marka linki komut öneki: `tm.link.<sıra>`.
const LINK: &str = "tm.link.";

/// Süren "Bitir ve gönder" dışa aktarması: (iş kimliği, çıktı yolu).
static BEKLEYEN: Mutex<Option<(u64, String)>> = Mutex::new(None);

fn bekleyen() -> std::sync::MutexGuard<'static, Option<(u64, String)>> {
    BEKLEYEN.lock().unwrap_or_else(std::sync::PoisonError::into_inner)
}

/// Yardım menüsüne marka linkleri (Discord öğesinden sonra).
pub fn menu_ogeleri(items: &mut Vec<MenuItem>) {
    let Some(o) = oturum::simdiki() else { return };
    let Some(at) = items.iter().position(|i| i.id == "help.discord") else { return };
    let path = items[at].path.clone();
    for (k, l) in o.linkler.iter().enumerate() {
        items.insert(at + 1 + k, MenuItem { id: format!("{LINK}{k}"), label: l.etiket.clone(), path: path.clone(), shortcut: None, enabled: true, checked: None });
    }
}

/// Bizim komutlar (`None` = bizim değil).
pub fn calistir(app: &mut FilmcraftApp, ctx: &egui::Context, id: &str) -> Option<Result<Value, String>> {
    if let Some(sira) = id.strip_prefix(LINK) {
        let adres = oturum::simdiki().and_then(|o| sira.parse::<usize>().ok().and_then(|k| o.linkler.get(k))).map(|l| l.adres.clone());
        return Some(match adres {
            Some(a) => {
                crate::links::open(app, ctx, &a);
                Ok(json!({"url": a}))
            }
            None => Err("bilinmeyen link".into()),
        });
    }
    if id == "tm.gonder" {
        return Some(baslat(app));
    }
    None
}

/// Dışa Aktar penceresinde, "Send to Queue"nun soluna "Bitir ve TM AutoClip'e gönder" (yalnız akış kipinde).
pub fn disa_aktar_dugmesi(app: &mut FilmcraftApp, ui: &mut egui::Ui, kuyruk: Rect) {
    let Some(o) = oturum::akis() else { return };
    let t = app.tokens;
    let etiket = o.metin("gonder", "Bitir ve TM AutoClip'e gönder").to_string();
    let suruyor = bekleyen().is_some();
    let g = ui.painter().layout_no_wrap(etiket.clone(), Tokens::semibold(12.5), Color32::WHITE);
    let w = g.size().x + 28.0;
    let r = Rect::from_min_size(pos2(kuyruk.min.x - 10.0 - w, kuyruk.min.y), vec2(w, kuyruk.height()));
    let resp = ui.interact(r, egui::Id::new("export-tm-gonder"), Sense::click());
    let renk = if suruyor { t.text_dim } else if resp.hovered() { t.accent_hover } else { t.accent };
    ui.painter().rect_filled(r, 15.0, renk);
    ui.painter().text(r.center(), Align2::CENTER_CENTER, &etiket, Tokens::semibold(12.5), Color32::WHITE);
    app.auto.add("export.tmGonder", r, &etiket);
    if resp.clicked() && !suruyor {
        match baslat(app) {
            Ok(v) => app.ui.status = format!("TM AutoClip için dışa aktarılıyor: {}", v["path"].as_str().unwrap_or("")),
            Err(e) => app.ui.status = e,
        }
    }
}

/// Dışa aktarmayı oturumun çıktı yoluna başlatır.
fn baslat(app: &mut FilmcraftApp) -> Result<Value, String> {
    let o = oturum::akis().ok_or("TM AutoClip akış oturumu yok")?;
    if bekleyen().is_some() {
        return Err("TM AutoClip dışa aktarması zaten sürüyor".into());
    }
    let (_, yol) = o.cikti.clone().ok_or("oturumda çıktı yolu yok")?;
    // Adım 6b: önce proje kaydedilir (açılan proje dosyasına) — çift yön eşitleyici kullanıcının düzenlemelerini projeden okur.
    if let Err(e) = app.session.execute("file.save", json!({})) {
        log::error!("TM AutoClip: proje kaydedilemedi: {e}");
        return Err(format!("Proje kaydedilemedi: {e}"));
    }
    // Kapsayıcı ayardaki biçime göre: beklenen yolun adı + ayarın uzantısı (teslim.json gerçek yolu bildirir).
    let yol = yol.with_extension(app.ui.export.settings.extension());
    let mut p = crate::panels::export_mode::export_params(app);
    p["path"] = json!(yol.to_string_lossy());
    // Akışın altyazısı FilmCraft'ın altyazı izinde ("TM:altyazi"): gönderilen video yayına gider, altyazı videoda olmalı. Görünür
    // (gözü açık) altyazı izleri yakılır; kullanıcı izi gizlerse ya da silerse yakılmaz.
    p["burnCaptions"] = json!(true);
    let v = app.session.execute("file.exportMedia", p).map_err(|e| {
        log::error!("TM AutoClip: dışa aktarma başlamadı: {e}");
        e.to_string()
    })?;
    let is = v["job"].as_u64().ok_or("dışa aktarma işi kimliği dönmedi")?;
    let gercek = v["path"].as_str().map_or_else(|| yol.to_string_lossy().into_owned(), str::to_string);
    log::info!("TM AutoClip: dışa aktarma başladı (iş {is}, {gercek})");
    *bekleyen() = Some((is, gercek));
    Ok(v)
}

/// Her kare: süren dışa aktarma bittiyse teslim dosyasını yazar ve kapanır; hata verdiyse durumu bildirir.
pub fn kare(app: &mut FilmcraftApp, ctx: &egui::Context) {
    let Some((is, yol)) = bekleyen().clone() else { return };
    let Some(job) = app.session.jobs.iter().find(|j| j.id == is) else {
        log::warn!("TM AutoClip: dışa aktarma işi {is} listede yok");
        *bekleyen() = None;
        return;
    };
    let sonuc = job.result.lock().unwrap_or_else(std::sync::PoisonError::into_inner).clone();
    match sonuc {
        None => ctx.request_repaint_after(std::time::Duration::from_millis(250)),
        Some(Err(e)) => {
            *bekleyen() = None;
            log::error!("TM AutoClip: dışa aktarma başarısız: {e}");
            app.ui.status = format!("TM AutoClip'e gönderilemedi: {e}");
        }
        Some(Ok(_)) => {
            *bekleyen() = None;
            let Some(o) = oturum::akis() else { return };
            match oturum::teslim_yaz(o, std::path::Path::new(&yol)) {
                Ok(_) => {
                    app.ui.status = format!("TM AutoClip'e gönderildi: {yol}");
                    // Proje dışa aktarmadan önce kaydedildi; kaydet sorusu çıkmasın.
                    app.quit_confirmed = true;
                    ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                }
                Err(e) => {
                    log::error!("TM AutoClip: {e}");
                    app.ui.status = e;
                }
            }
        }
    }
}
