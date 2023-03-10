#![cfg_attr(
  all(not(debug_assertions), target_os = "windows"),
  windows_subsystem = "windows"
)]

mod app;
mod conf;
mod utils;

use app::{cmd, fs_extra, gpt, menu, setup, window};
use conf::ChatConfJson;
use tauri::api::path;
use tauri_plugin_autostart::MacosLauncher;
use tauri_plugin_log::{
  fern::colors::{Color, ColoredLevelConfig},
  LogTarget,
};

#[tokio::main]
async fn main() {
  ChatConfJson::init();
  // If the file does not exist, creating the file will block menu synchronization
  utils::create_chatgpt_prompts();
  let context = tauri::generate_context!();
  let colors = ColoredLevelConfig {
    error: Color::Red,
    warn: Color::Yellow,
    debug: Color::Blue,
    info: Color::BrightGreen,
    trace: Color::Cyan,
  };

  gpt::download_list("chat.download.json", "download", None, None);
  gpt::download_list("chat.notes.json", "notes", None, None);

  let chat_conf = ChatConfJson::get_chat_conf();

  let mut builder = tauri::Builder::default()
    // https://github.com/tauri-apps/tauri/pull/2736
    .plugin(
      tauri_plugin_log::Builder::default()
        .targets([
          // LogTarget::LogDir,
          // LOG PATH: ~/.chatgpt/ChatGPT.log
          LogTarget::Folder(path::home_dir().unwrap().join(".chatgpt")),
          LogTarget::Stdout,
          LogTarget::Webview,
        ])
        .level(log::LevelFilter::Debug)
        .with_colors(colors)
        .build(),
    )
    .plugin(tauri_plugin_positioner::init())
    .plugin(tauri_plugin_autostart::init(
      MacosLauncher::LaunchAgent,
      None,
    ))
    .invoke_handler(tauri::generate_handler![
      cmd::drag_window,
      cmd::fullscreen,
      cmd::download,
      cmd::save_file,
      cmd::open_link,
      cmd::get_chat_conf,
      cmd::get_theme,
      cmd::reset_chat_conf,
      cmd::run_check_update,
      cmd::form_cancel,
      cmd::form_confirm,
      cmd::form_msg,
      cmd::open_file,
      cmd::get_data,
      gpt::get_chat_model_cmd,
      gpt::parse_prompt,
      gpt::sync_prompts,
      gpt::sync_user_prompts,
      gpt::cmd_list,
      gpt::download_list,
      gpt::get_download_list,
      window::wa_window,
      window::control_window,
      window::window_reload,
      window::dalle2_search_window,
      fs_extra::metadata,
    ])
    .setup(setup::init)
    .menu(menu::init());

  if chat_conf.tray {
    builder = builder.system_tray(menu::tray_menu());
  }

  builder
    .on_menu_event(menu::menu_handler)
    .on_system_tray_event(menu::tray_handler)
    .on_window_event(|event| {
      // https://github.com/tauri-apps/tauri/discussions/2684
      if let tauri::WindowEvent::CloseRequested { api, .. } = event.event() {
        let win = event.window();
        if win.label() == "core" {
          // TODO: https://github.com/tauri-apps/tauri/issues/3084
          // event.window().hide().unwrap();
          // https://github.com/tauri-apps/tao/pull/517
          #[cfg(target_os = "macos")]
          event.window().minimize().unwrap();

          // fix: https://github.com/lencx/ChatGPT/issues/93
          #[cfg(not(target_os = "macos"))]
          event.window().hide().unwrap();
        } else {
          win.close().unwrap();
        }
        api.prevent_close();
      }
    })
    .run(context)
    .expect("error while running ChatGPT application");
}
