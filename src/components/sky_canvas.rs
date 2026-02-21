/// Canvas d'arrière-plan animé représentant le ciel.
///
/// Mode sombre : étoiles qui scintillent (opacité oscillante).
/// Mode clair  : nuages blancs qui dérivent lentement.
///
/// Boucle `requestAnimationFrame` — pattern officiel wasm-bindgen :
/// <https://rustwasm.github.io/wasm-bindgen/examples/request-animation-frame.html>
use std::cell::RefCell;
use std::f64::consts::TAU;
use std::rc::Rc;

use leptos::prelude::*;
use wasm_bindgen::{prelude::*, JsCast, JsValue};
use web_sys::{CanvasRenderingContext2d, HtmlCanvasElement};

use crate::app::{Theme, ThemeCtx};

// ─── Génération : stoppe l'ancienne boucle quand le thème change ─────────────

thread_local! {
    static ANIM_GEN: std::cell::Cell<u32> = const { std::cell::Cell::new(0) };
}

fn bump_gen() -> u32 {
    ANIM_GEN.with(|g| {
        let v = g.get().wrapping_add(1);
        g.set(v);
        v
    })
}
fn get_gen() -> u32 {
    ANIM_GEN.with(|g| g.get())
}

// ─── Étoile scintillante ─────────────────────────────────────────────────────

struct Star {
    x: f64,
    y: f64,
    r: f64,
    alpha: f64,
    delta: f64, // vitesse d'oscillation de l'opacité
}

impl Star {
    fn random(w: f64, h: f64) -> Self {
        let sign = if js_sys::Math::random() > 0.5 { 1.0 } else { -1.0 };
        Self {
            x: js_sys::Math::random() * w,
            y: js_sys::Math::random() * h,
            r: js_sys::Math::random() * 1.5 + 0.2,
            alpha: js_sys::Math::random(),
            delta: (js_sys::Math::random() * 0.013 + 0.002) * sign,
        }
    }

    fn tick(&mut self) {
        self.alpha += self.delta;
        if self.alpha >= 1.0 {
            self.alpha = 1.0;
            self.delta = -self.delta.abs();
        } else if self.alpha <= 0.04 {
            self.alpha = 0.04;
            self.delta = self.delta.abs();
        }
    }

    fn draw(&self, ctx: &CanvasRenderingContext2d) {
        ctx.save();
        ctx.set_global_alpha(self.alpha);
        ctx.begin_path();
        let _ = ctx.arc(self.x, self.y, self.r, 0.0, TAU);
        #[allow(deprecated)]
        ctx.set_fill_style(&JsValue::from_str("white"));
        ctx.fill();
        ctx.restore();
    }
}

// ─── Nuage flottant ──────────────────────────────────────────────────────────

struct Cloud {
    x: f64,
    y: f64,
    blobs: Vec<(f64, f64, f64)>, // (dx, dy, rayon)
    speed: f64,
    alpha: f64,
    half_span: f64, // moitié de la largeur totale
}

impl Cloud {
    fn random(w: f64, h: f64) -> Self {
        let base = js_sys::Math::random() * 40.0 + 22.0;
        let n = (js_sys::Math::random() * 3.0 + 3.0) as usize;
        let blobs: Vec<(f64, f64, f64)> = (0..n)
            .map(|_| {
                (
                    (js_sys::Math::random() - 0.5) * base * 3.2,
                    (js_sys::Math::random() - 0.5) * base * 0.45,
                    js_sys::Math::random() * base * 0.5 + base * 0.45,
                )
            })
            .collect();

        let half_span = blobs
            .iter()
            .map(|(dx, _, r)| dx.abs() + r)
            .fold(0.0_f64, f64::max)
            + 10.0;

        Self {
            x: js_sys::Math::random() * w,
            y: js_sys::Math::random() * h * 0.42 + 18.0,
            blobs,
            speed: js_sys::Math::random() * 0.22 + 0.04,
            alpha: js_sys::Math::random() * 0.28 + 0.07,
            half_span,
        }
    }

    fn tick(&mut self, w: f64) {
        self.x += self.speed;
        if self.x > w + self.half_span {
            self.x = -self.half_span;
        }
    }

    fn draw(&self, ctx: &CanvasRenderingContext2d) {
        ctx.save();
        ctx.set_global_alpha(self.alpha);
        #[allow(deprecated)]
        ctx.set_fill_style(&JsValue::from_str("white"));
        for &(dx, dy, r) in &self.blobs {
            ctx.begin_path();
            let _ = ctx.arc(self.x + dx, self.y + dy, r, 0.0, TAU);
            ctx.fill();
        }
        ctx.restore();
    }
}

// ─── État de l'animation ─────────────────────────────────────────────────────

struct SkyAnim {
    ctx: CanvasRenderingContext2d,
    stars: Vec<Star>,
    clouds: Vec<Cloud>,
    w: f64,
    h: f64,
    dark: bool,
}

