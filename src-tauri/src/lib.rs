#![allow(unstable_name_collisions)]
use std::{collections::HashMap, sync::RwLock};

use tauri::{App, api::http::ClientBuilder};
use itertools::Itertools;
use mdns_sd::{ServiceDaemon, ServiceEvent};

#[cfg(mobile)]
mod mobile;
#[cfg(mobile)]
pub use mobile::*;

pub type SetupHook = Box<dyn FnOnce(&mut App) -> Result<(), Box<dyn std::error::Error>> + Send>;

#[derive(Default)]
pub struct AppBuilder {
  setup: Option<SetupHook>,
}

impl AppBuilder {
  pub fn new() -> Self {
    Self::default()
  }

  #[must_use]
  pub fn setup<F>(mut self, setup: F) -> Self
  where
    F: FnOnce(&mut App) -> Result<(), Box<dyn std::error::Error>> + Send + 'static,
  {
    self.setup.replace(Box::new(setup));
    self
  }

  pub fn run(self) {
    let setup = self.setup;
    tauri::Builder::default()
      .setup(move |app| {
        if let Some(setup) = setup {
          (setup)(app)?;
        }
        Ok(())
      })
      .manage(AppState::default())
      .invoke_handler(tauri::generate_handler![
        connect,
        send_color,
        get_status,
        send_cycle,
        send_reset,
        send_wifi_credentials,
      ])
      .run(tauri::generate_context!())
      .expect("error while running tauri application");
  }
}

// MY CODE BELOW
#[derive(Default)]
struct AppState {
    ip: RwLock<String>,
}

#[tauri::command]
async fn connect(state: tauri::State<'_, AppState>) -> Result<(), String> {
    let ip = find_lamp_ip().await?;
    *state.ip.write().unwrap() = ip;

    Ok(())
}

#[tauri::command]
async fn get_status(state: tauri::State<'_, AppState>) -> Result<String, String> {
    let lamp_ip = state.ip.read().unwrap().clone();

    let url = format!("http://{lamp_ip}/status");
    let status = reqwest::get(url).await
        .map_err(|err| format!("Failed to get status from lamp: {err:#?}"))?
        .text()
        .await
        .map_err(|err| format!("Failed to read text response: {err:#?}"))?;

    Ok(status)
}

#[tauri::command]
async fn send_color(state: tauri::State<'_, AppState>, components: HashMap<&str, f32>) -> Result<(), String> {
    let lamp_ip = state.ip.read().unwrap().clone();
    
    let query_params: String = components.into_iter()
        .map(|(key, value)| format!("{key}={value}"))
        .intersperse("&".to_string())
        .collect();

    let url = format!("http://{lamp_ip}/set?{query_params}");
    reqwest::Client::new().post(url).send().await
        .map_err(|err| format!("Failed to send color to the lamp: {err:#?}"))?;

    Ok(())
}

#[tauri::command]
async fn send_cycle(state: tauri::State<'_, AppState>) -> Result<String, String> {
    let lamp_ip = state.ip.read().unwrap().clone();

    let url = format!("http://{lamp_ip}/cycle");
    let status = reqwest::get(url).await
        .map_err(|err| format!("Failed to send cycle command: {err:#?}"))?
        .text()
        .await
        .map_err(|err| format!("Failed to read text response: {err:#?}"))?;

    Ok(status)
}

#[tauri::command]
async fn send_reset(state: tauri::State<'_, AppState>) -> Result<(), String> {
    let lamp_ip = state.ip.read().unwrap().clone();

    let url = format!("http://{lamp_ip}/reset");
    reqwest::Client::new().post(url).send().await
        .map_err(|err| format!("Failed to send cycle: {err:#?}"))?;

    Ok(())
}

#[tauri::command]
async fn send_wifi_credentials(ssid: String, password: String) -> Result<(), String> {
    let url = format!("http://192.168.71.1/connect?ssid={ssid}&password={password}");
    reqwest::get(url).await
        .map_err(|err| format!("Failed to send wifi credentials: {err:#?}"))?;

    Ok(())
}

async fn find_lamp_ip() -> Result<String, String> {
    let mdns = ServiceDaemon::new()
        .map_err(|err| format!("Failed to create mdns deamon: {err:#?}"))?;

    let service_type = "_lamp._tcp.local.";
    let receiver = mdns.browse(service_type)
        .map_err(|err| format!("Failed to create mdns browser: {err:#?}"))?;

    let result = loop {
        match receiver.recv_async().await {
            Ok(ServiceEvent::ServiceResolved(info)) => {
                let ip = info.get_addresses()
                    .into_iter()
                    .cloned()
                    .next()
                    .expect("The advertised lamp service had no IP address");

                break Ok(ip.to_string())
            },
            Ok(event) => println!("Found mDNS event: {event:#?}"),
            Err(err) => break Err(format!("Mdns browser error: {err:#?}")),
        }
    };

    mdns.shutdown().ok();

    result
}