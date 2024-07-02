use std::path::PathBuf;

use ansi_term::Colour::{Blue, Green, Red, Yellow};
use aze_lib::{
    client::{create_aze_client, AzeClient},
    constants::PLAYER_FILE_PATH,
    utils::{card_from_number, card_from_number_unique, get_stats, Player, Ws_config},
};
use clap::Parser;
use dialoguer::Input;
use miden_objects::accounts::AccountId;
use std::path::Path;

#[derive(Debug, Clone, Parser)]
pub struct SeeHandsCmd;

impl SeeHandsCmd {
    pub async fn execute(&self, ws_config: &PathBuf) -> Result<(), Box<dyn std::error::Error>> {
        let gameid: u64 = get_id();
        let mut client: AzeClient = create_aze_client();
        let game_account_id = AccountId::try_from(gameid).unwrap();
        let ws_url = Ws_config::load(ws_config).url.unwrap();
        let stat_data: aze_lib::utils::StatResponse =
            get_stats(game_account_id.to_string(), ws_url).await?;

        // Check: If game has not ended there is no hand to show. Comment for testing
        if stat_data.current_state != 3 {
            println!("{}", Red.bold().paint("Game not ended yet!!!"));
            return Ok(());
        }

        let hands_output = format!(
            "{} \n\
             {} \n\
             {} \n\
             {} \n\
             {} \n\
             {} \n\
             {} \n\
             {} \n\
             {} \n\
             {} \n\
             {} \n\
             {} \n\
             {} \n\
             {} \n\
             {} \n\
             {} 
            ",
            Blue.bold()
                .paint("+---------------------------------------------------+"),
            Red.bold()
                .paint("|                  PLAYER HANDS                     |"),
            Blue.bold()
                .paint("|---------------------------------------------------|"),
            Blue.bold()
                .paint("|                 COMMUNITY CARDS                   |"),
            Blue.bold().paint(format!(
                "|------ {:^37} ------|",
                format!(
                    "{:4} {:4} {:4} {:4} {:4}",
                    card_from_number(
                        stat_data.community_cards[0][0],
                        stat_data.community_cards[0][1],
                    ),
                    card_from_number(
                        stat_data.community_cards[1][0],
                        stat_data.community_cards[1][1],
                    ),
                    card_from_number(
                        stat_data.community_cards[2][0],
                        stat_data.community_cards[2][1],
                    ),
                    card_from_number(
                        stat_data.community_cards[3][0],
                        stat_data.community_cards[3][1],
                    ),
                    card_from_number(
                        stat_data.community_cards[4][0],
                        stat_data.community_cards[4][1],
                    )
                )
            )),
            Blue.bold()
                .paint("|---------------------------------------------------|"),
            Red.bold().paint(format!(
                "| {:^15}| {:15}| {:15} |",
                "Players", "Hand Type", "Hand Cards"
            )),
            Blue.bold()
                .paint("|---------------------------------------------------|"),
            Yellow.bold().paint(format!(
                "| {:^15}| {:15}| {:15} |",
                "P1",
                get_hand(stat_data.player_hands[0]).unwrap(),
                format!(
                    "{} {}",
                    card_from_number_unique(stat_data.player_hand_cards[0][0]),
                    card_from_number_unique(stat_data.player_hand_cards[0][1])
                )
            )),
            Blue.bold()
                .paint("|---------------------------------------------------|"),
            Yellow.bold().paint(format!(
                "| {:^15}| {:15}| {:15} |",
                "P2",
                get_hand(stat_data.player_hands[1]).unwrap(),
                format!(
                    "{} {}",
                    card_from_number_unique(stat_data.player_hand_cards[1][0]),
                    card_from_number_unique(stat_data.player_hand_cards[1][1])
                )
            )),
            Blue.bold()
                .paint("|---------------------------------------------------|"),
            Yellow.bold().paint(format!(
                "| {:^15}| {:15}| {:15} |",
                "P3",
                get_hand(stat_data.player_hands[2]).unwrap(),
                format!(
                    "{} {}",
                    card_from_number_unique(stat_data.player_hand_cards[2][0]),
                    card_from_number_unique(stat_data.player_hand_cards[2][1])
                )
            )),
            Blue.bold()
                .paint("|---------------------------------------------------|"),
            Yellow.bold().paint(format!(
                "| {:^15}| {:15}| {:15} |",
                "P4",
                get_hand(stat_data.player_hands[3]).unwrap(),
                format!(
                    "{} {}",
                    card_from_number_unique(stat_data.player_hand_cards[3][0]),
                    card_from_number_unique(stat_data.player_hand_cards[3][1])
                )
            )),
            Blue.bold()
                .paint("+---------------------------------------------------+"),
        );

        println!("{}", hands_output);
        Ok(())
    }
}

fn get_hand(int_hand: u64) -> Result<(String), String> {
    let hand = match int_hand {
        1 => "Royal Flush",
        2 => "Straight Flush",
        3 => "4 of Kind",
        4 => "Full House",
        5 => "Flush",
        6 => "Straight",
        7 => "3 of Kind",
        8 => "Two Pair",
        9 => "Pair",
        10 => "High Card",
        _ => "NA",
    };
    Ok(hand.to_string())
}

fn get_id() -> u64 {
    let path = Path::new(PLAYER_FILE_PATH);
    let player: Player =
        toml::from_str(&std::fs::read_to_string(path).expect("Failed to read Player.toml"))
            .expect("Failed to deserialize player data");
    let game_id = player.game_id().unwrap();

    game_id
}
