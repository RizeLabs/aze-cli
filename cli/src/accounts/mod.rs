use aze_lib::accounts::create_basic_aze_player_account;
use aze_lib::client::{
    self, create_aze_client, AzeAccountTemplate, AzeClient, AzeGameMethods, AzeTransactionTemplate,
    SendCardTransactionData,
    GenPrivateKeyTransactionData,
    ShuffleCardTransactionData,
    RemaskTransactionData,
    SetCardsTransactionData,
    UnmaskTransactionData,
    InterUnmaskTransactionData,
    SendUnmaskedCardsTransactionData,
    SetHandTransactionData,
    SendCommunityCardsTransactionData,
};
use aze_lib::constants::{
    FIRST_PLAYER_INDEX, HIGHEST_BET, NO_OF_PLAYERS, PLAYER_INITIAL_BALANCE, SMALL_BUY_IN_AMOUNT,
    PLAYER_DATA_SLOT, DEFAULT_ACTION_TYPE, PLAYER_CARD1_SLOT, PLAYER_CARD2_SLOT, TEMP_CARD_SLOT,
};
use aze_lib::executor::execute_tx_and_sync;
use aze_lib::notes::{consume_notes, mint_note};
use aze_lib::storage::GameStorageSlotData;
use aze_types::accounts::{
    AccountCreationError, AccountCreationRequest, AccountCreationResponse,
    PlayerAccountCreationRequest, PlayerAccountCreationResponse,
};
use aze_lib::utils::card_from_number;
use miden_client::client::{
    accounts::{AccountStorageMode, AccountTemplate},
    transactions::transaction_request::TransactionTemplate,
};
use miden_lib::AuthScheme;
use miden_objects::{
    accounts::{AccountId, AuthSecretKey},
    crypto::dsa::rpo_falcon512::{PublicKey, SecretKey},
    notes::{ NoteType, NoteTag, NoteExecutionHint },
    Felt, FieldElement
};
use tokio::time::{ sleep, Duration };
use ecgfp5::scalar::Scalar;

pub async fn create_aze_game_account(
    player_account_ids: Vec<u64>,
    small_blind: u8,
    buy_in: u64,
) -> Result<AccountId, AccountCreationError> {
    let mut client: AzeClient = create_aze_client();
    let slot_data = GameStorageSlotData::new(
        small_blind,
        buy_in as u8,
        NO_OF_PLAYERS,
        FIRST_PLAYER_INDEX,
        HIGHEST_BET,
        PLAYER_INITIAL_BALANCE,
        player_account_ids.clone(),
    );

    let (game_account, _) = client
        .new_game_account(
            AzeAccountTemplate::GameAccount {
                mutable_code: false,
                storage_mode: AccountStorageMode::Local,
            },
            Some(slot_data),
        )
        .unwrap();

    let game_account_id = game_account.id();

    let tag: u64 = game_account.id().into();
    let note_tag: NoteTag = ((tag + tag) as u32).into();
    client.add_note_tag(note_tag).unwrap();

    // Send note for shuffling and encryption
    let sender_account_id = game_account_id;
    let target_account_id = AccountId::try_from(player_account_ids[0]).unwrap();
    let shuffle_card_data = ShuffleCardTransactionData::new(
        sender_account_id,
        target_account_id,
        [DEFAULT_ACTION_TYPE, player_account_ids[1], player_account_ids[2], player_account_ids[3]]
    );

    let transaction_template = AzeTransactionTemplate::ShuffleCard(shuffle_card_data);
    let txn_request = client
        .build_aze_shuffle_card_tx_request(transaction_template)
        .unwrap();
    execute_tx_and_sync(&mut client, txn_request.clone()).await;
    
    Ok(game_account_id)
}

pub async fn create_aze_player_account(
    identifier: String,
) -> Result<AccountId, AccountCreationError> {
    use miden_objects::accounts::AccountType;
    let key_pair = SecretKey::new();
    let pub_key: PublicKey = key_pair.public_key();
    let auth_scheme: AuthScheme = AuthScheme::RpoFalcon512 { pub_key };

    // initial seed to create the wallet account
    let init_seed: [u8; 32] = [
        95, 113, 209, 94, 84, 105, 250, 242, 223, 203, 216, 124, 22, 159, 14, 132, 215, 85, 183,
        204, 149, 90, 166, 68, 100, 73, 106, 168, 125, 237, 138, 16,
    ];

    let (player_account, seed) = create_basic_aze_player_account(
        init_seed,
        auth_scheme,
        AccountType::RegularAccountImmutableCode,
    )
    .unwrap();

    let mut client: AzeClient = create_aze_client();
    client.insert_account(
        &player_account,
        Some(seed),
        &AuthSecretKey::RpoFalcon512(key_pair),
    );

    let tag: u64 = player_account.id().into();
    let note_tag: NoteTag = ((tag + tag) as u32).into();
    client.add_note_tag(note_tag).unwrap();

    // keygen
    let gen_key_data = GenPrivateKeyTransactionData::new(
        player_account.id(),
        player_account.id(),
    );
    let transaction_template = AzeTransactionTemplate::GenKey(gen_key_data);
    let txn_request = client
        .build_aze_key_gen_tx_request(transaction_template)
        .unwrap();
    execute_tx_and_sync(&mut client, txn_request.clone()).await;
    let note_id = txn_request.expected_output_notes()[0].id();
    let note = client.get_input_note(note_id).unwrap();
    consume_notes(&mut client, player_account.id(), &[note.try_into().unwrap()]).await;

    Ok(player_account.id())
}

