use aze_enc::{ keygen, mask, remask, inter_unmask, final_unmask, CardCipher };
use aze_lib::accounts::create_basic_aze_player_account;
use aze_lib::client::{
    self, create_aze_client, AzeAccountTemplate, AzeClient, AzeGameMethods, AzeTransactionTemplate,
    SendCardTransactionData,
    GenPrivateKeyTransactionData,
    ShuffleCardTransactionData,
    RemaskTransactionData,
    SetCardsTransactionData,
    SetCommCardsTransactionData,
    SetHandTransactionData,
};
use aze_lib::constants::{
    FIRST_PLAYER_INDEX, HIGHEST_BET, NO_OF_PLAYERS, PLAYER_INITIAL_BALANCE, SMALL_BUY_IN_AMOUNT,
    PLAYER_DATA_SLOT, DEFAULT_ACTION_TYPE, PLAYER_CARD1_SLOT, PLAYER_CARD2_SLOT, TEMP_CARD_SLOT,
    FLOP_NO_OF_CARDS, FLOP_SLOT, DECK_SIZE, CURRENT_PHASE_SLOT,
};
use aze_lib::executor::execute_tx_and_sync;
use aze_lib::notes::{consume_notes, mint_note};
use aze_lib::storage::GameStorageSlotData;
use aze_types::accounts::{
    AccountCreationError, AccountCreationRequest, AccountCreationResponse,
    PlayerAccountCreationRequest, PlayerAccountCreationResponse,
};
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
        player_account_ids.clone()
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

    // distribute cards
    let sender_account_id = game_account.id();
    let mut cards = vec![];

    for i in 1..2 * NO_OF_PLAYERS + 1 {
        let slot_index = i;
        let mut card: [Felt; 4] = game_account.storage().get_item(slot_index as u8).into();
        card[2] = Felt::from(slot_index);
        cards.push(card);
    }

    for (i, _) in player_account_ids.iter().enumerate() {
        println!("Distributing cards to player {:?}", player_account_ids[i]);
        let target_account_id = AccountId::try_from(player_account_ids[i]).unwrap();
        let input_cards = [cards[2 * i], cards[2 * i + 1]];
        let sendcard_txn_data = SendCardTransactionData::new(
            sender_account_id,
            target_account_id,
            &input_cards
        );
        let transaction_template = AzeTransactionTemplate::SendCard(sendcard_txn_data);
        let txn_request = client.build_aze_send_card_tx_request(transaction_template).unwrap();
        execute_tx_and_sync(&mut client, txn_request.clone()).await;
    }
    
    Ok(game_account.id())
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

    Ok(player_account.id())
}

pub async fn consume_game_notes(account_id: AccountId) {
    let mut client: AzeClient = create_aze_client();
    client.sync_state().await.unwrap();
    let account = client.get_account(account_id).unwrap();
    let expected_note_tag = NoteTag::from_account_id(account_id, NoteExecutionHint::Local).unwrap();
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
    }
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

    let commit_hand_data = SetHandTransactionData::new(
        account_id,
        game_account_id,
        &cards,
        player_hand,
    );
    let transaction_template = AzeTransactionTemplate::SetHand(commit_hand_data);
    let txn_request = client
        .build_aze_set_hand_tx_request(transaction_template)
        .unwrap();
    execute_tx_and_sync(&mut client, txn_request.clone()).await;
}