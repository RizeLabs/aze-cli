use miden_objects::{
    accounts::{
        Account, AccountCode, AccountId, AccountStorage, AccountType, SlotItem, StorageSlot,
        StorageSlotType, StorageMap
    },
    assembly::ModuleAst,
    assets::{Asset, AssetVault},
    crypto::hash::rpo::RpoDigest,
    AccountError, Felt, FieldElement, Word, ZERO,
};

use crate::constants::{ PLAYER_STATS_SLOTS, SECRET_KEY_SLOT, DEFAULT_SKEY, MASKING_FACTOR_SLOT, DEFAULT_MASKING_FACTOR };
use crate::storage::GameStorageSlotData;
use miden_lib::{transaction::TransactionKernel, AuthScheme};

fn construct_game_constructor_storage(
    auth_scheme: AuthScheme,
    slot_data: GameStorageSlotData,
) -> Vec<SlotItem> {
    let mut game_info: Vec<SlotItem> = vec![];
    // generate 52 cards
    let mut cards: Vec<SlotItem> = vec![];
    let mut player_pub_keys: Vec<SlotItem> = vec![];

    let small_blind_amt = slot_data.small_blind_amt();
    let buy_in_amt = slot_data.buy_in_amt();
    let no_of_players = slot_data.player_count();
    let flop_index = slot_data.flop_index();
    let player_account_ids = slot_data.player_account_ids();

    let mut slot_index = 1u8;

    let (_, storage_slot_0_data): (&str, Word) = match auth_scheme {
        AuthScheme::RpoFalcon512 { pub_key } => ("basic::auth_tx_rpo_falcon512", pub_key.into()),
    };

    let auth_slot = SlotItem {
        index: slot_index - 1, // 0th slot
        slot: StorageSlot::new_value(storage_slot_0_data),
    };

    for card_suit in 1..5 {
        for card_number in 1..14 {
            let slot_item: SlotItem = SlotItem {
                index: slot_index,
                slot: StorageSlot {
                    slot_type: StorageSlotType::Value { value_arity: 0 },
                    value: [
                        Felt::from(card_suit as u8),
                        Felt::from(card_number as u8),
                        Felt::ZERO, // denotes is encrypted
                        Felt::ZERO,
                    ],
                },
            };

            cards.push(slot_item);
            slot_index += 1;
        }
    }

    let game_stats = vec![
        SlotItem {
            index: slot_index, // storing next_turn here
            slot: StorageSlot {
                slot_type: StorageSlotType::Value { value_arity: 0 },
                value: [
                    Felt::from(flop_index as u8),
                    Felt::ZERO,
                    Felt::ZERO,
                    Felt::ZERO,
                ],
            },
        },
        SlotItem {
            index: slot_index + 1, // storing small blind amt here
            slot: StorageSlot {
                slot_type: StorageSlotType::Value { value_arity: 0 },
                value: [
                    Felt::from(small_blind_amt as u8),
                    Felt::ZERO,
                    Felt::ZERO,
                    Felt::ZERO,
                ],
            },
        },
        SlotItem {
            index: slot_index + 2, // storing big blind amt here
            slot: StorageSlot {
                slot_type: StorageSlotType::Value { value_arity: 0 },
                value: [
                    Felt::from(small_blind_amt * (2 as u8)),
                    Felt::ZERO,
                    Felt::ZERO,
                    Felt::ZERO,
                ],
            },
        },
        SlotItem {
            index: slot_index + 3, // storing buy in amt here
            slot: StorageSlot {
                slot_type: StorageSlotType::Value { value_arity: 0 },
                value: [
                    Felt::from(buy_in_amt as u8),
                    Felt::ZERO,
                    Felt::ZERO,
                    Felt::ZERO,
                ],
            },
        },
        SlotItem {
            index: slot_index + 4, // storing no of players here
            slot: StorageSlot {
                slot_type: StorageSlotType::Value { value_arity: 0 },
                value: [
                    Felt::from(no_of_players as u8),
                    Felt::ZERO,
                    Felt::ZERO,
                    Felt::ZERO,
                ],
            },
        },
        SlotItem {
            index: slot_index + 5, // raiser pub key index
            slot: StorageSlot {
                slot_type: StorageSlotType::Value { value_arity: 0 },
                value: [Felt::ZERO, Felt::ZERO, Felt::ZERO, Felt::ZERO],
            },
        },
        SlotItem {
            index: slot_index + 7, // storing curr turn pub key index
            slot: StorageSlot {
                slot_type: StorageSlotType::Value { value_arity: 0 },
                value: [
                    Felt::from(slot_data.current_turn_index()),
                    Felt::ZERO,
                    Felt::ZERO,
                    Felt::ZERO,
                ],
            },
        },
        SlotItem {
            index: slot_index + 8, // storing highest bet
            slot: StorageSlot {
                slot_type: StorageSlotType::Value { value_arity: 0 },
                value: [
                    Felt::from(slot_data.highest_bet()),
                    Felt::ZERO,
                    Felt::ZERO,
                    Felt::ZERO,
                ],
            },
        },
    ];

    slot_index += 11;

    for i in 0..no_of_players {
        let player_slots = vec![
            SlotItem {
                index: slot_index, // account id
                slot: StorageSlot {
                    slot_type: StorageSlotType::Value { value_arity: 0 },
                    value: [Felt::new(player_account_ids[i as usize]), Felt::ZERO, Felt::ZERO, Felt::ZERO],
                },
            },
            SlotItem {
                index: slot_index + 1, // pub key
                slot: StorageSlot {
                    slot_type: StorageSlotType::Value { value_arity: 0 },
                    value: [
                        Felt::from(1 as u8),
                        Felt::from(1 as u8),
                        Felt::from(1 as u8),
                        Felt::from(1 as u8),
                    ],
                },
            },
            SlotItem {
                index: slot_index + 3, // current bet
                slot: StorageSlot {
                    slot_type: StorageSlotType::Value { value_arity: 0 },
                    value: [Felt::ZERO, Felt::ZERO, Felt::ZERO, Felt::ZERO],
                },
            },
            SlotItem {
                index: slot_index + 4, // player balance
                slot: StorageSlot {
                    slot_type: StorageSlotType::Value { value_arity: 0 },
                    value: [
                        Felt::from(slot_data.player_balance()),
                        Felt::ZERO,
                        Felt::ZERO,
                        Felt::ZERO,
                    ],
                },
            },
        ];
        player_pub_keys.extend(player_slots);

        slot_index += PLAYER_STATS_SLOTS; // since the mid 13 elements would cover the player stats and initially all those values are zero
    }

    // merge player_id with card_suit
    game_info.push(auth_slot);
    game_info.extend(cards);
    game_info.extend(game_stats);
    game_info.extend(player_pub_keys);
    game_info
}