impl SkyAnim {
    fn draw_frame(&mut self) {
        let ctx = &self.ctx;
        ctx.clear_rect(0.0, 0.0, self.w, self.h);

        // ── Gradient de fond ────────────────────────────────────────────────
        let grad = ctx.create_linear_gradient(0.0, 0.0, 0.0, self.h);
        if self.dark {
            let _ = grad.add_color_stop(0.0,  "#020617"); // slate-950
            let _ = grad.add_color_stop(0.55, "#0f172a"); // slate-900
            let _ = grad.add_color_stop(1.0,  "#1e293b"); // slate-800
        } else {
            let _ = grad.add_color_stop(0.0,  "#bfdbfe"); // blue-200
            let _ = grad.add_color_stop(0.45, "#dbeafe"); // blue-100
            let _ = grad.add_color_stop(1.0,  "#f0f9ff"); // sky-50
        }
        #[allow(deprecated)]
        ctx.set_fill_style(grad.as_ref());
        ctx.fill_rect(0.0, 0.0, self.w, self.h);

        // ── Étoiles ou nuages ───────────────────────────────────────────────
        if self.dark {
            for star in &mut self.stars {
                star.tick();
                star.draw(ctx);
            }
        } else {
            for cloud in &mut self.clouds {
                cloud.tick(self.w);
                cloud.draw(ctx);
            }
        }
    }
}

// ─── Lancement de la boucle rAF ──────────────────────────────────────────────

fn start_animation(canvas: HtmlCanvasElement, is_dark: bool) {
    let window = web_sys::window().unwrap();
    let vw = window.inner_width().unwrap().as_f64().unwrap();
    let vh = window.inner_height().unwrap().as_f64().unwrap();

    canvas.set_width(vw as u32);
    canvas.set_height(vh as u32);

    let ctx: CanvasRenderingContext2d = canvas
        .get_context("2d")
        .unwrap()
        .unwrap()
        .dyn_into()
        .unwrap();

    let my_gen = bump_gen(); // invalide automatiquement l'ancienne boucle

    let anim = Rc::new(RefCell::new(SkyAnim {
        ctx,
        stars: (0..230).map(|_| Star::random(vw, vh)).collect(),
        clouds: (0..8).map(|_| Cloud::random(vw, vh)).collect(),
        w: vw,
        h: vh,
        dark: is_dark,
    }));

    // ── Pattern rAF auto-référentiel (doc wasm-bindgen officielle) ───────────
    //
    // `f` est capturé par la closure stockée dans `g`.
    // Quand `g` quitte la portée, le Rc count tombe à 1 (depuis `f` à l'intérieur
    // de la closure). La closure reste vivante aussi longtemps qu'elle s'auto-appelle.
    // Pour stopper, on fait `f.borrow_mut().take()` — le Rc tombe à 0, tout est libéré.
    let f: Rc<RefCell<Option<Closure<dyn FnMut()>>>> = Rc::new(RefCell::new(None));
    let g = f.clone();

    *g.borrow_mut() = Some(Closure::wrap(Box::new({
        let anim = anim.clone();
        move || {
            // L'ancienne boucle se suicide si une nouvelle génération a démarré
            if get_gen() != my_gen {
                let _ = f.borrow_mut().take(); // drop la closure → arrêt
                return;
            }

            anim.borrow_mut().draw_frame();

            // Planifie le prochain frame
            web_sys::window()
                .unwrap()
                .request_animation_frame(
                    f.borrow().as_ref().unwrap().as_ref().unchecked_ref(),
                )
                .unwrap();
        }
    }) as Box<dyn FnMut()>));

    // Premier frame
    web_sys::window()
        .unwrap()
        .request_animation_frame(
            g.borrow().as_ref().unwrap().as_ref().unchecked_ref(),
        )
        .unwrap();

    // `g` quitte la portée ici → Rc count 1 (depuis f à l'intérieur de la closure)
    // La boucle continue indéfiniment (ou jusqu'au prochain changement de thème).
}

// ─── Composant Leptos ────────────────────────────────────────────────────────

#[component]
pub fn SkyCanvas() -> impl IntoView {
    let canvas_ref: NodeRef<leptos::html::Canvas> = NodeRef::new();
    let theme_ctx = use_context::<ThemeCtx>().expect("ThemeCtx manquant");

    Effect::new(move |_| {
        let is_dark = match theme_ctx.theme.get() {
            Theme::Dark   => true,
            Theme::Light  => false,
            Theme::System => web_sys::window()
                .and_then(|w| w.match_media("(prefers-color-scheme: dark)").ok().flatten())
                .map(|mq| mq.matches())
                .unwrap_or(false),
        };

        if let Some(canvas) = canvas_ref.get() {
            start_animation(canvas, is_dark);
        }
    });

    view! {
        // Style inline = priorité maximale — garantit position:fixed même si main.css
        // charge après le premier rendu WASM (évite que le canvas prenne 100vh dans le flux).
        <canvas
            id="sky-canvas"
            node_ref=canvas_ref
            style="position:fixed;inset:0;width:100%;height:100%;z-index:0;pointer-events:none;"
        />
    }
}
