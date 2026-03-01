/// Sky Canvas — 60 fps, ultra-réaliste.
///
/// Nuit  : 300 étoiles twinkle organique (2 sinusoïdes incommensurables) +
///         étoiles filantes très rares (1-2 max à l'écran).
/// Jour  : ciel dégradé 14h30 + soleil avec halo pulsé + 9 nuages parallax.
/// Transition : cross-fade 800 ms avec dissolution douce des éléments.
use std::cell::{Cell, RefCell};
use std::f64::consts::TAU;
use std::rc::Rc;

use js_sys::Math;
use leptos::prelude::*;
use wasm_bindgen::{prelude::*, JsCast, JsValue};
use web_sys::{CanvasRenderingContext2d, HtmlCanvasElement};

use crate::theme::{Theme, ThemeCtx};

// ─── Thread-locals ────────────────────────────────────────────────────────────

thread_local! {
    /// Génération courante — la boucle rAF s'arrête si sa génération est dépassée.
    static ANIM_GEN: Cell<u32> = const { Cell::new(0) };
    /// Changement de thème en attente (consommé au prochain draw_frame).
    static PENDING: Cell<Option<bool>> = const { Cell::new(None) };
    /// La boucle est-elle déjà démarrée ?
    static STARTED: Cell<bool> = const { Cell::new(false) };
}

fn bump_gen() -> u32 {
    ANIM_GEN.with(|g| { let v = g.get().wrapping_add(1); g.set(v); v })
}
fn get_gen() -> u32 { ANIM_GEN.with(|g| g.get()) }

/// Notifie la boucle d'un changement de thème (appelé par le composant Leptos).
pub fn notify_theme(dark: bool) {
    PENDING.with(|p| p.set(Some(dark)));
}
fn take_pending() -> Option<bool> {
    PENDING.with(|p| p.replace(None))
}

// ─── Helpers ──────────────────────────────────────────────────────────────────

#[inline] fn rnd()              -> f64 { Math::random() }
#[inline] fn rng(lo: f64, hi: f64) -> f64 { lo + rnd() * (hi - lo) }

#[inline]
fn fill_grad(ctx: &CanvasRenderingContext2d, g: &web_sys::CanvasGradient) {
    #[allow(deprecated)]
    ctx.set_fill_style(g.as_ref());
}
#[inline]
fn stroke_grad(ctx: &CanvasRenderingContext2d, g: &web_sys::CanvasGradient) {
    #[allow(deprecated)]
    ctx.set_stroke_style(g.as_ref());
}

// ─── Étoile ───────────────────────────────────────────────────────────────────

struct Star {
    x: f64, y: f64,
    r: f64,
    base_a: f64,
    // Twinkle organique : somme de 2 sinusoïdes aux fréquences incommensurables
    f1: f64, p1: f64,   // onde rapide
    f2: f64, p2: f64,   // onde lente
    // Teinte légère (blanc / blanc-chaud / blanc-froid)
    rgb: &'static str,
}

impl Star {
    fn random(w: f64, h: f64) -> Self {
        // Distribution des tailles : 70 % petites, 23 % moyennes, 7 % brillantes
        let roll = rnd();
        let r = if roll < 0.70 { rng(0.25, 0.70) }
                else if roll < 0.93 { rng(0.70, 1.40) }
                else { rng(1.40, 2.55) };

        let t = rnd();
        let rgb = if t < 0.70 { "255,255,255" }
                  else if t < 0.85 { "255,240,200" }  // blanc-chaud
                  else { "200,220,255" };               // blanc-froid

        // Périodes en secondes à 60 fps : f = 1/(période_s * 60)
        Self {
            x: rnd() * w,
            y: rnd() * h,
            r,
            base_a: rng(0.35, 1.0),
            f1: rng(0.004, 0.012),    // 1.4–4.2 s
            p1: rnd() * TAU,
            f2: rng(0.001, 0.004),    // 4.2–16.7 s
            p2: rnd() * TAU,
            rgb,
        }
    }