// method to create a basic aze game account
// it might also would take in cards but for now we are just initializing it with 52 hardcoded cards
pub fn create_basic_aze_game_account(
    init_seed: [u8; 32],
    auth_scheme: AuthScheme,
    account_type: AccountType,
    slot_data: GameStorageSlotData,
) -> Result<(Account, Word), AccountError> {
    if matches!(
        account_type,
        AccountType::FungibleFaucet | AccountType::NonFungibleFaucet
    ) {
        return Err(AccountError::AccountIdInvalidFieldElement(
            "Basic aze accounts cannot have a faucet account type".to_string(),
        ));
    }

    let aze_game_account_code_src: &str = include_str!("../../contracts/core/game.masm");

    let aze_game_account_code_ast = ModuleAst::parse(aze_game_account_code_src)
        .map_err(|e| AccountError::AccountCodeAssemblerError(e.into()))?;
    let account_assembler = TransactionKernel::assembler();
    let aze_game_account_code =
        AccountCode::new(aze_game_account_code_ast.clone(), &account_assembler)?;

    let game_constructor_item = construct_game_constructor_storage(auth_scheme, slot_data);

    // initializing game storage with 52 cards
    let aze_game_account_storage = AccountStorage::new(game_constructor_item, vec![])?;

    // we need to fund the account with some fungible asset which it could use to rewards players
    let account_vault = AssetVault::new(&[]).expect("error on empty vault");

    let account_seed = AccountId::get_account_seed(
        init_seed,
        account_type,
        miden_objects::accounts::AccountStorageType::OnChain,
        aze_game_account_code.root(),
        aze_game_account_storage.root(),
    )?;
    let account_id = AccountId::new(
        account_seed,
        aze_game_account_code.root(),
        aze_game_account_storage.root(),
    )?;
    Ok((
        Account::new(
            account_id,
            account_vault,
            aze_game_account_storage,
            aze_game_account_code,
            ZERO,
        ),
        account_seed,
    ))
}

