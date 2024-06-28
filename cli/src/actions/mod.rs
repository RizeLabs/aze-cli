use aze_lib::client::{
    create_aze_client, AzeAccountTemplate, AzeClient, AzeGameMethods, AzeTransactionTemplate,
    PlayBetTransactionData, PlayCallTransactionData, PlayCheckTransactionData,
    PlayFoldTransactionData, PlayRaiseTransactionData,
};
use aze_lib::constants::{HIGHEST_BET_SLOT, SMALL_BUY_IN_AMOUNT};
use aze_lib::executor::execute_tx_and_sync;
use aze_lib::storage::GameStorageSlotData;
use aze_types::actions::{GameActionError, GameActionResponse};
use miden_client::client::{
    accounts::{AccountStorageMode, AccountTemplate},
    transactions::transaction_request::TransactionTemplate,
};
use miden_objects::{
    accounts::AccountId,
    notes::NoteType,
};

use aze_lib::utils::{broadcast_message, read_player_data, Ws_config};

pub async fn raise(
    player_id: u64,
    game_id: u64,
    amount: Option<u8>,
    ws_config_path: &std::path::PathBuf,
) -> Result<GameActionResponse, String> {
    let mut client: AzeClient = create_aze_client();
    let player_account_id = AccountId::try_from(player_id).unwrap();
    let game_account_id = AccountId::try_from(game_id).unwrap();
    let ws_url = Ws_config::load(ws_config_path).url.unwrap();

    let _ = broadcast_message(
        game_account_id.to_string(),
        ws_url.clone(),
        format!(
            "Player: {} plays raise by amount: {}",
            read_player_data().expect("Failed to read player data from Player.toml"),
            amount.unwrap()
        ),
    )
    .await;

    // request highest bet from game account client
    let highest_bet = 5; // for now

    let playraise_txn_data = PlayRaiseTransactionData::new(
        player_account_id,
        game_account_id,
        highest_bet as u8 + amount.unwrap(),
    );

    let transaction_template = AzeTransactionTemplate::PlayRaise(playraise_txn_data);
    let txn_request = client
        .build_aze_play_raise_tx_request(transaction_template)
        .unwrap();
    execute_tx_and_sync(&mut client, txn_request.clone()).await;

    // note to be consumed by game account

    Ok(GameActionResponse { is_taken: true })
}

pub async fn call(
    player_id: u64,
    game_id: u64,
    ws_config_path: &std::path::PathBuf,
) -> Result<GameActionResponse, String> {
    let mut client: AzeClient = create_aze_client();
    let player_account_id = AccountId::try_from(player_id).unwrap();
    let game_account_id = AccountId::try_from(game_id).unwrap();

    let mut ws_url: String = String::new();

    match Ws_config::load(ws_config_path).url {
        Some(url) => {
            ws_url = url;
        }

        None => {
            eprintln!("Ws_config DNE, use init or connect command before action");
        }
    }
    let _ = broadcast_message(
        game_account_id.to_string(),
        ws_url.clone(),
        format!("Player: {} plays call ", read_player_data().expect("Failed to read player data from Player.toml")),
    )
    .await;

    let playcall_txn_data = PlayCallTransactionData::new(
        player_account_id,
        game_account_id,
    );

    let transaction_template = AzeTransactionTemplate::PlayCall(playcall_txn_data);
    let txn_request = client
        .build_aze_play_call_tx_request(transaction_template)
        .unwrap();
    execute_tx_and_sync(&mut client, txn_request.clone()).await;

    // note to be consumed by game account

    Ok(GameActionResponse { is_taken: true })
}

