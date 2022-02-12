#![cfg_attr(
  all(not(debug_assertions), target_os = "windows"),
  windows_subsystem = "windows"
)]

use crate::cmd::AppArg;
use std::collections::HashMap;
use std::thread;
use tauri::api::{dialog, shell};
use tauri::regex::Regex;
use tauri::{
  scope, CustomMenuItem, Manager, Menu, MenuEntry, MenuItem, Submenu, WindowBuilder, WindowUrl,
};

mod cmd;
mod files;
mod frames;
mod image;

#[macro_export]
macro_rules! throw {
  ($($arg:tt)*) => {{
    return Err(format!($($arg)*))
  }};
}

fn main() {
  let ctx = tauri::generate_context!();
  let tauri_app = tauri::Builder::default()
    .invoke_handler(tauri::generate_handler![
      cmd::error_popup,
      cmd::get_app,
      cmd::show,
      cmd::close_window,
      cmd::get_page,
      files::open_files,
      files::close_file,
      files::save_file,
      image::get_image,
      image::remove_image,
      image::set_image,
    ])
    .create_window("main", WindowUrl::default(), |win, webview| {
      let win = win
        .title("Mr Tagger")
        .resizable(true)
        .decorations(true)
        .always_on_top(false)
        .inner_size(800.0, 550.0)
        .min_inner_size(400.0, 200.0)
        .skip_taskbar(false)
        .fullscreen(false);
      return (win, webview);
    })
    .manage(cmd::AppState(Default::default()))
    .menu(Menu::with_items([
      #[cfg(target_os = "macos")]
      MenuEntry::Submenu(Submenu::new(
        &ctx.package_info().name,
        Menu::with_items([
          MenuItem::About(ctx.package_info().name.clone()).into(),
          MenuItem::Separator.into(),
          MenuItem::Services.into(),
          MenuItem::Separator.into(),
          MenuItem::Hide.into(),
          MenuItem::HideOthers.into(),
          MenuItem::ShowAll.into(),
          MenuItem::Separator.into(),
          MenuItem::Quit.into(),
        ]),
      )),
      MenuEntry::Submenu(Submenu::new(
        "File",
        Menu::with_items([
          CustomMenuItem::new("Open...", "Open...")
            .accelerator("cmdOrControl+O")
            .into(),
          MenuItem::Separator.into(),
          CustomMenuItem::new("Close", "Close")
            .accelerator("cmdOrControl+W")
            .into(),
          CustomMenuItem::new("Save", "Save")
            .accelerator("cmdOrControl+S")
            .into(),
          CustomMenuItem::new("Save As...", "Save As...")
            .accelerator("shift+cmdOrControl+S")
            .into(),
        ]),
      )),
      MenuEntry::Submenu(Submenu::new(
        "Edit",
        Menu::with_items([
          MenuItem::Undo.into(),
          MenuItem::Redo.into(),
          MenuItem::Separator.into(),
          MenuItem::Cut.into(),
          MenuItem::Copy.into(),
          MenuItem::Paste.into(),
          #[cfg(not(target_os = "macos"))]
          MenuItem::Separator.into(),
          MenuItem::SelectAll.into(),
        ]),
      )),
      MenuEntry::Submenu(Submenu::new(
        "View",
        Menu::with_items([MenuItem::EnterFullScreen.into()]),
      )),
      MenuEntry::Submenu(Submenu::new(
        "Window",
        Menu::with_items([MenuItem::Minimize.into(), MenuItem::Zoom.into()]),
      )),
      // You should always have a Help menu on macOS because it will automatically
      // show a menu search field
      MenuEntry::Submenu(Submenu::new(
        "Help",
        Menu::with_items([CustomMenuItem::new("Learn More", "Learn More").into()]),
      )),
    ]))
    .on_menu_event(|event| {
      let event_name = event.menu_item_id();
      event.window().emit("menu", event_name).unwrap();
      match event_name {
        "Learn More" => {
          let shell_scope = scope::ShellScope::new(scope::ShellScopeConfig {
            open: Some(Regex::new("^https?://").unwrap()),
            scopes: HashMap::new(),
          });
          let link = "https://github.com/probablykasper/mr-tagger".to_string();
          shell::open(&shell_scope, link, None).unwrap();
        }
        _ => {}
      }
    })
    .build(ctx)
    .expect("error while running tauri app");
  tauri_app.run(|app_handle, e| match e {
    tauri::RunEvent::CloseRequested { label, api, .. } => {
      if label == "main" {
        let app: AppArg<'_> = app_handle.state();
        let app = app.0.lock().unwrap();
        for file in &app.files {
          if file.dirty {
            api.prevent_close();
            let app_handle = app_handle.clone();
            thread::spawn(move || {
              let w = app_handle.get_window("main").unwrap();
              dialog::ask(
                Some(&w),
                "You have unsaved changes. Close without saving?",
                "",
                move |res| {
                  let w = app_handle.get_window("main").unwrap();
                  if res == true {
                    w.close().unwrap();
                  }
                },
              );
            });
            break;
          }
        }
      }
    }
    _ => {}
  })
}
