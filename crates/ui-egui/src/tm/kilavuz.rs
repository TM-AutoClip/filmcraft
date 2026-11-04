//! TM AutoClip yerel yaması — program monitöründe DÜZENLE AYARLARININ KILAVUZU (5 Kas 2026; geliştirici: "FilmCraft'ta filigran yeri: yamayla
//! kılavuz çiz (yalnız görüntüleyicide)"). FilmCraft'ta render'a girmeyen kılavuz katman yok: filigranın yeri program monitörünün üstüne kesik
//! çizgili dikdörtgen + adıyla ÇİZİLİR — projeye, sekansa ve dışa aktarmaya hiçbir şey yazılmaz (filigranı TM AutoClip dönüşte uygular).
//!
//! Veri: akış oturumunun iş klasöründe `tm-kilavuz.json` (TM AutoClip yazar; .NET `FilmCraftGuideFile`):
//! `{"surum":1,"kutular":[{"ad":"Filigran","sutun":-1,"satir":1,"bosX":3,"bosY":3,"genislik":20,"yukseklik":6,"renk":"#3E9CF0"}]}` —
//! ölçüler karenin KISA kenarının yüzdesi (TM AutoClip'in filigran kuralı: çapa sütunu/satırı -1 · 0 · 1 + boşluk + boyut). Dosya yoksa ya da
//! bozuksa hiçbir şey çizilmez (sebep günlükte). Düz kipte (oturumsuz) okunmaz.

use std::sync::OnceLock;

use egui::{Align2, Color32, Rect, Stroke, StrokeKind, pos2, vec2};
use serde_json::Value;

use super::oturum;
use crate::theme::Tokens;

/// İş klasöründeki kılavuz dosyası.
pub const DOSYA: &str = "tm-kilavuz.json";

/// Kılavuz kutusu (ölçüler karenin kısa kenarının yüzdesi).
#[derive(Clone, Debug, PartialEq)]
pub struct Kutu {
    pub ad: String,
    /// Çapanın sütunu: -1 sol · 0 orta · 1 sağ.
    pub sutun: i32,
    /// Çapanın satırı: -1 üst · 0 orta · 1 alt.
    pub satir: i32,
    pub bos_x: f32,
    pub bos_y: f32,
    pub genislik: f32,
    pub yukseklik: f32,
    pub renk: Color32,
}

static KUTULAR: OnceLock<Vec<Kutu>> = OnceLock::new();

/// Oturumun kılavuz kutuları (ilk çağrıda iş klasöründen okunur; akış oturumu yoksa boş).
fn kutular() -> &'static [Kutu] {
    KUTULAR.get_or_init(|| {
        let Some(klasor) = oturum::akis().and_then(|o| o.is_klasoru.clone()) else { return Vec::new() };
        let yol = klasor.join(DOSYA);
        match std::fs::read_to_string(&yol) {
            Ok(metin) => oku(&metin),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Vec::new(),
            Err(e) => {
                log::warn!("TM AutoClip: kılavuz dosyası okunamadı: {e}");
                Vec::new()
            }
        }
    })
}

/// Kılavuz dosyasını okur (saf; birim testli). Biçimsiz dosya ya da kutu atlanır.
pub fn oku(metin: &str) -> Vec<Kutu> {
    let v: Value = match serde_json::from_str(metin) {
        Ok(v) => v,
        Err(e) => {
            log::warn!("TM AutoClip: kılavuz dosyası bozuk: {e}");
            return Vec::new();
        }
    };
    if v.get("surum").and_then(Value::as_i64) != Some(1) {
        log::warn!("TM AutoClip: kılavuz dosyasının biçimi tanınmıyor");
        return Vec::new();
    }
    let sayi = |k: &Value, ad: &str| k.get(ad).and_then(Value::as_f64).filter(|n| n.is_finite()).map(|n| n as f32);
    v.get("kutular")
        .and_then(Value::as_array)
        .map(|a| {
            a.iter()
                .filter_map(|k| {
                    let (genislik, yukseklik) = (sayi(k, "genislik")?, sayi(k, "yukseklik")?);
                    (genislik > 0.0 && yukseklik > 0.0).then(|| Kutu {
                        ad: k.get("ad").and_then(Value::as_str).unwrap_or("").chars().take(64).collect(),
                        sutun: k.get("sutun").and_then(Value::as_i64).unwrap_or(0).clamp(-1, 1) as i32,
                        satir: k.get("satir").and_then(Value::as_i64).unwrap_or(0).clamp(-1, 1) as i32,
                        bos_x: sayi(k, "bosX").unwrap_or(0.0).clamp(0.0, 100.0),
                        bos_y: sayi(k, "bosY").unwrap_or(0.0).clamp(0.0, 100.0),
                        genislik: genislik.min(400.0),
                        yukseklik: yukseklik.min(400.0),
                        renk: k.get("renk").and_then(Value::as_str).and_then(renk).unwrap_or(Color32::from_rgb(0x3E, 0x9C, 0xF0)),
                    })
                })
                .collect()
        })
        .unwrap_or_default()
}