pub async fn check(
    player_id: u64,
    game_id: u64,
    ws_config_path: &std::path::PathBuf,
) -> Result<GameActionResponse, String> {
    let mut client: AzeClient = create_aze_client();
    let player_account_id = AccountId::try_from(player_id).unwrap();
    let game_account_id = AccountId::try_from(game_id).unwrap();

    let mut ws_url: String = String::new();

    match Ws_config::load(ws_config_path).url {
        Some(url) => {
            ws_url = url;
        }

        None => {
            eprintln!("Ws_config DNE, use init or connect command before action");
        }
    }
    let _ = broadcast_message(
        game_account_id.to_string(),
        ws_url.clone(),
        format!("Player: {} plays check", read_player_data().expect("Failed to read player data from Player.toml")),
    )
    .await;

    let playcheck_txn_data = PlayCheckTransactionData::new(
        player_account_id,
        game_account_id,
    );

    let transaction_template = AzeTransactionTemplate::PlayCheck(playcheck_txn_data);
    let txn_request = client
        .build_aze_play_check_tx_request(transaction_template)
        .unwrap();
    execute_tx_and_sync(&mut client, txn_request.clone()).await;

    // note to be consumed by game account

    Ok(GameActionResponse { is_taken: true })
}

pub async fn fold(
    player_id: u64,
    game_id: u64,
    ws_config_path: &std::path::PathBuf,
) -> Result<GameActionResponse, String> {
    let mut client: AzeClient = create_aze_client();
    let player_account_id = AccountId::try_from(player_id).unwrap();
    let game_account_id = AccountId::try_from(game_id).unwrap();

    let mut ws_url: String = String::new();

    match Ws_config::load(ws_config_path).url {
        Some(url) => {
            ws_url = url;
        }

        None => {
            eprintln!("Ws_config DNE, use init or connect command before action");
        }
    }

    let _ = broadcast_message(
        game_account_id.to_string(),
        ws_url.clone(),
        format!("Player: {} plays fold", read_player_data().expect("Failed to read player data from Player.toml")),
    )
    .await;

    let playfold_txn_data = PlayFoldTransactionData::new(
        player_account_id,
        game_account_id,
    );

    let transaction_template = AzeTransactionTemplate::PlayFold(playfold_txn_data);
    let txn_request = client
        .build_aze_play_fold_tx_request(transaction_template)
        .unwrap();
    execute_tx_and_sync(&mut client, txn_request.clone()).await;

    // note to be consumed by game account

    Ok(GameActionResponse { is_taken: true })
}

pub async fn bet(
    player_id: u64,
    game_id: u64,
    amount: u8,
    ws_config_path: &std::path::PathBuf,
) -> Result<GameActionResponse, String> {
    let mut client: AzeClient = create_aze_client();
    let player_account_id = AccountId::try_from(player_id).unwrap();
    let game_account_id = AccountId::try_from(game_id).unwrap();
    let mut ws_url: String = String::new();

    match Ws_config::load(ws_config_path).url {
        Some(url) => {
            ws_url = url;
        }

        None => {
            eprintln!("Ws_config DNE, use init or connect command before action");
        }
    }
    let _ = broadcast_message(
        game_account_id.to_string(),
        ws_url.clone(),
        format!("Player: {} bet amount: {}", read_player_data().expect("Failed to read player data from Player.toml"), amount),
    )
    .await;

    let playbet_txn_data = PlayBetTransactionData::new(
        player_account_id,
        game_account_id,
        amount,
    );

    let transaction_template = AzeTransactionTemplate::PlayBet(playbet_txn_data);
    let txn_request = client
        .build_aze_play_bet_tx_request(transaction_template)
        .unwrap();
    execute_tx_and_sync(&mut client, txn_request.clone()).await;

    // note to be consumed by game account

    Ok(GameActionResponse { is_taken: true })
}

pub async fn small_blind(
    player_id: u64,
    game_id: u64,
    ws_config_path: &std::path::PathBuf,
) -> Result<GameActionResponse, String> {
    // request small blind amount from game account
    let small_blind = 5; // for now
    bet(player_id, game_id, small_blind, ws_config_path).await
}

pub async fn big_blind(
    player_id: u64,
    game_id: u64,
    ws_config_path: &std::path::PathBuf,
) -> Result<GameActionResponse, String> {
    // request big blind amount from game account
    let big_blind = 10; // for now
    bet(player_id, game_id, big_blind, ws_config_path).await
}
