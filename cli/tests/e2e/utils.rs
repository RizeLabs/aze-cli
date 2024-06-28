use aze_lib::client::{
    create_aze_client,
    AzeAccountTemplate,
    AzeClient,
    AzeGameMethods,
    AzeTransactionTemplate,
    GenPrivateKeyTransactionData,
    ShuffleCardTransactionData,
    RemaskTransactionData,
    InterUnmaskTransactionData,
    SendUnmaskedCardsTransactionData,
    UnmaskTransactionData,
    SetCardsTransactionData,
    SetHandTransactionData,
    SendCommunityCardsTransactionData,
};
use aze_lib::accounts::create_basic_aze_player_account;
use aze_lib::constants::{
    SMALL_BLIND_AMOUNT,
    BUY_IN_AMOUNT,
    NO_OF_PLAYERS,
    FIRST_PLAYER_INDEX,
    HIGHEST_BET,
    PLAYER_INITIAL_BALANCE,
    SMALL_BUY_IN_AMOUNT,
    DEFAULT_ACTION_TYPE,
    PLAYER_DATA_SLOT,
    PLAYER_CARD1_SLOT,
    PLAYER_CARD2_SLOT,
    TEMP_CARD_SLOT,
    REQUESTER_SLOT,
    CURRENT_PHASE_SLOT,
    FLOP_SLOT,
    PHASE_DATA_SLOT,
};
use aze_lib::executor::execute_tx_and_sync;
use aze_lib::utils::{ get_random_coin, load_config };
use aze_lib::notes::{ consume_notes, mint_note };
use aze_lib::storage::GameStorageSlotData;
use miden_client::{
    client::{
        accounts::{ AccountTemplate, AccountStorageMode },
        transactions::transaction_request::TransactionTemplate,
    },
};
use miden_lib::AuthScheme;
use miden_objects::{
    accounts::{ Account, AccountId, AccountType, AuthSecretKey },
    crypto::{
        dsa::rpo_falcon512::{PublicKey, SecretKey},
        hash::rpo::RpoDigest,
    },
    notes::NoteType,
    Felt, FieldElement
};

pub fn create_test_client() -> AzeClient {
    create_aze_client()
}

pub async fn create_player_account(client: &mut AzeClient) -> AccountId {
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

    client.insert_account(
        &player_account,
        Some(seed),
        &AuthSecretKey::RpoFalcon512(key_pair),
    );

    // keygen
    let gen_key_data = GenPrivateKeyTransactionData::new(
        player_account.id(),
        player_account.id(),
    );
    let transaction_template = AzeTransactionTemplate::GenKey(gen_key_data);
    let txn_request = client
        .build_aze_key_gen_tx_request(transaction_template)
        .unwrap();
    execute_tx_and_sync(client, txn_request.clone()).await;
    let note_id = txn_request.expected_output_notes()[0].id();
    let note = client.get_input_note(note_id).unwrap();
    consume_notes(client, player_account.id(), &[note.try_into().unwrap()]).await;

    player_account.id()
}

