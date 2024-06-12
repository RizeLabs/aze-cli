use miden_objects::{
    accounts::{Account, AccountCode, AccountId, AccountStorage, SlotItem},
    assembly::{ModuleAst, ProgramAst},
    assets::{Asset, AssetVault, FungibleAsset, TokenSymbol},
    crypto::{
        dsa::rpo_falcon512::SecretKey,
        rand::{FeltRng, RpoRandomCoin},
        utils::Serializable,
    },
    notes::{Note, NoteId, NoteScript, NoteType},
    transaction::{
        ChainMmr, ExecutedTransaction, InputNote, InputNotes, ProvenTransaction, TransactionInputs,
    },
    BlockHeader, Felt, Word,
};

use crate::{
    client::{AzeAccountTemplate, AzeClient, AzeGameMethods},
    constants::{
        BUY_IN_AMOUNT, CURRENT_TURN_INDEX_SLOT, HIGHEST_BET, NO_OF_PLAYERS, PLAYER_INITIAL_BALANCE,
        SMALL_BLIND_AMOUNT, SMALL_BUY_IN_AMOUNT,
    },
    notes::{consume_notes, mint_note},
    storage::GameStorageSlotData,
};
use ::rand::Rng;
use figment::{
    providers::{Format, Toml},
    Figment,
};
use miden_client::{
    client::{
        accounts::{AccountStorageMode, AccountTemplate},
        rpc::NodeRpcClient,
        Client,
    },
    config::ClientConfig,
    errors::{ClientError, IdPrefixFetchError},
    store::{sqlite_store::SqliteStore, InputNoteRecord, NoteFilter as ClientNoteFilter, Store},
};
use std::path::Path;
use std::{env::temp_dir, fs, time::Duration};

use reqwest::Client as httpClient;
use serde::{Deserialize, Serialize};
use std::error::Error;

// use uuid::Uuid;

pub fn get_new_key_pair_with_advice_map() -> (Word, Vec<Felt>) {
    let keypair = SecretKey::new();

    let pk: Word = keypair.public_key().into();
    let pk_sk_bytes = keypair.to_bytes();
    let pk_sk_felts: Vec<Felt> = pk_sk_bytes
        .iter()
        .map(|a| Felt::new(*a as u64))
        .collect::<Vec<Felt>>();

    (pk, pk_sk_felts)
}

pub fn create_aze_store_path() -> std::path::PathBuf {
    let mut temp_file = temp_dir();
    temp_file.push(format!("{}.sqlite3", "random")); // for now don't know why uuid is not importing
    temp_file
}

pub fn load_config(config_file: &Path) -> Result<ClientConfig, String> {
    Figment::from(Toml::file(config_file))
        .extract()
        .map_err(|err| {
            format!(
                "Failed to load {} config file: {err}",
                config_file.display()
            )
        })
}

pub fn get_random_coin() -> RpoRandomCoin {
    // TODO: Initialize coin status once along with the client and persist status for retrieval
    let mut rng = rand::thread_rng();
    let coin_seed: [u64; 4] = rng.gen();

    RpoRandomCoin::new(coin_seed.map(Felt::new))
}

// TODO hide this methods under debug feature
pub async fn log_account_status(client: &AzeClient, account_id: AccountId) {
    let (regular_account, _seed) = client.get_account(account_id).unwrap();
    println!(
        "Account asset count --> {:?}",
        regular_account.vault().assets().count()
    );
    println!(
        "Account storage root --> {:?}",
        regular_account.storage().root()
    );
    println!(
        "Account slot 100 --> {:?}",
        regular_account.storage().get_item(100)
    );
    println!(
        "Account slot 101 --> {:?}",
        regular_account.storage().get_item(101)
    );
}

pub async fn log_slots(client: &AzeClient, account_id: AccountId) {
    let (regular_account, _seed) = client.get_account(account_id).unwrap();
    for i in 1..100 {
        println!(
            "Account slot {:?} --> {:?}",
            i,
            regular_account.storage().get_item(i)
        );
    }
}

