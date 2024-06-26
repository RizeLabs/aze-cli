use crate::actions;
use aze_lib::gamestate::Check_Action;
use aze_lib::utils::{ Ws_config, Player };
use aze_lib::{
    constants::{BUY_IN_AMOUNT, NO_OF_PLAYERS, SMALL_BLIND_AMOUNT, PLAYER_FILE_PATH},
    utils::validate_action,
};
use aze_types::actions::{ActionType, GameActionResponse};
use clap::{Parser, ValueEnum};
use dialoguer::{Input, Select};
use std::fs::File;
use std::io::Write;
use std::path::Path;

#[derive(Debug, Clone, Parser)]
pub struct ActionCmd {}

impl ActionCmd {
    pub async fn execute(&self, ws_config_path: &std::path::PathBuf) -> Result<(), String> {
        let (playerid, gameid) = get_or_prompt_ids();

        let action_type = Select::new()
            .with_prompt("What is your action type?")
            .item("Raise")
            .item("Small Blind")
            .item("Big Blind")
            .item("Call")
            .item("Check")
            .item("Fold")
            .interact()
            .expect("Failed to get action type");

        let action_type = match action_type {
            0 => ActionType::Raise,
            1 => ActionType::SmallBlind,
            2 => ActionType::BigBlind,
            3 => ActionType::Call,
            4 => ActionType::Check,
            5 => ActionType::Fold,
            _ => panic!("Invalid action type selected"),
        };

        let amount = if action_type == ActionType::Raise {
            let amount: u8 = Input::<String>::new()
                .with_prompt("What is the raise amount?")
                .interact()
                .expect("Failed to get amount")
                .parse()
                .expect("Invalid amount");

            Some(amount)
        } else {
            None
        };

        match send_action(playerid, gameid, action_type, amount, ws_config_path).await {
            Ok(_) => {
                println!("Action performed successfully");
                Ok(())
            }
            Err(e) => Err(format!("{}", e)),
        }
    }
}

async fn send_action(
    player_id: u64,
    game_id: u64,
    action_type: ActionType,
    amount: Option<u8>,
    ws_config_path: &std::path::PathBuf
) -> Result<GameActionResponse, String> {
    let amount_u64 = amount.map(|value| value as u64);
    let ws_url = Ws_config::load(ws_config_path).url.unwrap();
    let result = validate_action(
        Check_Action {
            action_type,
            amount: amount_u64,
        },
        ws_url,
        player_id,
        game_id.clone(),
    )
    .await.unwrap();
    if result == false {
        return Err("Invalid Action".to_string());
    }
    match action_type {
        ActionType::Raise => actions::raise(player_id, game_id, amount, ws_config_path).await,
        ActionType::SmallBlind => actions::small_blind(player_id, game_id, ws_config_path).await,
        ActionType::BigBlind => actions::big_blind(player_id, game_id, ws_config_path).await,
        ActionType::Call => actions::call(player_id, game_id, ws_config_path).await,
        ActionType::Check => actions::check(player_id, game_id, ws_config_path).await,
        ActionType::Fold => actions::fold(player_id, game_id, ws_config_path).await,
    }
}

fn get_or_prompt_ids() -> (u64, u64) {
    let path = Path::new(PLAYER_FILE_PATH);
    let mut player_id: u64 = 0;
    let mut identifier: String = "".to_string();
    if path.exists() {
        let player: Player = toml::from_str(
            &std::fs::read_to_string(path).expect("Failed to read Player.toml file"),
        )
        .expect("Failed to parse Player.toml file");

        if let Some(game_id) = player.game_id() {
            return (player.player_id(), game_id);
        }
        else {
            player_id = player.player_id();
            identifier = player.identifier();
        }
    }
    else {
        player_id = Input::<String>::new()
            .with_prompt("What is your player id?")
            .interact()
            .expect("Failed to get player id")
            .parse()
            .expect("Invalid player id");
    }

    let game_id: u64 = Input::<String>::new()
        .with_prompt("What is the game id?")
        .interact()
        .expect("Failed to get game id")
        .parse()
        .expect("Invalid game id");

    let player = Player::new(player_id, identifier, Some(game_id));
    let toml_string = toml::to_string(&player).expect("Failed to serialize player data");
    let mut file = File::create(&path).expect("Failed to create Player.toml file");
    file.write_all(toml_string.as_bytes())
        .expect("Failed to write player data to Player.toml file");

    (player_id, game_id)
}