    fn draw(&self, ctx: &CanvasRenderingContext2d, t: f64, mult: f64) {
        // Oscillation organique Perlin-like : somme de 2 sinus
        let wave = 0.50
            + 0.33 * (t * self.f1 * TAU + self.p1).sin()
            + 0.17 * (t * self.f2 * TAU + self.p2).sin();
        let a = (self.base_a * wave.max(0.04)).min(1.0) * mult;
        if a < 0.02 { return; }

        ctx.save();
        ctx.set_global_alpha(a);

        // Disque de l'étoile
        ctx.begin_path();
        let _ = ctx.arc(self.x, self.y, self.r, 0.0, TAU);
        #[allow(deprecated)]
        ctx.set_fill_style(&JsValue::from_str(&format!("rgb({})", self.rgb)));
        ctx.fill();

        // Halo pour les grandes étoiles
        if self.r > 1.3 {
            let gr = self.r * 4.8;
            let gx = ctx.create_radial_gradient(
                self.x, self.y, 0.0,
                self.x, self.y, gr,
            );
            if let Ok(g) = gx {
                let _ = g.add_color_stop(0.0, &format!("rgba({},0.50)", self.rgb));
                let _ = g.add_color_stop(1.0, "rgba(0,0,0,0)");
                fill_grad(ctx, &g);
                ctx.begin_path();
                let _ = ctx.arc(self.x, self.y, gr, 0.0, TAU);
                ctx.fill();
            }
        }
        ctx.restore();
    }
}

// ─── Étoile filante ───────────────────────────────────────────────────────────

struct Shooter {
    x: f64, y: f64,
    vx: f64, vy: f64,
    trail: f64,
    max_a: f64,
    life: f64,  // 0 → 1
}

impl Shooter {
    fn spawn(w: f64, h: f64) -> Self {
        // Trajectoire : légèrement descendante vers la droite
        let ang = rng(0.06 * TAU, 0.14 * TAU);
        let spd = rng(9.0, 16.0);
        Self {
            x:     rng(w * 0.05, w * 0.72),
            y:     rng(h * 0.03, h * 0.38),
            vx:    spd *  ang.cos(),
            vy:    spd *  ang.sin(),
            trail: rng(100.0, 220.0),
            max_a: rng(0.60, 1.00),
            life:  0.0,
        }
    }

    /// Retourne `true` tant qu'elle est vivante.
    fn tick(&mut self) -> bool {
        self.x   += self.vx;
        self.y   += self.vy;
        self.life += 0.017;  // ~59 frames ≈ 1 s de vie
        self.life < 1.0
    }

    fn draw(&self, ctx: &CanvasRenderingContext2d, mult: f64) {
        // Enveloppe : montée rapide 0→0.25, descente lente 0.25→1
        let env = if self.life < 0.25 { self.life / 0.25 }
                  else { 1.0 - (self.life - 0.25) / 0.75 };
        let a = (env * self.max_a * mult).max(0.0);
        if a < 0.01 { return; }

        let spd = (self.vx * self.vx + self.vy * self.vy).sqrt();
        let (dx, dy) = (self.vx / spd, self.vy / spd);
        let (tx, ty) = (self.x - dx * self.trail, self.y - dy * self.trail);

        let g = ctx.create_linear_gradient(tx, ty, self.x, self.y);
        let _ = g.add_color_stop(0.0,  "rgba(255,255,255,0)");
        let _ = g.add_color_stop(0.60, &format!("rgba(210,230,255,{})", a * 0.55));
        let _ = g.add_color_stop(1.0,  &format!("rgba(255,255,255,{})", a));

        ctx.save();
        ctx.begin_path();
        ctx.move_to(tx, ty);
        ctx.line_to(self.x, self.y);
        stroke_grad(ctx, &g);
        ctx.set_line_width(1.8);
        ctx.stroke();

        // Tête brillante
        ctx.set_global_alpha(a);
        ctx.begin_path();
        let _ = ctx.arc(self.x, self.y, 1.8, 0.0, TAU);
        #[allow(deprecated)]
        ctx.set_fill_style(&JsValue::from_str("white"));
        ctx.fill();
        ctx.restore();
    }
}

// ─── Nuage ────────────────────────────────────────────────────────────────────

struct Cloud {
    x: f64, y: f64,
    speed: f64,
    alpha: f64,
    blobs: Vec<(f64, f64, f64)>,  // (dx, dy, rayon) relatif au centre
    span: f64,   // demi-largeur pour la détection de sortie
    cw:   f64,   // largeur du canvas (pour wrap)
}