// method to create basic aze player account in case the user don't have an existing account
pub fn create_basic_aze_player_account(
    init_seed: [u8; 32],
    auth_scheme: AuthScheme,
    account_type: AccountType,
) -> Result<(Account, Word), AccountError> {
    if matches!(
        account_type,
        AccountType::FungibleFaucet | AccountType::NonFungibleFaucet
    ) {
        return Err(AccountError::AccountIdInvalidFieldElement(
            "Basic aze player accounts cannot have a faucet account type".to_string(),
        ));
    }

    let (_, storage_slot_0_data): (&str, Word) = match auth_scheme {
        AuthScheme::RpoFalcon512 { pub_key } => ("basic::auth_tx_rpo_falcon512", pub_key.into()),
    };

    let aze_player_account_code_src: &str = include_str!("../../contracts/core/player.masm");

    let aze_player_account_code_ast = ModuleAst::parse(aze_player_account_code_src)
        .map_err(|e| AccountError::AccountCodeAssemblerError(e.into()))?;
    let account_assembler = TransactionKernel::assembler();
    let aze_player_account_code =
        AccountCode::new(aze_player_account_code_ast.clone(), &account_assembler)?;

    let aze_player_account_storage = AccountStorage::new(
        vec![
            SlotItem {
                index: 0,
                slot: StorageSlot {
                    slot_type: StorageSlotType::Value { value_arity: 0 },
                    value: storage_slot_0_data,
                },
            },
            SlotItem {
                index: SECRET_KEY_SLOT,
                slot: StorageSlot {
                    slot_type: StorageSlotType::Value { value_arity: 0 },
                    value: [Felt::from(DEFAULT_SKEY), Felt::ZERO, Felt::ZERO, Felt::ZERO],
                },
            },
            SlotItem {
                index: MASKING_FACTOR_SLOT,
                slot: StorageSlot {
                    slot_type: StorageSlotType::Value { value_arity: 0 },
                    value: [Felt::from(DEFAULT_MASKING_FACTOR), Felt::ZERO, Felt::ZERO, Felt::ZERO],
                },
            },
            // for testing only,
            SlotItem {
                index: 100,
                slot: StorageSlot {
                    slot_type: StorageSlotType::Value { value_arity: 0 },
                    value: [Felt::new(15911754940807515092), Felt::new(127677651693142771), Felt::ZERO, Felt::ZERO],
                },
            },
            SlotItem {
                index: 101,
                slot: StorageSlot {
                    slot_type: StorageSlotType::Value { value_arity: 0 },
                    value: [Felt::new(15911754940807515092), Felt::new(7730906248022274125), Felt::ZERO, Felt::ZERO],
                },
            },
        ],
        vec![],
    )?;
    let account_vault = AssetVault::new(&[]).expect("error on empty vault");

    let account_seed = AccountId::get_account_seed(
        init_seed,
        account_type,
        miden_objects::accounts::AccountStorageType::OnChain,
        aze_player_account_code.root(),
        aze_player_account_storage.root(),
    )?;
    let account_id = AccountId::new(
        account_seed,
        aze_player_account_code.root(),
        aze_player_account_storage.root(),
    )?;
    Ok((
        Account::new(
            account_id,
            account_vault,
            aze_player_account_storage,
            aze_player_account_code,
            ZERO,
        ),
        account_seed,
    ))
}

const fn account_id(account_type: AccountType, storage: AccountStorageType, rest: u64) -> u64 {
    let mut id = 0;

    id ^= (storage as u64) << 62;
    id ^= (account_type as u64) << 60;
    id ^= rest;

    id
}

pub const ON_CHAIN: u64 = 0b00;
pub const OFF_CHAIN: u64 = 0b10;

#[repr(u64)]
pub enum AccountStorageType {
    OnChain = ON_CHAIN,
    OffChain = OFF_CHAIN,
}
