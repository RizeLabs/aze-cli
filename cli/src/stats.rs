use std::path::PathBuf;

use ansi_term::Colour::{Blue, Green, Red, Yellow};
use aze_lib::{
    client::{create_aze_client, AzeClient},
    utils::{card_from_number, get_stats, Ws_config},
};
use clap::Parser;
use dialoguer::Input;
use miden_objects::accounts::AccountId;

#[derive(Parser, Debug, Clone)]
pub struct StatsCmd;

impl StatsCmd {
    pub async fn execute(&self, ws_config: &PathBuf) -> Result<(), Box<dyn std::error::Error>> {
        let gameid: u64 = Input::<String>::new()
            .with_prompt("What is the game id?")
            .interact()
            .expect("Failed to get game id")
            .parse()
            .expect("Invalid game id");
        let mut client: AzeClient = create_aze_client();
        let game_account_id = AccountId::try_from(gameid).unwrap();
        let game_account = client.get_account(game_account_id).unwrap().0;
        let ws_url = Ws_config::load(ws_config).url.unwrap();
        let stat_data: aze_lib::utils::StatResponse =
            get_stats(game_account_id.to_string(), ws_url).await?;

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
            Blue.bold()
                .paint("+---------------------------------------------------+"),
            Red.bold()
                .paint("|                   POKER TABLE                     |"),
            Blue.bold()
                .paint("|---------------------------------------------------|"),
            Blue.bold()
                .paint("|                 COMMUNITY CARDS                   |"),
            Blue.bold().paint(format!(
                "|------ {:^37} ------|",
                format!(
                    "{:4} {:4} {:4} {:4} {:4}",
                    card_from_number(stat_data.community_cards[0]),
                    card_from_number(stat_data.community_cards[1]),
                    card_from_number(stat_data.community_cards[2]),
                    card_from_number(stat_data.community_cards[3]),
                    card_from_number(stat_data.community_cards[4])
                )
            )),
            Blue.bold().paint("|---------------------------------------------------|"),
            Yellow
                .bold()
                .paint(format!("|     {:^20} {:^20}     |", "P1", "P2")),
            Yellow.bold().paint(format!(
                "|     {:^20} {:^20}     |",
                format!("Balance: {}",stat_data.player_balances[0]), format!("Balance: {}",stat_data.player_balances[1])
            )),
            Blue.bold()
                .paint("|---------------------------------------------------|"),
            Yellow.bold().paint(format!(
                "|          {:^31}          |",
                format!("POT VALUE: {}", stat_data.pot_value)
            )),
            Blue.bold()
                .paint("|---------------------------------------------------|"),
            Yellow
                .bold()
                .paint(format!("|     {:^20} {:^20}     |", "P1", "P2")),
            Yellow.bold().paint(format!(
                "|     {:^20} {:^20}     |",
                format!("Balance: {}",stat_data.player_balances[2]), format!("Balance: {}",stat_data.player_balances[3])
            )),
            Blue.bold()
                .paint("|---------------------------------------------------|"),
            Yellow
                .bold()
                .paint(format!("|          {:^31}          |", format!("Turn: {}",stat_data.current_player))),
            Blue.bold()
                .paint("+---------------------------------------------------+")
        );

        println!("{}", poker_table);
        Ok(())
    }
}