impl Cloud {
    fn random(cw: f64, ch: f64) -> Self {
        let scale = rng(0.50, 1.65);
        let y     = rng(ch * 0.05, ch * 0.42);
        // Parallax : les nuages plus grands (premier plan) vont plus vite
        let speed = scale * rng(0.10, 0.30);
        let alpha = rng(0.68, 0.95);
        let br    = scale * rng(40.0, 88.0);
        let n     = rng(5.0, 10.0) as usize;

        let mut blobs: Vec<(f64, f64, f64)> = vec![(0.0, 0.0, br)];
        for _ in 1..n {
            let ang  = rnd() * TAU;
            let dist = rng(br * 0.22, br * 0.85);
            let bx   = ang.cos() * dist;
            // Les blobs sont biaisés vers le haut (nuages : sommet bombé)
            let by   = (ang.sin() * dist * 0.42).abs();
            let r    = br * rng(0.42, 0.90);
            blobs.push((bx, -by, r));
        }

        let span = blobs.iter()
            .map(|(bx, _, r)| bx.abs() + r)
            .fold(0.0_f64, f64::max)
            + 10.0;

        Self { x: rnd() * cw, y, speed, alpha, blobs, span, cw }
    }

    fn tick(&mut self) {
        self.x += self.speed;
        if self.x - self.span > self.cw * 1.1 {
            self.x = -self.span * 2.2;
        }
    }

    fn draw(&self, ctx: &CanvasRenderingContext2d, mult: f64) {
        let a = self.alpha * mult;
        if a < 0.01 { return; }
        ctx.save();
        ctx.set_global_alpha(a);

        for &(bx, by, r) in &self.blobs {
            let cx = self.x + bx;
            let cy = self.y + by;
            // Dégradé radial : centre blanc pur → bords fondus
            let gx = ctx.create_radial_gradient(
                cx, cy - r * 0.20, r * 0.06,
                cx, cy,            r,
            );
            if let Ok(g) = gx {
                let _ = g.add_color_stop(0.0,  "rgba(255,255,255,1.0)");
                let _ = g.add_color_stop(0.42, "rgba(250,252,255,0.88)");
                let _ = g.add_color_stop(0.78, "rgba(238,248,255,0.50)");
                let _ = g.add_color_stop(1.0,  "rgba(224,242,255,0.0)");
                fill_grad(ctx, &g);
                ctx.begin_path();
                let _ = ctx.arc(cx, cy, r, 0.0, TAU);
                ctx.fill();
            }
        }
        ctx.restore();
    }
}

// ─── Arrière-plans ────────────────────────────────────────────────────────────

fn draw_night_sky(ctx: &CanvasRenderingContext2d, w: f64, h: f64, a: f64) {
    if a < 0.01 { return; }
    let g = ctx.create_linear_gradient(0.0, 0.0, 0.0, h);
    let _ = g.add_color_stop(0.0,  "#020617");   // slate-950
    let _ = g.add_color_stop(0.45, "#0f172a");   // slate-900
    let _ = g.add_color_stop(1.0,  "#1e293b");   // slate-800
    ctx.save();
    ctx.set_global_alpha(a);
    fill_grad(ctx, &g);
    ctx.fill_rect(0.0, 0.0, w, h);
    ctx.restore();
}

fn draw_day_sky(ctx: &CanvasRenderingContext2d, w: f64, h: f64, a: f64) {
    if a < 0.01 { return; }
    ctx.save();
    ctx.set_global_alpha(a);
    // Dégradé principal ciel 14h30 : bleu profond → bleu ciel → blanc-bleuté
    let g = ctx.create_linear_gradient(0.0, 0.0, 0.0, h);
    let _ = g.add_color_stop(0.00, "#1a6dbf");
    let _ = g.add_color_stop(0.28, "#4a9eda");
    let _ = g.add_color_stop(0.58, "#82c8f0");
    let _ = g.add_color_stop(0.85, "#c4e8f8");
    let _ = g.add_color_stop(1.00, "#eaf6ff");
    fill_grad(ctx, &g);
    ctx.fill_rect(0.0, 0.0, w, h);
    // Brume d'horizon
    let hz = ctx.create_linear_gradient(0.0, h * 0.72, 0.0, h);
    let _ = hz.add_color_stop(0.0, "rgba(255,255,255,0)");
    let _ = hz.add_color_stop(1.0, "rgba(255,255,255,0.20)");
    fill_grad(ctx, &hz);
    ctx.fill_rect(0.0, h * 0.72, w, h * 0.28);
    ctx.restore();
}

