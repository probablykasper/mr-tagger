#![cfg_attr(
  all(not(debug_assertions), target_os = "windows"),
  windows_subsystem = "windows"
)]

use crate::cmd::AppArg;
use std::thread;
use tauri::api::{dialog, shell};
use tauri::{CustomMenuItem, Manager, Menu, MenuItem, Submenu, WindowBuilder, WindowUrl};

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
  fn custom_menu(name: &str) -> CustomMenuItem {
    let c = CustomMenuItem::new(name.to_string(), name);
    return c;
  }
  let menu = Menu::new()
    .add_submenu(Submenu::new(
      // on macOS first menu is always app name
      "Mr Tagger",
      Menu::new()
        .add_native_item(MenuItem::About("Mr Tagger".to_string()))
        .add_native_item(MenuItem::Separator)
        .add_native_item(MenuItem::Services)
        .add_native_item(MenuItem::Separator)
        .add_native_item(MenuItem::Hide)
        .add_native_item(MenuItem::HideOthers)
        .add_native_item(MenuItem::ShowAll)
        .add_native_item(MenuItem::Separator)
        .add_native_item(MenuItem::Quit),
    ))
    .add_submenu(Submenu::new(
      "File",
      Menu::new()
        .add_item(custom_menu("Open...").accelerator("cmdOrControl+O"))
        .add_native_item(MenuItem::Separator)
        .add_item(custom_menu("Close").accelerator("cmdOrControl+W"))
        .add_item(custom_menu("Save").accelerator("cmdOrControl+S"))
        .add_item(custom_menu("Save As...").accelerator("shift+cmdOrControl+S")),
    ))
    .add_submenu(Submenu::new("Edit", {
      let mut menu = Menu::new();
      menu = menu.add_native_item(MenuItem::Undo);
      menu = menu.add_native_item(MenuItem::Redo);
      menu = menu.add_native_item(MenuItem::Separator);
      menu = menu.add_native_item(MenuItem::Cut);
      menu = menu.add_native_item(MenuItem::Copy);
      menu = menu.add_native_item(MenuItem::Paste);
      #[cfg(not(target_os = "macos"))]
      {
        menu = menu.add_native_item(MenuItem::Separator);
      }
      menu = menu.add_native_item(MenuItem::SelectAll);
      menu
    }))
    .add_submenu(Submenu::new("View", Menu::new()))
    .add_submenu(Submenu::new(
      "Window",
      Menu::new()
        .add_native_item(MenuItem::Minimize)
        .add_native_item(MenuItem::Zoom),
    ))
    .add_submenu(Submenu::new(
      "Help",
      Menu::new().add_item(custom_menu("Learn More")),
    ))
    .add_native_item(MenuItem::Copy);

  let ctx = tauri::generate_context!();
  let tauri_app = tauri::Builder::default()
    .invoke_handler(tauri::generate_handler![
      cmd::error_popup,
      cmd::get_app,
      cmd::show,
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
        .transparent(false)
        .decorations(true)
        .always_on_top(false)
        .inner_size(800.0, 550.0)
        .min_inner_size(400.0, 200.0)
        .skip_taskbar(false)
        .fullscreen(false);
      return (win, webview);
    })
    .manage(cmd::AppState(Default::default()))
    .menu(menu)
    .on_menu_event(|event| {
      let event_name = event.menu_item_id();
      event.window().emit("menu", event_name).unwrap();
      match event_name {
        "Learn More" => {
          shell::open(
            "https://github.com/probablykasper/mr-tagger".to_string(),
            None,
          )
          .unwrap();
        }
        _ => {}
      }
    })
    .build(ctx)
    .expect("error while running tauri app");
  tauri_app.run(|app_handle, e| match e {
    tauri::Event::CloseRequested { label, api, .. } => {
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
