use crate::accounts::{ create_aze_game_account, consume_game_notes, send_community_cards };
use aze_lib::client::{ create_aze_client, AzeClient };
use aze_lib::broadcast::initialise_server;
use aze_lib::constants::{
    BUY_IN_AMOUNT,
    COMMUNITY_CARDS,
    CURRENT_PHASE_SLOT,
    NO_OF_PLAYERS,
    SMALL_BLIND_AMOUNT,
};
use aze_lib::utils::{ broadcast_message, card_from_number, Ws_config };
use aze_types::accounts::AccountCreationError;
use clap::{ Parser, ValueEnum };
use figment::{ providers::{ Format, Toml }, Figment };
use miden_objects::{ accounts::AccountId, Felt, FieldElement };
use serde::Deserialize;
use std::path::PathBuf;
use tokio::task::LocalSet;
use tokio::time::{ sleep, Duration };

#[derive(ValueEnum, Debug, Clone)]
enum GameType {
    Holdem,
    Omaha,
    Hilo,
}

#[derive(Debug, Clone, Parser)]
pub struct InitCmd {
    #[arg(short, long, value_enum, default_value = "holdem")]
    game_type: GameType,

    #[arg(short, long, num_args = NO_OF_PLAYERS as usize)]
    player: Option<Vec<u64>>,

    #[arg(short, long, default_value_t = SMALL_BLIND_AMOUNT)]
    small_blind: u8,

    #[arg(short, long, default_value_t = BUY_IN_AMOUNT)]
    buy_in: u64,

    #[arg(short, long, value_parser)]
    config: Option<std::path::PathBuf>,
}

impl InitCmd {
    pub async fn execute(&self, ws_config: &PathBuf) -> Result<(), String> {
        let mut player_ids = self.player.clone().unwrap_or_else(Vec::new);
        let mut small_blind_amount = self.small_blind;
        let mut buy_in_amount = self.buy_in;

        if let Some(config_path) = &self.config {
            match load_config(&config_path) {
                Ok(config) => {
                    player_ids = config.player_ids;
                    small_blind_amount = config.small_blind;
                    buy_in_amount = config.buy_in;
                }
                Err(e) => {
                    return Err(format!("Error loading config: {}", e));
                }
            }
        }

        match create_aze_game_account(player_ids.clone(), small_blind_amount, buy_in_amount).await {
            Ok(game_account_id) => {
                println!("Game account created: {:?}", game_account_id);
                /*
                    Start ws and http server on exposed port of user in background
                    Setup local off chain game state 
                */
                let config_clone = ws_config.clone();
                let player_ids_clone = player_ids.clone();
                tokio::spawn(async move {
                    match
                        initialise_server(
                            game_account_id.to_string(),
                            &config_clone,
                            buy_in_amount.clone(),
                            small_blind_amount.clone(),
                            player_ids_clone
                        )
                    {
                        Some(ws_url) => {
                            println!("Game server started at: {}", ws_url);
                            Ok(())
                        }
                        None => {
                            return Err("Error starting ws server");
                        }
                    }
                });
                let mut client: AzeClient = create_aze_client();
                let local_set = LocalSet::new();
                let mut ws_url: String = String::new();

                match Ws_config::load(ws_config).url {
                    Some(url) => {
                        ws_url = url;
                    }

                    None => {
                        eprintln!("Ws_config DNE, use init or connect command before action");
                    }
                }
                local_set.run_until(async {
                    loop {
                        let (game_account, _) = client.get_account(game_account_id).unwrap();
                        let phase_data = game_account
                            .storage()
                            .get_item(CURRENT_PHASE_SLOT)
                            .as_elements()
                            .to_vec();
                        let pre_phase = phase_data[0].as_int();
                        let mut pre_community_cards: [[Felt; 4]; 5] = [[Felt::ZERO; 4]; 5];
                        for i in 0..5 {
                            let card_digest = game_account.storage().get_item(COMMUNITY_CARDS[i]);
                            pre_community_cards[i] = card_digest.into();
                        }

                        consume_game_notes(game_account_id).await;

                        let (game_account, _) = client.get_account(game_account_id).unwrap();
                        let phase_data = game_account
                            .storage()
                            .get_item(CURRENT_PHASE_SLOT)
                            .as_elements()
                            .to_vec();
                        let phase = phase_data[0].as_int();
                        let mut community_cards: [[Felt; 4]; 5] = [[Felt::ZERO; 4]; 5];
                        for i in 0..5 {
                            let card_digest = game_account.storage().get_item(COMMUNITY_CARDS[i]);
                            community_cards[i] = card_digest.into();
                        }

                        // broadcast cards that have changed due to unmasking
                        for (i, cards) in community_cards.iter().enumerate() {
                            if pre_community_cards[i] != *cards {
                                let card = card_from_number(cards[0].into());
                                let _ = broadcast_message(
                                    game_account_id.clone().to_string(),
                                    ws_url.clone(),
                                    format!("Community card {} --> {}", i + 1, card)
                                ).await;
                            }
                        }

                        // if phase is not incremented post consumption, continue
                        if pre_phase + 1 != phase {
                            sleep(Duration::from_secs(2)).await;
                            continue;
                        }

                        let _ = broadcast_message(
                            game_account_id.clone().to_string(),
                            ws_url.clone(),
                            format!("Revealing community cards...")
                        ).await;

                        let player_account_id = AccountId::try_from(player_ids[0]).unwrap();
                        let mut cards: [[Felt; 4]; 3] = [[Felt::ZERO; 4]; 3];

                        match pre_phase {
                            0 => {
                                for (i, slot) in (1..4).enumerate() {
                                    let card_digest = game_account.storage().get_item(slot);
                                    cards[i] = card_digest.into();
                                }
                            },
                            1 => {
                                cards[0] = game_account.storage().get_item(4).into();
                            },
                            2 => {
                                cards[0] = game_account.storage().get_item(5).into();
                            },
                            3 => {
                                let _ = broadcast_message(
                                    game_account_id.to_string(),
                                    ws_url.clone(),
                                    format!("Game Ended")
                                ).await;
                                break;
                            },
                            _ => ()
                        }

                        // if phase changes, send community cards for unmasking
                        send_community_cards(
                            game_account_id,
                            player_account_id,
                            cards,
                            phase as u8
                        ).await;
                        sleep(Duration::from_secs(2)).await;
                    }
                }).await;
                Ok(())
            }
            Err(e) => Err(format!("Error creating game account: {}", e)),
        }
    }
}

#[derive(Deserialize, Debug)]
struct Config {
    player_ids: Vec<u64>,
    small_blind: u8,
    buy_in: u64,
}

fn load_config(config_file: &PathBuf) -> Result<Config, String> {
    Figment::from(Toml::file(config_file))
        .extract()
        .map_err(|err| { format!("Failed to load {} config file: {err}", config_file.display()) })
}
