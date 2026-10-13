// SPDX-FileCopyrightText: 2025 Simon Repp
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::net::{IpAddr, Ipv4Addr};
use std::path::Path;
use std::sync::{Arc, Mutex};

use indoc::formatdoc;
use local_ip_address::local_ip;
use tauri::{
    AppHandle,
    Emitter,
    Manager,
    Url,
    WebviewUrl,
    WebviewWindow,
    WebviewWindowBuilder,
    WindowEvent
};
use tauri_plugin_dialog::{
    DialogExt,
    MessageDialogButtons,
    MessageDialogKind
};

use hyper8_core::Context;
use hyper8_core::watcher::watch_site_dir;
use hyper8_translations::Translations;
use hyper8_version::VERSION_ADAPTIVE_PATCH;
use hyper8_web::ServerMode;

use crate::{EditorMode, LauncherState};
use crate::EXIT_CODE_SUCCESS;

/// Make sure site_dir definitely exists when you call this
pub fn start_editor(
    app: AppHandle,
    launcher_window: WebviewWindow,
    editor_mode: EditorMode,
    site_dir: &Path,
    translations: Translations,
) {
    let context = Context::new(
        None,
        None,
        None,
        site_dir
    );

    if context.ffmpeg.is_none() {
        let message = format!("\
            FFmpeg could not be found. This is likely a problem with your \
            installation. If you built Hyper 8 yourself, make sure to \
            install ffmpeg (including ffprobe) on your system. You can also compile hyper8 with \
            --features use-bundled-ffmpeg and place an ffmpeg and ffprobe \
            executable next to the hyper8 executable if you prefer not to \
            install ffmpeg system-wide.
        ");

        app
            .dialog()
            .message(message)
            .kind(MessageDialogKind::Info)
            .title(translations.error)
            .buttons(MessageDialogButtons::Ok)
            .show(move |_| { });

        return;
    }

    watch_site_dir(Arc::clone(&context));

    let ip = if editor_mode == EditorMode::Collaborative {
        match local_ip() {
            Ok(ip) => ip,
            // TODO: In this case the information that the address
            // (0.0.0.0) can be shared with and used by others is obviously
            // wrong, so the hint should show something different then.
            // However, chances are that the setup is unusable in this case
            // anyway (which would be the local ip if we can't determine
            // it?), so maybe we should just treat this as a hard error and
            // return that to the user.
            Err(_) => IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0))
        }
    } else {
        IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1))
    };

    let state = app.state::<Arc<Mutex<LauncherState>>>();

    let server_mode = {
        let launcher_state = state.lock().unwrap();

        match editor_mode {
            EditorMode::Collaborative => {
                let inherited_editor_state = launcher_state.editor_state.lock().unwrap().clone();
                ServerMode::MultipleRemoteUsers { inherited_editor_state }
            }
            EditorMode::BrowserBased |
            EditorMode::Standalone => {
                let synced_editor_state = launcher_state.editor_state.clone();
                ServerMode::SingleLocalUser { synced_editor_state }
            }
        }
    };

    let result = hyper8_web::start_server(
        context,
        Some(ip),
        None,
        server_mode
    );

    match result {
        Ok(url) => {
            if editor_mode == EditorMode::Standalone {
                launcher_window.hide().unwrap();

                let webview_url = WebviewUrl::External(Url::parse(&url).unwrap());

                let window = WebviewWindowBuilder::new(&app, "editor", webview_url)
                    .enable_clipboard_access() // (possibly not needed, probably can't hurt either)
                    .title(&format!("Hyper8 {VERSION_ADAPTIVE_PATCH} – {}", site_dir.display()))
                    .build()
                    .unwrap();

                window.on_window_event(move |event: &WindowEvent| {
                    if let WindowEvent::Destroyed = event {
                        app.exit(EXIT_CODE_SUCCESS);
                    }
                });

                let _ = window.maximize();
            } else {
                let editor_note = if editor_mode == EditorMode::Collaborative {
                    translations.collaborative_editor_note
                } else {
                    translations.local_editor_note
                };

                let t_shut_down_editor_and_quit = translations.shut_down_editor_and_quit;
                let t_the_editor_is_available_at_xxx = translations
                    .the_editor_is_available_at_xxx(&url);

                let copy_icon = hyper8_icons::copy(Some(translations.copy));
                let failure_icon = hyper8_icons::failure(translations.failed);
                let success_icon = hyper8_icons::success(translations.copied);

                let content = formatdoc!(r#"
                    <div>
                        <span>{t_the_editor_is_available_at_xxx}</span>
                        <button data-url="{url}">{copy_icon}</button>
                    </div>
                    <div>
                        {editor_note}
                    </div>
                    <button class="quit">{t_shut_down_editor_and_quit}</button>
                    <template data-copy-icon>{copy_icon}</template>
                    <template data-failed-icon>{failure_icon}</template>
                    <template data-success-icon>{success_icon}</template>
                "#);

                let _ = app.emit("show-overlay", content);

                let _ = tauri_plugin_opener::open_url(url, None::<&str>);
            };
        }
        Err(err) => {
            app
                .dialog()
                .message(err)
                .kind(MessageDialogKind::Info)
                .title(translations.error)
                .buttons(MessageDialogButtons::Ok)
                .show(move |_| { });
        }
    }
}