pub async fn setup_accounts(
    mut client: &mut AzeClient,
) -> (FungibleAsset, AccountId, AccountId, GameStorageSlotData) {
    let slot_data = GameStorageSlotData::new(
        SMALL_BLIND_AMOUNT,
        SMALL_BUY_IN_AMOUNT,
        NO_OF_PLAYERS,
        CURRENT_TURN_INDEX_SLOT,
        HIGHEST_BET,
        PLAYER_INITIAL_BALANCE,
    );

    let (game_account, _) = client
        .new_game_account(
            AzeAccountTemplate::GameAccount {
                mutable_code: false,
                storage_mode: AccountStorageMode::Local, // for now
            },
            Some(slot_data.clone()),
        )
        .unwrap();
    let game_account_id = game_account.id();
    log_slots(&client, game_account_id).await;

    let (player_account, _) = client
        .new_game_account(
            AzeAccountTemplate::PlayerAccount {
                mutable_code: false,
                storage_mode: AccountStorageMode::Local, // for now
            },
            None,
        )
        .unwrap();
    let player_account_id = player_account.id();

    let (faucet_account, _) = client
        .new_account(AccountTemplate::FungibleFaucet {
            token_symbol: TokenSymbol::new("MATIC").unwrap(),
            decimals: 8,
            max_supply: 1_000_000_000,
            storage_mode: AccountStorageMode::Local,
        })
        .unwrap();
    let faucet_account_id = faucet_account.id();

    let note = mint_note(
        &mut client,
        player_account_id,
        faucet_account_id,
        NoteType::Public,
    )
    .await;
    println!("Minted note");
    consume_notes(&mut client, player_account_id, &[note]).await;

    let fungible_asset = FungibleAsset::new(faucet_account_id, BUY_IN_AMOUNT).unwrap();
    let sender_account_id = player_account_id;
    let target_account_id = game_account_id;

    return (
        fungible_asset,
        sender_account_id,
        target_account_id,
        slot_data,
    );
}

#[derive(Serialize)]
pub struct PublishRequest {
    game_id: String,
    event: String,
}
#[derive(Serialize)]
pub struct StatRequest {
    game_id: String,
}
#[derive(Serialize, Deserialize)]
pub struct StatResponse {
    pub community_cards: Vec<u64>,
    pub player_balances: Vec<u64>,
    pub current_player: u64,
    pub pot_value: u64,
}

// Config for saving broadcast url
#[derive(Default, Serialize, Deserialize)]
pub struct Ws_config {
    pub url: Option<String>,
}

impl Ws_config {
    pub fn new() -> Self {
        Ws_config { url: None }
    }

    pub fn load(config_path: &std::path::PathBuf) -> Self {
        if let Ok(config_data) = fs::read_to_string(config_path) {
            serde_json::from_str(&config_data).unwrap_or_default()
        } else {
            Ws_config::new()
        }
    }

    pub fn save(&self, config_path: &std::path::PathBuf) {
        if let Ok(config_data) = serde_json::to_string_pretty(self) {
            fs::write(config_path, config_data).expect("Unable to write config file");
        }
    }
}

pub async fn broadcast_message(game_id: String, url: String, message: String) -> Result<(), Box<dyn Error>> {
    let client = httpClient::new();
    let url = url::Url::parse(&url).unwrap();
    let base_url = format!("http://{}", url.host_str().unwrap());
    let port = url.port().map(|p| format!(":{}", p)).unwrap_or_default();
    let publish_url = format!("{}{}{}", base_url, port, "/publish");
    
    let request_body = PublishRequest {
        game_id,
        event: message,
    };

    let response = client
        .post(&publish_url)
        .json(&request_body)
        .send()
        .await?;

    if response.status().is_success() {
        println!("Message successfully published");
        Ok(())
    } else {
        let status = response.status();
        let error_text = response.text().await?;
        eprintln!("Failed to publish message: {} - {}", status, error_text);
        Err(format!("Failed to publish message: {} - {}", status, error_text).into())
    }
}

pub async fn get_stats(game_id: String, url: String) -> Result<StatResponse, Box<dyn Error>>{
    let client = httpClient::new();
    let url = url::Url::parse(&url).unwrap();
    let base_url = format!("http://{}", url.host_str().unwrap());
    let port = url.port().map(|p| format!(":{}", p)).unwrap_or_default();
    let stat_url = format!("{}{}{}", base_url, port, "/stats");

    let request_body = StatRequest {
        game_id
    };

    let response = client
        .post(&stat_url)
        .json(&request_body)
        .send()
        .await?;

    if response.status().is_success() {
        Ok(response.json().await? )
    }else {
        let status = response.status();
        let error_text = response.text().await?;
        eprintln!("Failed to get stats: {} - {}", status, error_text);
        Err(format!("Failed to get stats: {} - {}", status, error_text).into())
    }
}