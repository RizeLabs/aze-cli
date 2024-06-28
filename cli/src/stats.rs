use std::path::PathBuf;

use ansi_term::Colour::{ Blue, Green, Red, Yellow };
use aze_lib::{
    client::{ create_aze_client, AzeClient },
    constants::PLAYER_FILE_PATH,
    utils::{ card_from_number, get_stats, Ws_config, Player },
};
use clap::Parser;
use dialoguer::Input;
use miden_objects::accounts::AccountId;
use std::path::Path;

#[derive(Parser, Debug, Clone)]
pub struct StatsCmd;

impl StatsCmd {
    pub async fn execute(&self, ws_config: &PathBuf) -> Result<(), Box<dyn std::error::Error>> {
        let gameid: u64 = get_id();
        let mut client: AzeClient = create_aze_client();
        let game_account_id = AccountId::try_from(gameid).unwrap();
        let ws_url = Ws_config::load(ws_config).url.unwrap();
        let stat_data: aze_lib::utils::StatResponse = get_stats(
            game_account_id.to_string(),
            ws_url
        ).await?;

        let poker_table = format!(
            "{}\n\
             {}\n\
             {}\n\
             {}\n\
             {}\n\
             {}\n\
             {}\n\
             {}\n\
             {}\n\
             {}\n\
             {}\n\
             {}\n\
             {}\n\
             {}\n\
             {}\n\
             {}",
            Blue.bold().paint("+---------------------------------------------------+"),
            Red.bold().paint("|                   POKER TABLE                     |"),
            Blue.bold().paint("|---------------------------------------------------|"),
            Blue.bold().paint("|                 COMMUNITY CARDS                   |"),
            Blue.bold().paint(
                format!(
                    "|------ {:^37} ------|",
                    format!(
                        "{:4} {:4} {:4} {:4} {:4}",
                        card_from_number(stat_data.community_cards[0]),
                        card_from_number(stat_data.community_cards[1]),
                        card_from_number(stat_data.community_cards[2]),
                        card_from_number(stat_data.community_cards[3]),
                        card_from_number(stat_data.community_cards[4])
                    )
                )
            ),
            Blue.bold().paint("|---------------------------------------------------|"),
            Yellow.bold().paint(
                format!(
                    "|     {:^20} {:^20}     |",
                    if stat_data.has_folded[0] == 0 {
                        "P1".to_string()
                    } else {
                        strike_through("P1".to_string())
                    },
                    if stat_data.has_folded[1] == 0 {
                        "P2".to_string()
                    } else {
                        strike_through("P2".to_string())
                    }
                )
            ),
            Yellow.bold().paint(
                format!(
                    "|     {:^20} {:^20}     |",
                    format!("Balance: {}", stat_data.player_balances[0]),
                    format!("Balance: {}", stat_data.player_balances[1])
                )
            ),
            Blue.bold().paint("|---------------------------------------------------|"),
            Yellow.bold().paint(
                format!(
                    "|          {:^31}          |",
                    format!("POT VALUE: {}", stat_data.pot_value)
                )
            ),
            Blue.bold().paint("|---------------------------------------------------|"),
            Yellow.bold().paint(
                format!(
                    "|     {:^20} {:^20}     |",
                    if stat_data.has_folded[2] == 0 {
                        "P3".to_string()
                    } else {
                        strike_through("P3".to_string())
                    },
                    if stat_data.has_folded[3] == 0 {
                        "P4".to_string()
                    } else {
                        strike_through("P4".to_string())
                    }
                )
            ),
            Yellow.bold().paint(
                format!(
                    "|     {:^20} {:^20}     |",
                    format!("Balance: {}", stat_data.player_balances[2]),
                    format!("Balance: {}", stat_data.player_balances[3])
                )
            ),
            Blue.bold().paint("|---------------------------------------------------|"),
            Yellow.bold().paint(
                format!(
                    "|          {:^31}          |",
                    format!("Turn: {}", stat_data.current_player)
                )
            ),
            Blue.bold().paint("+---------------------------------------------------+")
        );

        println!("{}", poker_table);
        Ok(())
    }
}

fn get_id() -> u64 {
    let path = Path::new(PLAYER_FILE_PATH);
    let player: Player = toml
        ::from_str(&std::fs::read_to_string(path).expect("Failed to read Player.toml"))
        .expect("Failed to deserialize player data");
    let game_id = player.game_id().unwrap();

    game_id
}

fn strike_through(msg: String) -> String {
    let strike_start = "\x1b[9m";
    let reset = "\x1b[0m";
    format!("{}{}{}", strike_start, msg, reset)
}