pub async fn consume_game_notes(account_id: AccountId) {
    let mut client: AzeClient = create_aze_client();
    let _ = match client.sync_state().await {
        Ok(_) => (),
        Err(e) => {
            return;
        }
    };
    let account = client.get_account(account_id).unwrap();
    let tag: u64 = account_id.into();
    let expected_note_tag: NoteTag = ((tag + tag) as u32).into();
    let consumable_notes = client.get_consumable_notes(Some(account_id)).unwrap();
    let filtered_notes: Vec<_> = consumable_notes
        .into_iter()
        .filter(
            |consumable_note| consumable_note.note.metadata().unwrap().tag() == expected_note_tag
        )
        .collect();

    println!("Consumable notes: {:?}", filtered_notes.len());

    for consumable_note in filtered_notes {
        let tx_template = TransactionTemplate::ConsumeNotes(account_id, vec![consumable_note.note.id()]);
        let tx_request = client.build_transaction_request(tx_template).unwrap();
        execute_tx_and_sync(&mut client, tx_request).await;
        sleep(Duration::from_secs(2)).await;
    }
}

pub async fn enc_action(action_type: u64, account_id: AccountId, target_account: AccountId) {
    let mut client: AzeClient = create_aze_client();
    let (player_account, _) = client.get_account(account_id).unwrap();
    let mut cards: [[Felt; 4]; 52] = [[Felt::ZERO; 4]; 52];
    for (i, slot) in (1..53).enumerate() {
        let card_digest = player_account.storage().get_item(slot);
        cards[i] = card_digest.into();
    }

    if action_type == 4 {
        // send set cards note to game account
        let set_cards_data = SetCardsTransactionData::new(
            player_account.id(),
            target_account,
            &cards,
        );
        let transaction_template = AzeTransactionTemplate::SetCards(set_cards_data);
        let txn_request = client
            .build_aze_set_cards_tx_request(transaction_template)
            .unwrap();
        execute_tx_and_sync(&mut client, txn_request.clone()).await;
        return
    }

    // send remask note
    let player_data = player_account.storage().get_item(PLAYER_DATA_SLOT).as_elements().to_vec();
    let mut player_data = [action_type + 1, player_data[1].as_int(), player_data[2].as_int(), player_data[3].as_int()];
    player_data[action_type as usize] = account_id.into();
    let remask_data = RemaskTransactionData::new(
        account_id,
        target_account,
        &cards,
        player_data
    );
    let transaction_template = AzeTransactionTemplate::Remask(remask_data);
    let txn_request = client
        .build_aze_remask_tx_request(transaction_template)
        .unwrap();
    execute_tx_and_sync(&mut client, txn_request.clone()).await;
}

pub async fn p2p_unmask_flow(sender_account_id: AccountId, cards: [[Felt; 4]; 3]) -> Result<(), String> {
    let mut client: AzeClient = create_aze_client();
    let (player_account, _) = client.get_account(sender_account_id).unwrap();
    let player_data = player_account.storage().get_item(PLAYER_DATA_SLOT).as_elements().to_vec();
    let action_type = player_data[0].as_int() as u8;

    let index_bound = NO_OF_PLAYERS * (NO_OF_PLAYERS - 1);
    let modulo = action_type % index_bound;
    let next_player_idx = ((modulo as f64) / (NO_OF_PLAYERS as f64)).ceil() as u8;
    
    let receiver_account_id = AccountId::try_from(player_data[next_player_idx as usize].as_int()).unwrap();
    // send inter-unmask note
    let inter_unmask_data = InterUnmaskTransactionData::new(
        sender_account_id,
        receiver_account_id,
        &cards,
        sender_account_id,
    );
    let transaction_template = AzeTransactionTemplate::InterUnmask(inter_unmask_data);
    let txn_request = client
        .build_aze_inter_unmask_tx_request(transaction_template)
        .unwrap();
    execute_tx_and_sync(&mut client, txn_request.clone()).await;

    Ok(())
}

