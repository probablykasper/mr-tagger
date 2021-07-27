use tauri::command;

#[command]
pub fn example() -> String {
  return "Hello world".to_string();
}