fn draw_sun(ctx: &CanvasRenderingContext2d, w: f64, h: f64, t: f64, a: f64) {
    if a < 0.01 { return; }
    // Position 14h30 : ~72 % en x, ~20 % en y
    let sx = w * 0.72;
    let sy = h * 0.20;
    let r  = 36.0;

    // Deux pulsations indépendantes pour plus d'organicité
    let p1 = 1.0 + 0.07 * (t * 0.00048 * TAU).sin();
    let p2 = 1.0 + 0.045 * (t * 0.00019 * TAU).sin();

    ctx.save();
    ctx.set_global_alpha(a);

    // ── Reflet large et diffus (lumière solaire dans le ciel) ────────────────
    let gx = ctx.create_radial_gradient(sx, sy, 0.0, sx, sy, r * 14.0 * p2);
    if let Ok(g) = gx {
        let _ = g.add_color_stop(0.0,  "rgba(255,250,200,0.18)");
        let _ = g.add_color_stop(0.40, "rgba(255,235,150,0.07)");
        let _ = g.add_color_stop(0.75, "rgba(255,220,100,0.02)");
        let _ = g.add_color_stop(1.0,  "rgba(255,200, 50,0)");
        fill_grad(ctx, &g);
        ctx.begin_path();
        let _ = ctx.arc(sx, sy, r * 14.0 * p2, 0.0, TAU);
        ctx.fill();
    }

    // ── Lueur douce proche (aureole subtile) ─────────────────────────────────
    let gx = ctx.create_radial_gradient(sx, sy, 0.0, sx, sy, r * 3.5 * p1);
    if let Ok(g) = gx {
        let _ = g.add_color_stop(0.0,  "rgba(255,255,230,0.22)");
        let _ = g.add_color_stop(0.55, "rgba(255,245,180,0.08)");
        let _ = g.add_color_stop(1.0,  "rgba(255,230,120,0)");
        fill_grad(ctx, &g);
        ctx.begin_path();
        let _ = ctx.arc(sx, sy, r * 3.5 * p1, 0.0, TAU);
        ctx.fill();
    }

    // ── Disque solaire quasi-invisible — juste une tache claire ──────────────
    let gx = ctx.create_radial_gradient(sx, sy, 0.0, sx, sy, r);
    if let Ok(g) = gx {
        let _ = g.add_color_stop(0.0,  "rgba(255,255,255,0.28)");
        let _ = g.add_color_stop(0.60, "rgba(255,252,210,0.10)");
        let _ = g.add_color_stop(1.0,  "rgba(255,240,160,0)");
        fill_grad(ctx, &g);
        ctx.begin_path();
        let _ = ctx.arc(sx, sy, r, 0.0, TAU);
        ctx.fill();
    }

    ctx.restore();
}

// ─── État principal de l'animation ───────────────────────────────────────────

struct SkyAnim {
    ctx:  CanvasRenderingContext2d,
    w: f64, h: f64,
    t: f64,

    // Éléments nuit
    stars:    Vec<Star>,
    shooters: Vec<Shooter>,
    shoot_cd: f64,   // cooldown avant la prochaine étoile filante

    // Éléments jour
    clouds: Vec<Cloud>,

    // État du thème et de la transition
    is_dark:    bool,
    prev_dark:  bool,
    blend:      f64,   // 0 → 1 (thème entrant)
    in_trans:   bool,
}

impl SkyAnim {
    fn new(ctx: CanvasRenderingContext2d, w: f64, h: f64, dark: bool) -> Self {
        Self {
            ctx, w, h, t: 0.0,
            stars:     (0..300).map(|_| Star::random(w, h)).collect(),
            shooters:  Vec::with_capacity(2),
            shoot_cd:  rng(480.0, 1800.0),
            clouds:    (0..9).map(|_| Cloud::random(w, h)).collect(),
            is_dark: dark, prev_dark: dark,
            blend: 1.0, in_trans: false,
        }
    }

    fn switch_theme(&mut self, dark: bool) {
        if dark == self.is_dark && !self.in_trans { return; }
        self.prev_dark = self.is_dark;
        self.is_dark   = dark;
        self.blend     = 0.0;
        self.in_trans  = true;
    }

