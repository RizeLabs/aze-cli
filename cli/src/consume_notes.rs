use crate::accounts::{
    consume_game_notes,
    enc_action,
    p2p_unmask_flow,
    self_unmask,
    set_community_cards,
    send_unmasked_cards,
};
use aze_lib::client::{ create_aze_client, AzeClient };
use aze_lib::constants::{
    PLAYER_DATA_SLOT,
    PLAYER_CARD1_SLOT,
    TEMP_CARD_SLOT,
    REQUESTER_SLOT,
    PHASE_DATA_SLOT,
    FLOP_SLOT,
    PLAYER_FILE_PATH
};
use aze_lib::utils::Player;
use clap::Parser;
use miden_objects::{
    accounts::AccountId,
    Felt, FieldElement
};
use tokio::time::{ sleep, Duration };
use tokio::task::LocalSet;
use std::path::Path;
use dialoguer::{Input, Select};
use std::fs::File;
use std::io::Write;

#[derive(Debug, Clone, Parser)]
pub struct ConsumeNotesCmd {
    #[arg(short, long, default_value_t = 0)]
    player_id: u64,

    #[arg(short, long, default_value_t = 0)]
    game_id: u64,
}

impl ConsumeNotesCmd {
    pub async fn execute(&self) -> Result<(), String> {
        let mut client: AzeClient = create_aze_client();
        let (account_id, game_id) = get_or_prompt_ids();
        let local_set = LocalSet::new();
        local_set.run_until(async {
            loop {
                let (player_account, _) = client.get_account(account_id).unwrap();
                let player_data = player_account
                    .storage()
                    .get_item(PLAYER_DATA_SLOT)
                    .as_elements()
                    .to_vec();
                let requester_info = player_account
                    .storage()
                    .get_item(REQUESTER_SLOT)
                    .as_elements()
                    .to_vec();
                let action_type_pre = player_data[0].as_int();
                let requester_id = requester_info[0].as_int();
                let community_card = player_account.storage().get_item(TEMP_CARD_SLOT).as_elements().to_vec();

                consume_game_notes(account_id).await;

                let (player_account, _) = client.get_account(account_id).unwrap();
                let player_data = player_account
                    .storage()
                    .get_item(PLAYER_DATA_SLOT)
                    .as_elements()
                    .to_vec();
                let requester_info = player_account
                    .storage()
                    .get_item(REQUESTER_SLOT)
                    .as_elements()
                    .to_vec();
                let action_type = player_data[0].as_int();
                let requester_id_post = requester_info[0].as_int();
                let community_card_post = player_account.storage().get_item(TEMP_CARD_SLOT).as_elements().to_vec();

                // if requester_id has changed post consumption
                if requester_id != requester_id_post {

                    if community_card != community_card_post {
                        let mut cards: [[Felt; 4]; 3] = [[Felt::ZERO; 4]; 3];
                        for (i, slot) in (TEMP_CARD_SLOT..TEMP_CARD_SLOT + 3).enumerate() {
                            let card_digest = player_account.storage().get_item(slot);
                            cards[i] = card_digest.into();
                        }
                        p2p_unmask_flow(account_id, cards).await;
                        return
                    }

                    let requester_account_id = AccountId::try_from(requester_id_post).unwrap();
                    send_unmasked_cards(account_id, requester_account_id).await;
                }

                // if action type hasn't changed post consumption, continue
                if action_type == action_type_pre {
                    sleep(Duration::from_secs(5)).await;
                    continue;
                } else if
                    // check here if note triggered enc/dec action
                    (1..4).contains(&action_type)
                {
                    let target_account = AccountId::try_from(
                        player_data[action_type as usize]
                    ).unwrap();
                    enc_action(action_type, account_id, target_account).await;
                } else if action_type == 4 {
                    let target_account = AccountId::try_from(game_id).unwrap();
                    enc_action(action_type, account_id, target_account).await;
                } else if
                    (5..13).contains(&action_type) ||
                    (17..25).contains(&action_type) ||
                    (29..37).contains(&action_type) ||
                    (41..49).contains(&action_type)
                {
                    let mut cards: [[Felt; 4]; 3] = [[Felt::ZERO; 4]; 3];
                    for (i, slot) in (TEMP_CARD_SLOT..TEMP_CARD_SLOT + 3).enumerate() {
                        let card_digest = player_account.storage().get_item(slot);
                        cards[i] = card_digest.into();
                    }
                    p2p_unmask_flow(account_id, cards).await;
                } else if (13..17).contains(&action_type) {
                    self_unmask(account_id, PLAYER_CARD1_SLOT).await;
                } else if
                    (25..29).contains(&action_type) ||
                    (37..41).contains(&action_type) ||
                    (49..53).contains(&action_type)
                {
                    self_unmask(account_id, TEMP_CARD_SLOT).await;
                    // send cards to game account
                    let game_account_id = AccountId::try_from(game_id).unwrap();
                    let mut cards: [[Felt; 4]; 3] = [[Felt::ZERO; 4]; 3];
                    for (i, slot) in (TEMP_CARD_SLOT..TEMP_CARD_SLOT + 3).enumerate() {
                        let card_digest = player_account.storage().get_item(slot);
                        cards[i] = card_digest.into();
                    }
                    let current_phase = player_account.storage().get_item(PHASE_DATA_SLOT).as_elements().to_vec()[0].as_int();
                    let card_slot = match current_phase {
                        1 => FLOP_SLOT,
                        2 => FLOP_SLOT + 3,
                        3 => FLOP_SLOT + 4,
                        _ => FLOP_SLOT,
                    };
                    set_community_cards(account_id, game_account_id, cards, card_slot).await;
                }

                sleep(Duration::from_secs(5)).await;
            }
        }).await;
        Ok(())
    }
}

fn get_or_prompt_ids() -> (AccountId, AccountId) {
    let path = Path::new(PLAYER_FILE_PATH);
    let mut player_id: u64 = 0;
    let mut identifier: String = "".to_string();
    if path.exists() {
        let player: Player = toml::from_str(
            &std::fs::read_to_string(path).expect("Failed to read Player.toml file"),
        )
        .expect("Failed to parse Player.toml file");

        if let Some(game_id) = player.game_id() {
            let player_id = AccountId::try_from(player.player_id()).unwrap();
            let game_id = AccountId::try_from(game_id).unwrap();
            return (player_id, game_id);
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

    let player_id = AccountId::try_from(player_id).unwrap();
    let game_id = AccountId::try_from(game_id).unwrap();
    (player_id, game_id)
}