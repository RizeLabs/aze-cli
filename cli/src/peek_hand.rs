use crate::accounts::{ p2p_unmask_flow };
use aze_lib::client::{ create_aze_client, AzeClient };
use aze_lib::constants::{ PLAYER_CARD1_SLOT, PLAYER_CARD2_SLOT, PLAYER_FILE_PATH };
use aze_lib::utils::{ card_from_number, Player };
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
        let mut cards: [[Felt; 4]; 3] = [[Felt::ZERO; 4]; 3];
        for (i, slot) in (PLAYER_CARD1_SLOT..PLAYER_CARD2_SLOT + 1).enumerate() {
            let card = player_account.storage().get_item(slot);
            cards[i] = card.into();
        }
        p2p_unmask_flow(sender_account_id, cards).await;

        let (player_account, _) = client.get_account(sender_account_id).unwrap();
        for (i, slot) in (PLAYER_CARD1_SLOT..PLAYER_CARD2_SLOT + 1).enumerate() {
            let card_digest: [Felt; 4] = player_account.storage().get_item(slot).into();
            let card = card_from_number(card_digest[0].as_int());
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