/// `#rrggbb` → renk.
fn renk(metin: &str) -> Option<Color32> {
    let h = metin.strip_prefix('#')?;
    if h.len() != 6 || !h.is_ascii() {
        return None;
    }
    let b = |i: usize| h.get(i..i + 2).and_then(|parca| u8::from_str_radix(parca, 16).ok());
    Some(Color32::from_rgb(b(0)?, b(2)?, b(4)?))
}

/// Kutunun karedeki yeri (piksel; kare = sekansın ölçüsü). TM AutoClip'in filigran kuralı: kısa kenarın yüzdesiyle boşluk ve boyut.
pub fn yer(k: &Kutu, kare: (u32, u32)) -> Rect {
    let (w, h) = (kare.0 as f32, kare.1 as f32);
    let s = w.min(h).max(2.0);
    let (kw, kh) = (s * k.genislik / 100.0, s * k.yukseklik / 100.0);
    let (mx, my) = (s * k.bos_x / 100.0, s * k.bos_y / 100.0);
    let x = match k.sutun {
        -1 => mx,
        1 => w - kw - mx,
        _ => (w - kw) / 2.0,
    };
    let y = match k.satir {
        -1 => my,
        1 => h - kh - my,
        _ => (h - kh) / 2.0,
    };
    Rect::from_min_size(pos2(x, y), vec2(kw, kh))
}

/// Program monitörünün üstüne kılavuzları çizer (yalnız boya; tıklama almaz, hiçbir şey yazmaz). `pic` = karenin ekrandaki yeri;
/// `program` = program monitörü mü (kaynak monitöründe çizilmez).
pub fn monitor(ui: &egui::Ui, pic: Rect, kare: (u32, u32), program: bool) {
    if !program {
        return;
    }
    let kutular = kutular();
    if kutular.is_empty() || kare.0 == 0 || kare.1 == 0 {
        return;
    }
    let (kx, ky) = (pic.width() / kare.0 as f32, pic.height() / kare.1 as f32);
    let p = ui.painter();
    for k in kutular {
        let r = yer(k, kare);
        let e = Rect::from_min_max(pic.min + vec2(r.min.x * kx, r.min.y * ky), pic.min + vec2(r.max.x * kx, r.max.y * ky));
        let koseler = [e.left_top(), e.right_top(), e.right_bottom(), e.left_bottom(), e.left_top()];
        p.extend(egui::Shape::dashed_line(&koseler, Stroke::new(1.5, k.renk), 6.0, 4.0));
        if !k.ad.is_empty() {
            let g = p.layout_no_wrap(k.ad.clone(), Tokens::ui(10.0), Color32::WHITE);
            let etiket = Rect::from_min_size(e.left_top() + vec2(2.0, 2.0), g.size() + vec2(8.0, 4.0));
            p.rect_filled(etiket, 3.0, k.renk.gamma_multiply(0.85));
            p.rect_stroke(etiket, 3.0, Stroke::new(1.0, Color32::from_black_alpha(90)), StrokeKind::Inside);
            p.text(etiket.center(), Align2::CENTER_CENTER, &k.ad, Tokens::ui(10.0), Color32::WHITE);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{Kutu, oku, yer};
    use egui::Color32;

    #[test]
    fn dosya_okunur_ve_bozuk_kutu_atlanir() {
        let k = oku(
            r##"{"surum":1,"kutular":[{"ad":"Filigran","sutun":1,"satir":1,"bosX":3,"bosY":4,"genislik":20,"yukseklik":10,"renk":"#ff0000"},{"ad":"bos","genislik":0,"yukseklik":5}]}"##,
        );
        assert_eq!(k.len(), 1, "genişliği 0 olan kutu atlanır");
        assert_eq!(k[0].ad, "Filigran");
        assert_eq!((k[0].sutun, k[0].satir), (1, 1));
        assert_eq!(k[0].renk, Color32::from_rgb(255, 0, 0));
        assert!(oku("{\"surum\":2,\"kutular\":[]}").is_empty(), "başka sürüm okunmaz");
        assert!(oku("bozuk").is_empty());
    }

    #[test]
    fn yer_kisa_kenarin_yuzdesiyle_capaya_gore() {
        let k = |sutun, satir| Kutu { ad: String::new(), sutun, satir, bos_x: 5.0, bos_y: 10.0, genislik: 20.0, yukseklik: 10.0, renk: Color32::WHITE };
        // 1920×1080: kısa kenar 1080 → boşluk 54 / 108, kutu 216×108.
        let sag_alt = yer(&k(1, 1), (1920, 1080));
        assert!((sag_alt.min.x - (1920.0 - 216.0 - 54.0)).abs() < 0.01);
        assert!((sag_alt.min.y - (1080.0 - 108.0 - 108.0)).abs() < 0.01);
        let sol_ust = yer(&k(-1, -1), (1920, 1080));
        assert!((sol_ust.min.x - 54.0).abs() < 0.01 && (sol_ust.min.y - 108.0).abs() < 0.01);
        let orta = yer(&k(0, 0), (1920, 1080));
        assert!((orta.center().x - 960.0).abs() < 0.01 && (orta.center().y - 540.0).abs() < 0.01);
    }
}