    fn draw_frame(&mut self) {
        // Consomme le changement de thème en attente
        if let Some(dark) = take_pending() { self.switch_theme(dark); }

        // Avance la transition : 800 ms ≈ 48 frames → +0.021/frame
        if self.in_trans {
            self.blend = (self.blend + 0.021).min(1.0);
            if self.blend >= 1.0 { self.in_trans = false; }
        }

        let ctx = &self.ctx;
        let (w, h, t) = (self.w, self.h, self.t);
        ctx.clear_rect(0.0, 0.0, w, h);

        if self.in_trans {
            let b = self.blend; // 0 → 1 (thème entrant)
            if self.prev_dark {
                // Sortant = nuit → entrant = jour
                draw_night_sky(ctx, w, h, 1.0);
                for s in &self.stars    { s.draw(ctx, t, 1.0 - b); }
                for s in &self.shooters { s.draw(ctx, 1.0 - b); }
                draw_day_sky(ctx, w, h, b);
                draw_sun(ctx, w, h, t, b);
                for c in &self.clouds   { c.draw(ctx, b); }
            } else {
                // Sortant = jour → entrant = nuit
                draw_day_sky(ctx, w, h, 1.0);
                draw_sun(ctx, w, h, t, 1.0 - b);
                for c in &self.clouds   { c.draw(ctx, 1.0 - b); }
                draw_night_sky(ctx, w, h, b);
                for s in &self.stars    { s.draw(ctx, t, b); }
            }
        } else if self.is_dark {
            draw_night_sky(ctx, w, h, 1.0);
            for s in &self.stars    { s.draw(ctx, t, 1.0); }
            for s in &self.shooters { s.draw(ctx, 1.0); }
        } else {
            draw_day_sky(ctx, w, h, 1.0);
            draw_sun(ctx, w, h, t, 1.0);
            for c in &self.clouds   { c.draw(ctx, 1.0); }
        }

        self.t += 1.0;

        // ── Étoiles filantes ────────────────────────────────────────────────
        let night_visible = self.is_dark || (self.in_trans && self.prev_dark);
        self.shooters.retain_mut(|s| s.tick());
        if night_visible {
            self.shoot_cd -= 1.0;
            if self.shoot_cd <= 0.0 && self.shooters.len() < 2 {
                self.shooters.push(Shooter::spawn(w, h));
                self.shoot_cd = rng(600.0, 1800.0);  // 10–30 s entre apparitions
            }
        }

        // ── Nuages ──────────────────────────────────────────────────────────
        for c in &mut self.clouds { c.tick(); }
    }
}

// ─── Lancement de la boucle rAF ──────────────────────────────────────────────

fn start_animation(canvas: HtmlCanvasElement, dark: bool) {
    let window = match web_sys::window() { Some(w) => w, None => return };
    let vw = window.inner_width().unwrap().as_f64().unwrap_or(1280.0);
    let vh = window.inner_height().unwrap().as_f64().unwrap_or(800.0);
    canvas.set_width(vw as u32);
    canvas.set_height(vh as u32);

    let ctx: CanvasRenderingContext2d = match canvas
        .get_context("2d").ok().flatten()
        .and_then(|c| c.dyn_into().ok())
    {
        Some(c) => c,
        None    => return,
    };

    STARTED.with(|s| s.set(true));
    let my_gen = bump_gen();

    let anim = Rc::new(RefCell::new(SkyAnim::new(ctx, vw, vh, dark)));

    // Pattern rAF auto-référentiel (doc officielle wasm-bindgen)
    let f: Rc<RefCell<Option<Closure<dyn FnMut()>>>> = Rc::new(RefCell::new(None));
    let g = f.clone();

    *g.borrow_mut() = Some(Closure::wrap(Box::new({
        let anim = anim.clone();
        let win  = window.clone();
        move || {
            if get_gen() != my_gen {
                let _ = f.borrow_mut().take(); // stoppe la boucle
                return;
            }
            anim.borrow_mut().draw_frame();
            win.request_animation_frame(
                f.borrow().as_ref().unwrap().as_ref().unchecked_ref(),
            ).unwrap();
        }
    }) as Box<dyn FnMut()>));

    window.request_animation_frame(
        g.borrow().as_ref().unwrap().as_ref().unchecked_ref(),
    ).unwrap();
}

// ─── Composant Leptos ─────────────────────────────────────────────────────────

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

        if STARTED.with(|s| s.get()) {
            // La boucle tourne déjà → signale simplement le changement de thème
            notify_theme(is_dark);
        } else if let Some(canvas) = canvas_ref.get() {
            start_animation(canvas, is_dark);
        }
    });

    view! {
        <canvas
            id="sky-canvas"
            node_ref=canvas_ref
            style="position:fixed;inset:0;width:100%;height:100%;z-index:-1;pointer-events:none;"
        />
    }
}
