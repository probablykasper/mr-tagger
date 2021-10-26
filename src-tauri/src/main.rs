#![cfg_attr(
  all(not(debug_assertions), target_os = "windows"),
  windows_subsystem = "windows"
)]

use crate::cmd::AppArg;
use crate::menu::AddDefaultSubmenus;
use std::thread;
use tauri::api::{dialog, shell};
use tauri::{CustomMenuItem, Manager, Menu, MenuItem, Submenu, WindowBuilder, WindowUrl};

mod cmd;
mod files;
mod frames;
mod image;
mod menu;

#[macro_export]
macro_rules! throw {
  ($($arg:tt)*) => {{
    return Err(format!($($arg)*))
  }};
}

fn custom_item(name: &str) -> CustomMenuItem {
  let c = CustomMenuItem::new(name.to_string(), name);
  return c;
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
    .menu(
      Menu::new()
        .add_default_app_submenu(&ctx.package_info().name)
        .add_submenu(Submenu::new(
          "File",
          Menu::new()
            .add_item(custom_item("Open...").accelerator("cmdOrControl+O"))
            .add_native_item(MenuItem::Separator)
            .add_item(custom_item("Close").accelerator("cmdOrControl+W"))
            .add_item(custom_item("Save").accelerator("cmdOrControl+S"))
            .add_item(custom_item("Save As...").accelerator("shift+cmdOrControl+S")),
        ))
        .add_default_edit_submenu()
        .add_default_view_submenu()
        .add_default_window_submenu()
        .add_submenu(Submenu::new(
          "Help",
          Menu::new().add_item(custom_item("Learn More")),
        )),
    )
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