pub async fn create_game_account(client: &mut AzeClient) -> AccountId {
    let slot_data = GameStorageSlotData::new(
        SMALL_BLIND_AMOUNT,
        BUY_IN_AMOUNT as u8,
        NO_OF_PLAYERS,
        FIRST_PLAYER_INDEX,
        HIGHEST_BET,
        PLAYER_INITIAL_BALANCE,
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
    
    game_account.id()
}

pub async fn mask_cards(client: &mut AzeClient, game_account_id: AccountId, player_account_ids: Vec<AccountId>) {
    let target_account_id = player_account_ids[0];
    let shuffle_card_data = ShuffleCardTransactionData::new(
        game_account_id,
        target_account_id,
        [DEFAULT_ACTION_TYPE, player_account_ids[1].into(), player_account_ids[2].into(), player_account_ids[3].into()]
    );

    let transaction_template = AzeTransactionTemplate::ShuffleCard(shuffle_card_data);
    let txn_request = client
        .build_aze_shuffle_card_tx_request(transaction_template)
        .unwrap();
    execute_tx_and_sync(client, txn_request.clone()).await;
    let note_id = txn_request.expected_output_notes()[0].id();
    let note = client.get_input_note(note_id).unwrap();
    consume_notes(client, target_account_id, &[note.try_into().unwrap()]).await;
}

pub async fn remask_cards(client: &mut AzeClient, game_account_id: AccountId, player_account_ids: Vec<AccountId>, action_type: u64) {
    let target_account_id = player_account_ids[action_type as usize - 1];
    let mut player_data = vec![player_account_ids[0].into(), player_account_ids[1].into(), player_account_ids[2].into(), player_account_ids[3].into()];
    player_data.remove(action_type as usize - 1);

    let previous_player_id = AccountId::try_from(player_account_ids[action_type as usize - 2]).unwrap();
    let (player_account, _) = client.get_account(previous_player_id).unwrap();
    let mut cards: [[Felt; 4]; 52] = [[Felt::ZERO; 4]; 52];
    for (i, slot) in (1..53).enumerate() {
        let card_digest = player_account.storage().get_item(slot);
        cards[i] = card_digest.into();
    }

    let remask_data = RemaskTransactionData::new(
        game_account_id,
        target_account_id,
        &cards,
        [action_type, player_data[0], player_data[1], player_data[2]]
    );

    let transaction_template = AzeTransactionTemplate::Remask(remask_data);
    let txn_request = client
        .build_aze_remask_tx_request(transaction_template)
        .unwrap();
    execute_tx_and_sync(client, txn_request.clone()).await;
    let note_id = txn_request.expected_output_notes()[0].id();
    let note = client.get_input_note(note_id).unwrap();
    consume_notes(client, target_account_id, &[note.try_into().unwrap()]).await;

    // set cards to game account
    let (player_account, _) = client.get_account(target_account_id).unwrap();
    let mut cards: [[Felt; 4]; 52] = [[Felt::ZERO; 4]; 52];
    for (i, slot) in (1..53).enumerate() {
        let card_digest = player_account.storage().get_item(slot);
        cards[i] = card_digest.into();
    }
    let set_cards_data = SetCardsTransactionData::new(
        target_account_id,
        game_account_id,
        &cards,
    );
    let transaction_template = AzeTransactionTemplate::SetCards(set_cards_data);
    let txn_request = client
        .build_aze_set_cards_tx_request(transaction_template)
        .unwrap();
    execute_tx_and_sync(client, txn_request.clone()).await;
    let note_id = txn_request.expected_output_notes()[0].id();
    let note = client.get_input_note(note_id).unwrap();
    consume_notes(client, game_account_id, &[note.try_into().unwrap()]).await;

    // check cards
    let (game_account, _) = client.get_account(game_account_id).unwrap();
    for slot in 1..53 {
        let card_digest = game_account.storage().get_item(slot);
        assert!(card_digest != RpoDigest::new([Felt::from(slot), Felt::ZERO, Felt::ZERO, Felt::ZERO]));
    }
}

pub async fn peek_hand(client: &mut AzeClient, player_account_id: AccountId) {
    let card_slot_start = PLAYER_CARD1_SLOT;
    let card_slot_end = PLAYER_CARD2_SLOT;
    p2p_unmask_flow(client, player_account_id, [card_slot_start, card_slot_end]).await;
}

pub async fn unmask_community_cards(client: &mut AzeClient, game_account_id: AccountId, player_account_id: AccountId, current_phase: u8) {
    let (game_account, _) = client.get_account(game_account_id).unwrap();
    let mut cards: [[Felt; 4]; 3] = [[Felt::ZERO; 4]; 3];
    for (i, slot) in (1..4).enumerate() {
        let card_digest = game_account.storage().get_item(slot);
        cards[i] = card_digest.into();
    }
    // send community cards to player account
    let send_cards_data = SendCommunityCardsTransactionData::new(
        game_account_id,
        player_account_id,
        &cards,
        current_phase as u8,
    );
    let transaction_template = AzeTransactionTemplate::SendCommunityCards(send_cards_data);
    let txn_request = client
        .build_send_community_cards_tx_request(transaction_template)
        .unwrap();
    execute_tx_and_sync(client, txn_request.clone()).await;
    let note_id = txn_request.expected_output_notes()[0].id();
    let note = client.get_input_note(note_id).unwrap();
    consume_notes(client, player_account_id, &[note.try_into().unwrap()]).await;
    // check cards
    let (player_account, _) = client.get_account(player_account_id).unwrap();
    for (i, slot) in (TEMP_CARD_SLOT..TEMP_CARD_SLOT + 3).enumerate() {
        let card: [Felt; 4] = player_account.storage().get_item(slot).into();
        assert_eq!(cards[i], card);
    }
    let phase_data = player_account.storage().get_item(PHASE_DATA_SLOT);
    assert_eq!(phase_data, RpoDigest::new([Felt::from(current_phase), Felt::ZERO, Felt::ZERO, Felt::ZERO]));
    // unmask community cards
    p2p_unmask_flow(client, player_account_id, [TEMP_CARD_SLOT, TEMP_CARD_SLOT + 2]).await;
    // set community cards back
    let (player_account, _) = client.get_account(player_account_id).unwrap();
    let mut cards: [[Felt; 4]; 3] = [[Felt::ZERO; 4]; 3];
    for (i, slot) in (TEMP_CARD_SLOT..TEMP_CARD_SLOT + 3).enumerate() {
        let card_digest = player_account.storage().get_item(slot);
        cards[i] = card_digest.into();
    }
    
    let card_slot = match current_phase {
        1 => FLOP_SLOT,
        2 => FLOP_SLOT + 3,
        3 => FLOP_SLOT + 4,
        _ => FLOP_SLOT,
    };
    let set_cards_data = UnmaskTransactionData::new(
        player_account_id,
        game_account_id,
        &cards,
        card_slot,
    );
    let transaction_template = AzeTransactionTemplate::Unmask(set_cards_data);
    let txn_request = client
        .build_aze_set_community_cards_tx_request(transaction_template)
        .unwrap();
    execute_tx_and_sync(client, txn_request.clone()).await;
    let note_id = txn_request.expected_output_notes()[0].id();
    let note = client.get_input_note(note_id).unwrap();
    consume_notes(client, game_account_id, &[note.try_into().unwrap()]).await;
    // check cards
    let (game_account, _) = client.get_account(game_account_id).unwrap();
    let end_slot = match current_phase {
        1 => FLOP_SLOT + 3,
        2 => FLOP_SLOT + 4,
        3 => FLOP_SLOT + 5,
        _ => FLOP_SLOT + 3,
    };
    for (i, slot) in (card_slot..end_slot).enumerate() {
        let card: [Felt; 4] = game_account.storage().get_item(slot).into();
        assert_eq!(cards[i], [Felt::from(17 + i as u8), Felt::ZERO, Felt::ZERO, Felt::ZERO]);
    }
}

pub async fn p2p_unmask_flow(client: &mut AzeClient, player_account_id: AccountId, card_slots: [u8; 2]) {
    let (player_account, _) = client.get_account(player_account_id).unwrap();
    let player_data = player_account.storage().get_item(PLAYER_DATA_SLOT).as_elements().to_vec();
    let player_ids = [player_data[1].as_int(), player_data[2].as_int(), player_data[3].as_int()];
    let mut cards: [[Felt; 4]; 3] = [[Felt::ZERO; 4]; 3];
    for (i, slot) in (card_slots[0]..card_slots[1] + 1).enumerate() {
        let card_digest = player_account.storage().get_item(slot);
        cards[i] = card_digest.into();
    }

    for player_id in player_ids.iter() {
        let player_id = AccountId::try_from(*player_id).unwrap();
        let inter_unmask_data = InterUnmaskTransactionData::new(
            player_account_id,
            player_id,
            &cards,
            player_account_id,
        );
        let transaction_template = AzeTransactionTemplate::InterUnmask(inter_unmask_data);
        let txn_request = client
        .build_aze_inter_unmask_tx_request(transaction_template)
        .unwrap();
        execute_tx_and_sync(client, txn_request.clone()).await;
        let note_id = txn_request.expected_output_notes()[0].id();
        let note = client.get_input_note(note_id).unwrap();
        consume_notes(client, player_id, &[note.try_into().unwrap()]).await;

        let (player_account, _) = client.get_account(player_account_id).unwrap();
        let player_data = player_account.storage().get_item(PLAYER_DATA_SLOT).as_elements().to_vec();
        let action_type = player_data[0].as_int() as u8 + NO_OF_PLAYERS;

        // send inter unmasked cards back
        let (player_account, _) = client.get_account(player_id).unwrap();
        let requester_id = player_account.storage().get_item(REQUESTER_SLOT).as_elements().to_vec()[0].as_int();
        let requester_id = AccountId::try_from(requester_id).unwrap();
        // check if the requester is the player_account_id
        assert_eq!(requester_id, player_account_id);

        let mut cards: [[Felt; 4]; 3] = [[Felt::ZERO; 4]; 3];
        for (i, slot) in (TEMP_CARD_SLOT..TEMP_CARD_SLOT + 3).enumerate() {
            let card_digest = player_account.storage().get_item(slot);
            cards[i] = card_digest.into();
        }
        let unmask_data = SendUnmaskedCardsTransactionData::new(
            player_id,
            requester_id,
            &cards,
        );
        let transaction_template = AzeTransactionTemplate::SendUnmaskedCards(unmask_data);
        let txn_request = client
            .build_aze_send_unmasked_cards_tx_request(transaction_template)
            .unwrap();
        execute_tx_and_sync(client, txn_request.clone()).await;
        let note_id = txn_request.expected_output_notes()[0].id();
        let note = client.get_input_note(note_id).unwrap();
        consume_notes(client, requester_id, &[note.try_into().unwrap()]).await;

        let (player_account, _) = client.get_account(player_account_id).unwrap();
        let player_data = player_account.storage().get_item(PLAYER_DATA_SLOT).as_elements().to_vec();
        let action_type_post = player_data[0].as_int() as u8;
        // check if the action type changed
        assert_eq!(action_type, action_type_post);
        // check cards
        for (i, slot) in (TEMP_CARD_SLOT..TEMP_CARD_SLOT + 3).enumerate() {
            let card: [Felt; 4] = player_account.storage().get_item(slot).into();
            assert_eq!(cards[i], card);
        }
    }
    let (player_account, _) = client.get_account(player_account_id).unwrap();
    // send unmask note to itself
    let unmask_data = UnmaskTransactionData::new(
        player_account_id,
        player_account_id,
        &cards,
        card_slots[0],
    );
    let transaction_template = AzeTransactionTemplate::Unmask(unmask_data);
    let txn_request = client
        .build_aze_unmask_tx_request(transaction_template)
        .unwrap();
    execute_tx_and_sync(client, txn_request.clone()).await;
    let note_id = txn_request.expected_output_notes()[0].id();
    let note = client.get_input_note(note_id).unwrap();
    consume_notes(client, player_account_id, &[note.try_into().unwrap()]).await;
}

pub async fn commit_hand(
    client: &mut AzeClient,
    game_account_id: AccountId, 
    player_account_id: AccountId,
    player_hand: u8,
    player_index: u8
) {
    let (player_account, _) = client.get_account(player_account_id).unwrap();

    // send commit hand note to game account
    let mut cards: [[Felt; 4]; 2] = [[Felt::ZERO; 4]; 2];
    for (i, slot) in (PLAYER_CARD1_SLOT..PLAYER_CARD2_SLOT + 1).enumerate() {
        let card = player_account.storage().get_item(slot);
        cards[i] = card.into();
    }

    let commit_hand_data = SetHandTransactionData::new(
        player_account_id,
        game_account_id,
        &cards,
        player_hand,
        player_index,
    );
    let transaction_template = AzeTransactionTemplate::SetHand(commit_hand_data);
    let txn_request = client
        .build_aze_set_hand_tx_request(transaction_template)
        .unwrap();
    execute_tx_and_sync(client, txn_request.clone()).await;
    let note_id = txn_request.expected_output_notes()[0].id();
    let note = client.get_input_note(note_id).unwrap();
    consume_notes(client, game_account_id, &[note.try_into().unwrap()]).await;
}