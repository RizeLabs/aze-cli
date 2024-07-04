use aze_lib::client::{ create_aze_client, AzeClient };
use aze_lib::constants::{ PLAYER_CARD1_SLOT, PLAYER_CARD2_SLOT, PLAYER_FILE_PATH };
use aze_lib::utils::Player;
use clap::Parser;
use miden_objects::{ 
    accounts::AccountId,
    Felt, FieldElement
};
use std::fs;
use std::path::Path;

#[derive(Debug, Clone, Parser)]
pub struct PeekHandCmd {
    #[arg(short, long, default_value_t = 0)]
    player_id: u64,
}

impl PeekHandCmd {
    pub async fn execute(&self) -> Result<(), String> {
        let sender_account_id = get_id(&self);
        let mut client: AzeClient = create_aze_client();
        let (player_account, _) = client.get_account(sender_account_id).unwrap();
        for (i, slot) in (PLAYER_CARD1_SLOT..PLAYER_CARD2_SLOT + 1).enumerate() {
            let card_digest = player_account.storage().get_item(slot);
            let card_suit = card_digest.as_elements()[0].as_int();
            let card_rank = card_digest.as_elements()[1].as_int();
            let card = card_from_number(card_suit, card_rank);
            println!("Card {}: {}", i + 1, card);
        }

        Ok(())
    }
}

fn get_id(cmd: &PeekHandCmd) -> AccountId {
    if cmd.player_id == 0 {
        let path = Path::new(PLAYER_FILE_PATH);
        let player: Player = toml::from_str(&fs::read_to_string(path).expect("Failed to read Player.toml")).expect("Failed to deserialize player data");
        return AccountId::try_from(player.player_id()).unwrap();
    } 

    AccountId::try_from(cmd.player_id).unwrap()
}