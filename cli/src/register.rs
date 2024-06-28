use crate::accounts::create_aze_player_account;
use aze_lib::utils::Player;
use aze_lib::constants::PLAYER_FILE_PATH;
use aze_types::accounts::AccountCreationError;
use clap::Parser;
use miden_objects::accounts::AccountId;
use std::fs::File;
use std::io::Write;
use std::path::Path;

#[derive(Debug, Clone, Parser)]
pub struct RegisterCmd {
    #[arg(short, long)]
    identifier: String,
}

impl RegisterCmd {
    pub async fn execute(&self) -> Result<(), String> {
        match create_aze_player_account(self.identifier.clone()).await {
            Ok(account_id) => {
                println!("Player account created: {:?}", account_id);
                let player = Player::new(account_id.clone().into(), self.identifier.clone(), None);
                let toml_string =
                    toml::to_string(&player).expect("Failed to serialize player data");
                let path = Path::new(PLAYER_FILE_PATH);
                let mut file = File::create(&path).expect("Failed to create Player.toml file");
                file.write_all(toml_string.as_bytes())
                    .expect("Failed to write player data to Player.toml file");
                Ok(())
            }
            Err(e) => Err(format!("Error creating player account: {}", e)),
        }
    }
}
