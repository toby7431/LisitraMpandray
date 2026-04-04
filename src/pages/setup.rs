/// Page de configuration au premier lancement.
///
/// L'utilisateur choisit le mode (Serveur ou Client) et,
/// en mode Client, entre l'adresse IP du PC serveur.
/// En mode Serveur, il définit le code PIN administrateur (une seule fois).
/// Après validation, la config est sauvegardée et l'app passe en mode normal.
use leptos::prelude::*;

use crate::services::config_service::{
    save_config, set_pin, start_mock_server, test_server_connection, AppConfig, AppMode,
};

#[component]
pub fn SetupPage(
    /// Signal mis à jour quand la configuration est terminée (Some(true)).
    is_configured: RwSignal<Option<bool>>,
) -> impl IntoView {
    // ── Signaux de formulaire ─────────────────────────────────────────────────
    let mode = RwSignal::new(AppMode::Server);
    let server_ip = RwSignal::new(String::new());
    let server_port = RwSignal::new(7654u16);

    // PIN (mode Serveur uniquement)
    let pin_val     = RwSignal::new(String::new());
    let pin_confirm = RwSignal::new(String::new());

    // ── État UI ───────────────────────────────────────────────────────────────
    let saving    = RwSignal::new(false);
    let testing   = RwSignal::new(false);
    let simulating = RwSignal::new(false);
    let test_result: RwSignal<Option<bool>> = RwSignal::new(None);
    let error_msg: RwSignal<Option<String>> = RwSignal::new(None);

    // ── Actions ───────────────────────────────────────────────────────────────

    let on_test = move |_| {
        let ip   = server_ip.get();
        let port = server_port.get();
        if ip.trim().is_empty() {
            error_msg.set(Some("Veuillez saisir l'adresse IP du serveur.".to_string()));
            return;
        }
        error_msg.set(None);
        test_result.set(None);
        testing.set(true);
        leptos::task::spawn_local(async move {
            match test_server_connection(&ip, port).await {
                Ok(ok) => test_result.set(Some(ok)),
                Err(e) => error_msg.set(Some(e)),
            }
            testing.set(false);
        });
    };

    let on_simulate = move |_| {
        error_msg.set(None);
        test_result.set(None);
        simulating.set(true);
        leptos::task::spawn_local(async move {
            match start_mock_server().await {
                Ok(port) => {
                    server_ip.set("127.0.0.1".to_string());
                    server_port.set(port);
                    match test_server_connection("127.0.0.1", port).await {
                        Ok(ok) => test_result.set(Some(ok)),
                        Err(e) => error_msg.set(Some(format!("Erreur de test : {e}"))),
                    }
                }
                Err(e) => error_msg.set(Some(
                    format!("Impossible de démarrer le serveur local : {e}")
                )),
            }
            simulating.set(false);
        });
    };

    let on_save = move |_| {
        let current_mode = mode.get();
        let ip   = server_ip.get();
        let port = server_port.get();
        let pin  = pin_val.get();
        let conf = pin_confirm.get();

        // Validation mode Client
        if current_mode == AppMode::Client && ip.trim().is_empty() {
            error_msg.set(Some("Veuillez saisir l'adresse IP du serveur.".to_string()));
            return;
        }

        // Validation PIN mode Serveur
        if current_mode == AppMode::Server {
            if pin.trim().len() < 4 {
                error_msg.set(Some(
                    "Le code PIN doit contenir au moins 4 chiffres.".to_string()
                ));
                return;
            }
            if pin != conf {
                error_msg.set(Some(
                    "Les codes PIN ne correspondent pas.".to_string()
                ));
                return;
            }
        }

        error_msg.set(None);
        saving.set(true);

        let config = AppConfig {
            mode: current_mode.clone(),
            server_ip: if current_mode == AppMode::Client {
                ip.trim().to_string()
            } else {
                "0.0.0.0".to_string()
            },
            server_port: port,
        };

        leptos::task::spawn_local(async move {
            match save_config(&config).await {
                Err(e) => {
                    error_msg.set(Some(format!("Erreur : {e}")));
                    saving.set(false);
                }
                Ok(_) => {
                    // En mode Serveur : enregistrer le PIN après la config
                    if current_mode == AppMode::Server {
                        if let Err(e) = set_pin(&pin).await {
                            error_msg.set(Some(format!("Erreur PIN : {e}")));
                            saving.set(false);
                            return;
                        }
                    }
                    is_configured.set(Some(true));
                }
            }
        });
    };

    // ── Vue ───────────────────────────────────────────────────────────────────

    view! {
        <div class="fixed inset-0 z-50 flex items-center justify-center px-4">
            <div class="w-full max-w-md
                        bg-white/90 dark:bg-slate-800/90
                        backdrop-blur-sm
                        rounded-2xl shadow-2xl
                        border border-white/30 dark:border-slate-700/50
                        p-8 space-y-6">

                // ── En-tête ──────────────────────────────────────────────────
                <div class="text-center space-y-2">
                    <div class="text-4xl mb-2">"⛪"</div>
                    <h1 class="text-2xl font-bold
                               text-blue-900 dark:text-blue-100
                               font-serif">
                        "Configuration initiale"
                    </h1>
                    <p class="text-sm text-slate-500 dark:text-slate-400">
                        "Ce PC est-il le serveur ou le client ?"
                    </p>
                </div>

                // ── Choix du mode ─────────────────────────────────────────────
                <div class="grid grid-cols-2 gap-3">
                    <button
                        class=move || {
                            let base = "flex flex-col items-center gap-2 p-4 rounded-xl border-2 \
                                        transition-all duration-200 cursor-pointer ";
                            if mode.get() == AppMode::Server {
                                format!("{base}border-blue-500 bg-blue-50 dark:bg-blue-900/30 \
                                         text-blue-700 dark:text-blue-300")
                            } else {
                                format!("{base}border-slate-200 dark:border-slate-600 \
                                         text-slate-600 dark:text-slate-400 \
                                         hover:border-blue-300 dark:hover:border-blue-600")
                            }
                        }
                        on:click=move |_| {
                            mode.set(AppMode::Server);
                            test_result.set(None);
                            error_msg.set(None);
                        }
                    >
                        <span class="text-2xl">"🖥️"</span>
                        <span class="font-semibold text-sm">"Serveur"</span>
                        <span class="text-xs text-center opacity-70">
                            "Ce PC stocke les données"
                        </span>
                    </button>

                    <button
                        class=move || {
                            let base = "flex flex-col items-center gap-2 p-4 rounded-xl border-2 \
                                        transition-all duration-200 cursor-pointer ";
                            if mode.get() == AppMode::Client {
                                format!("{base}border-green-500 bg-green-50 dark:bg-green-900/30 \
                                         text-green-700 dark:text-green-300")
                            } else {
                                format!("{base}border-slate-200 dark:border-slate-600 \
                                         text-slate-600 dark:text-slate-400 \
                                         hover:border-green-300 dark:hover:border-green-600")
                            }
                        }
                        on:click=move |_| {
                            mode.set(AppMode::Client);
                            error_msg.set(None);
                        }
                    >
                        <span class="text-2xl">"💻"</span>
                        <span class="font-semibold text-sm">"Client"</span>
                        <span class="text-xs text-center opacity-70">
                            "Se connecte au serveur"
                        </span>
                    </button>
                </div>

                // ── Champs mode Serveur (PIN) ─────────────────────────────────
                <Show when=move || mode.get() == AppMode::Server>
                    <div class="space-y-3">
                        <div class="rounded-lg bg-blue-50 dark:bg-blue-900/20
                                    border border-blue-200 dark:border-blue-800
                                    p-3 text-sm text-blue-700 dark:text-blue-300">
                            <p class="font-medium mb-1">"ℹ️ Ce PC sera le serveur"</p>
                            <p class="opacity-80">
                                "La base de données sera stockée ici. "
                                "Le serveur API démarrera sur le port "
                                <strong>{move || server_port.get()}</strong>
                                " pour permettre aux autres PC de se connecter."
                            </p>
                        </div>

                        <div>
                            <label class="block text-sm font-medium
                                          text-slate-700 dark:text-slate-300 mb-1">
                                "Code PIN administrateur"
                            </label>
                            <input
                                type="password"
                                inputmode="numeric"
                                maxlength="20"
                                placeholder="Minimum 4 chiffres"
                                class="w-full px-3 py-2 rounded-lg border
                                       border-slate-300 dark:border-slate-600
                                       bg-white dark:bg-slate-700
                                       text-slate-900 dark:text-slate-100
                                       placeholder-slate-400
                                       focus:outline-none focus:ring-2 focus:ring-blue-400
                                       text-sm"
                                prop:value=move || pin_val.get()
                                on:input=move |ev| pin_val.set(event_target_value(&ev))
                            />
                        </div>

                        <div>
                            <label class="block text-sm font-medium
                                          text-slate-700 dark:text-slate-300 mb-1">
                                "Confirmer le code PIN"
                            </label>
                            <input
                                type="password"
                                inputmode="numeric"
                                maxlength="20"
                                placeholder="Répétez le PIN"
                                class="w-full px-3 py-2 rounded-lg border
                                       border-slate-300 dark:border-slate-600
                                       bg-white dark:bg-slate-700
                                       text-slate-900 dark:text-slate-100
                                       placeholder-slate-400
                                       focus:outline-none focus:ring-2 focus:ring-blue-400
                                       text-sm"
                                prop:value=move || pin_confirm.get()
                                on:input=move |ev| pin_confirm.set(event_target_value(&ev))
                            />
                        </div>

                        <p class="text-xs text-slate-400 dark:text-slate-500">
                            "⚠ Ce code PIN est définitif et ne peut pas être modifié ultérieurement. "
                            "Il sera requis pour modifier des contributions dans les archives."
                        </p>
                    </div>
                </Show>

                // ── Champs mode Client ────────────────────────────────────────
                <Show when=move || mode.get() == AppMode::Client>
                    <div class="space-y-3">
                        <div>
                            <label class="block text-sm font-medium
                                          text-slate-700 dark:text-slate-300 mb-1">
                                "Adresse IP du serveur"
                            </label>
                            <input
                                type="text"
                                placeholder="ex: 192.168.1.10"
                                class="w-full px-3 py-2 rounded-lg border
                                       border-slate-300 dark:border-slate-600
                                       bg-white dark:bg-slate-700
                                       text-slate-900 dark:text-slate-100
                                       placeholder-slate-400
                                       focus:outline-none focus:ring-2 focus:ring-blue-400
                                       text-sm"
                                prop:value=move || server_ip.get()
                                on:input=move |ev| {
                                    server_ip.set(event_target_value(&ev));
                                    test_result.set(None);
                                }
                            />
                        </div>

                        <div>
                            <label class="block text-sm font-medium
                                          text-slate-700 dark:text-slate-300 mb-1">
                                "Port"
                            </label>
                            <input
                                type="number"
                                min="1024"
                                max="65535"
                                class="w-full px-3 py-2 rounded-lg border
                                       border-slate-300 dark:border-slate-600
                                       bg-white dark:bg-slate-700
                                       text-slate-900 dark:text-slate-100
                                       focus:outline-none focus:ring-2 focus:ring-blue-400
                                       text-sm"
                                prop:value=move || server_port.get().to_string()
                                on:input=move |ev| {
                                    if let Ok(p) = event_target_value(&ev).parse::<u16>() {
                                        server_port.set(p);
                                    }
                                }
                            />
                        </div>

                        // Bouton de test
                        <button
                            class="w-full py-2 px-4 rounded-lg border
                                   border-slate-300 dark:border-slate-600
                                   text-slate-700 dark:text-slate-300
                                   hover:bg-slate-100 dark:hover:bg-slate-700
                                   transition-colors text-sm font-medium
                                   disabled:opacity-50 disabled:cursor-not-allowed"
                            disabled=move || testing.get() || simulating.get()
                            on:click=on_test
                        >
                            {move || if testing.get() {
                                "Test en cours…".to_string()
                            } else {
                                "🔌 Tester la connexion".to_string()
                            }}
                        </button>

                        // Bouton simulation locale
                        <button
                            class="w-full py-2 px-4 rounded-lg border
                                   border-amber-400 dark:border-amber-600
                                   text-amber-700 dark:text-amber-300
                                   hover:bg-amber-50 dark:hover:bg-amber-900/20
                                   transition-colors text-sm font-medium
                                   disabled:opacity-50 disabled:cursor-not-allowed"
                            disabled=move || testing.get() || simulating.get()
                            on:click=on_simulate
                        >
                            {move || if simulating.get() {
                                "Démarrage du serveur local…".to_string()
                            } else {
                                "🧪 Simulation (ce PC uniquement)".to_string()
                            }}
                        </button>

                        // Résultat du test
                        {move || match test_result.get() {
                            Some(true)  => view! {
                                <p class="text-sm text-green-600 dark:text-green-400 text-center font-medium">
                                    "✅ Connexion réussie !"
                                </p>
                            }.into_any(),
                            Some(false) => view! {
                                <p class="text-sm text-red-500 dark:text-red-400 text-center">
                                    "❌ Impossible de joindre le serveur. Vérifiez l'IP, le port et que le serveur est démarré."
                                </p>
                            }.into_any(),
                            None => view! { <span /> }.into_any(),
                        }}
                    </div>
                </Show>

                // ── Erreur générale ───────────────────────────────────────────
                {move || error_msg.get().map(|msg| view! {
                    <p class="text-sm text-red-500 dark:text-red-400 text-center">{msg}</p>
                })}

                // ── Bouton Valider ────────────────────────────────────────────
                <button
                    class="w-full py-3 px-6 rounded-xl font-semibold text-white
                           bg-blue-600 hover:bg-blue-700 active:bg-blue-800
                           transition-colors duration-200
                           disabled:opacity-50 disabled:cursor-not-allowed
                           shadow-md"
                    disabled=move || saving.get()
                    on:click=on_save
                >
                    {move || if saving.get() {
                        "Démarrage…".to_string()
                    } else {
                        "✔ Valider et démarrer".to_string()
                    }}
                </button>
            </div>
        </div>
    }
}
