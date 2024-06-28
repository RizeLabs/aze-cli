use clap::Parser;
use std::error::Error;
use std::path::PathBuf;
use futures_util::{StreamExt, SinkExt}; // Import the required traits
use tokio_tungstenite::connect_async;
use tokio_tungstenite::tungstenite::protocol::Message;
use aze_lib::utils::{ Player, Ws_config };
use aze_lib::constants::PLAYER_FILE_PATH;
use ansi_term::Colour::{Blue, Green, Red, Yellow};
use miden_objects::accounts::{Account, AccountId};
use std::fs::{self, OpenOptions};
use std::io::{Read, Write};
use toml::Value;

#[derive(Debug, Clone, Parser)]
pub struct ConnectCmd {
    #[arg(short, long, help = "WebSocket server URL")]
    url: String,
}

impl ConnectCmd {
    pub async fn execute(&self, config_path: &PathBuf) -> Result<(), Box<dyn Error>> {
        // Connect to the WebSocket server
        let (ws_stream, _) = connect_async(&self.url).await?;
        let mut config = Ws_config::load(config_path);
        config.url = Some(self.url.to_string());
        config.save(config_path);

        // add game id to Player.toml
        let url = url::Url::parse(&self.url).unwrap();
        let game_id_hex = url
            .path_segments()
            .and_then(|segments| segments.last())
            .map(|s| s.to_string())
            .ok_or_else(|| "Delimiter not found in URL")
            .unwrap();
        let game_id: u64 = AccountId::from_hex(&game_id_hex).unwrap().into();
        let config_str = fs::read_to_string(PLAYER_FILE_PATH)?;
        let mut config: Player = toml::from_str(&config_str)?;
        config.set_game_id(game_id);
        let updated_config_str = toml::to_string(&config)?;
        fs::write(PLAYER_FILE_PATH, updated_config_str)?;

        println!("Connected to the game server at {}", self.url);

        let (mut _write, mut read) = ws_stream.split();

        // Read messages from the server
        while let Some(message) = read.next().await {
            match message {
                Ok(msg) => match msg {
                    Message::Text(text) => println!("{} {}", Yellow.bold().paint("Game Update: "), text),
                    _ => (),
                },
                Err(e) => {
                    eprintln!("{}", Red.bold().paint(format!("Error receiving message: {}", e)));
                    break;
                }
            }
        }

        Ok(())
    }
}