pub async fn self_unmask(account_id: AccountId, card_slot: u8) -> Result<(), String> {
    let mut client: AzeClient = create_aze_client();
    let (player_account, _) = client.get_account(account_id).unwrap();
    let mut cards: [[Felt; 4]; 3] = [[Felt::ZERO; 4]; 3];
    for (i, slot) in (TEMP_CARD_SLOT..TEMP_CARD_SLOT + 3).enumerate() {
        let card_digest = player_account.storage().get_item(slot);
        cards[i] = card_digest.into();
    }

    // send unmask note
    let unmask_data = UnmaskTransactionData::new(
        account_id,
        account_id,
        &cards,
        card_slot
    );
    let transaction_template = AzeTransactionTemplate::Unmask(unmask_data);
    let txn_request = client
        .build_aze_unmask_tx_request(transaction_template)
        .unwrap();
    execute_tx_and_sync(&mut client, txn_request.clone()).await;
    let note_id = txn_request.expected_output_notes()[0].id();
    let note = client.get_input_note(note_id).unwrap();
    consume_notes(&mut client, account_id, &[note.try_into().unwrap()]).await;
    let (player_account, _) = client.get_account(account_id).unwrap();
    for (i, slot) in (PLAYER_CARD1_SLOT..PLAYER_CARD2_SLOT + 1).enumerate() {
        let card_digest: [Felt; 4] = player_account.storage().get_item(slot).into();
        let card = card_from_number(card_digest[0].as_int());
        println!("Card {}: {}", i + 1, card);
    }
    Ok(())
}

pub async fn set_community_cards(account_id: AccountId, receiver_account_id: AccountId, cards: [[Felt; 4]; 3], card_slot: u8) {
    let mut client: AzeClient = create_aze_client();
    let (player_account, _) = client.get_account(account_id).unwrap();

    // send set cards note to game account
    let set_cards_data = UnmaskTransactionData::new(
        account_id,
        receiver_account_id,
        &cards,
        card_slot
    );
    let transaction_template = AzeTransactionTemplate::Unmask(set_cards_data);
    let txn_request = client
        .build_aze_set_community_cards_tx_request(transaction_template)
        .unwrap();
    execute_tx_and_sync(&mut client, txn_request.clone()).await;
}

pub async fn send_community_cards(account_id: AccountId, receiver_account_id: AccountId, cards: [[Felt; 4]; 3], phase: u8) {
    let mut client: AzeClient = create_aze_client();
    let (player_account, _) = client.get_account(account_id).unwrap();

    // send set cards note to game account
    let send_cards_data = SendCommunityCardsTransactionData::new(
        account_id,
        receiver_account_id,
        &cards,
        phase
    );
    let transaction_template = AzeTransactionTemplate::SendCommunityCards(send_cards_data);
    let txn_request = client
        .build_send_community_cards_tx_request(transaction_template)
        .unwrap();
    execute_tx_and_sync(&mut client, txn_request.clone()).await;
}

pub async fn send_unmasked_cards(account_id: AccountId, requester_id: AccountId) {
    let mut client: AzeClient = create_aze_client();
    let (player_account, _) = client.get_account(account_id).unwrap();

    let mut cards: [[Felt; 4]; 3] = [[Felt::ZERO; 4]; 3];
    for (i, slot) in (TEMP_CARD_SLOT..TEMP_CARD_SLOT + 3).enumerate() {
        let card_digest = player_account.storage().get_item(slot);
        cards[i] = card_digest.into();
    }

    // send set unmasked cards note
    let unmask_data = SendUnmaskedCardsTransactionData::new(
        account_id,
        requester_id,
        &cards,
    );
    let transaction_template = AzeTransactionTemplate::SendUnmaskedCards(unmask_data);
    let txn_request = client
        .build_aze_send_unmasked_cards_tx_request(transaction_template)
        .unwrap();
    execute_tx_and_sync(&mut client, txn_request.clone()).await;
}

pub async fn commit_hand(account_id: AccountId, game_account_id: AccountId, player_hand: u8) {
    let mut client: AzeClient = create_aze_client();
    let (player_account, _) = client.get_account(account_id).unwrap();

    // send commit hand note to game account

    let mut cards: [[Felt; 4]; 2] = [[Felt::ZERO; 4]; 2];
    for (i, slot) in (PLAYER_CARD1_SLOT..PLAYER_CARD2_SLOT + 1).enumerate() {
        let card = player_account.storage().get_item(slot);
        cards[i] = card.into();
    }

    let player_data = player_account.storage().get_item(PLAYER_DATA_SLOT).as_elements().to_vec();
    let action_type = player_data[0].as_int() as u8;
    let player_index = if action_type % NO_OF_PLAYERS != 0 {
        action_type % NO_OF_PLAYERS
    } else {
        NO_OF_PLAYERS
    };

    let commit_hand_data = SetHandTransactionData::new(
        account_id,
        game_account_id,
        &cards,
        player_hand,
        player_index
    );
    let transaction_template = AzeTransactionTemplate::SetHand(commit_hand_data);
    let txn_request = client
        .build_aze_set_hand_tx_request(transaction_template)
        .unwrap();
    execute_tx_and_sync(&mut client, txn_request.clone()).await;
}