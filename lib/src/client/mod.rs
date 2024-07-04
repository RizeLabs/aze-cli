use crate::accounts::{create_basic_aze_game_account, create_basic_aze_player_account};
use crate::constants::{ CLIENT_CONFIG_FILE_NAME, AUTH_SEND_NOTE_SCRIPT };
use crate::notes::{
    create_play_bet_note, create_play_call_note, create_play_check_note, create_play_fold_note,
    create_play_raise_note, create_send_card_note, create_key_gen_note, create_shuffle_card_note,
    create_remask_note, create_set_cards_note, create_set_community_cards_note, create_unmask_note, 
    create_inter_unmask_note, create_send_unmasked_cards_note, create_set_hand_note, create_send_community_cards_note
};
use crate::utils::{create_aze_store_path, load_config};
use miden_client::client::rpc::NodeRpcClient;
use miden_client::store::data_store::{self, ClientDataStore};
use miden_client::{client, store};
use std::rc::Rc;
extern crate alloc;
use alloc::collections::{BTreeMap, BTreeSet};

use miden_client::{
    client::{
        accounts::{AccountStorageMode, AccountTemplate},
        get_random_coin,
        rpc::TonicRpcClient,
        store_authenticator::StoreAuthenticator,
        transactions::transaction_request,
        transactions::transaction_request::TransactionRequest,
        Client,
    },
    config::{ClientConfig, RpcConfig},
    errors::{ClientError, NodeRpcClientError},
    store::{sqlite_store::SqliteStore, NoteFilter, Store, TransactionFilter},
};

use crate::storage::GameStorageSlotData;
use miden_lib::AuthScheme;
use miden_objects::assets::Asset;
use miden_objects::crypto::rand::FeltRng;
use miden_objects::crypto::rand::RpoRandomCoin;
use miden_objects::notes::NoteType;
use miden_objects::{
    accounts::{Account, AccountData, AccountId, AccountStub, AccountType, AuthSecretKey},
    assembly::ProgramAst,
    assets::TokenSymbol,
    crypto::dsa::rpo_falcon512::SecretKey,
    Felt, Word,
};
use miden_tx::utils::Serializable;
use miden_tx::TransactionAuthenticator;
use rand::{rngs::ThreadRng, Rng};

pub type AzeClient = Client<
    TonicRpcClient,
    RpoRandomCoin,
    SqliteStore,
    StoreAuthenticator<RpoRandomCoin, SqliteStore>,
>;

#[derive(Clone)]
pub struct SendCardTransactionData {
    sender_account_id: AccountId,
    target_account_id: AccountId,
    cards: [[Felt; 4]; 2],
}

#[derive(Clone)]
pub struct GenPrivateKeyTransactionData {
    sender_account_id: AccountId,
    target_account_id: AccountId,
}

#[derive(Clone)]
pub struct ShuffleCardTransactionData {
    sender_account_id: AccountId,
    target_account_id: AccountId,
    player_data: [u64; 4],
}

#[derive(Clone)]
pub struct RemaskTransactionData {
    sender_account_id: AccountId,
    target_account_id: AccountId,
    cards: [[Felt; 4]; 52],
    player_data: [u64; 4],
}

#[derive(Clone)]
pub struct PlayBetTransactionData {
    sender_account_id: AccountId,
    target_account_id: AccountId,
    player_bet: u8,
}

#[derive(Clone)]
pub struct PlayRaiseTransactionData {
    sender_account_id: AccountId,
    target_account_id: AccountId,
    player_bet: u8,
}

#[derive(Clone)]
pub struct PlayCallTransactionData {
    sender_account_id: AccountId,
    target_account_id: AccountId,
}

#[derive(Clone)]
pub struct PlayFoldTransactionData {
    sender_account_id: AccountId,
    target_account_id: AccountId,
}

#[derive(Clone)]
pub struct PlayCheckTransactionData {
    sender_account_id: AccountId,
    target_account_id: AccountId,
}

#[derive(Clone)]
pub struct SetCardsTransactionData {
    sender_account_id: AccountId,
    target_account_id: AccountId,
    cards: [[Felt; 4]; 52],
}

#[derive(Clone)]
pub struct SendUnmaskedCardsTransactionData {
    sender_account_id: AccountId,
    target_account_id: AccountId,
    cards: [[Felt; 4]; 3],
}

#[derive(Clone)]
pub struct SetCommCardsTransactionData {
    sender_account_id: AccountId,
    target_account_id: AccountId,
    cards: [[Felt; 4]; 3],
    card_slot: u8
}

#[derive(Clone)]
pub struct InterUnmaskTransactionData {
    sender_account_id: AccountId,
    target_account_id: AccountId,
    cards: [[Felt; 4]; 3],
    requester_id: AccountId,
}

#[derive(Clone)]
pub struct SetHandTransactionData {
    sender_account_id: AccountId,
    target_account_id: AccountId,
    cards: [[Felt; 4]; 2],
    player_hand: u8,
}

#[derive(Clone)]
pub struct SendCommunityCardsTransactionData {
    sender_account_id: AccountId,
    target_account_id: AccountId,
    cards: [[Felt; 4]; 3],
    current_phase: u8,
}

impl GenPrivateKeyTransactionData {
    pub fn account_id(&self) -> AccountId {
        self.sender_account_id
    }
    pub fn new(
        sender_account_id: AccountId,
        target_account_id: AccountId,
    ) -> Self {
        Self {
            sender_account_id,
            target_account_id,
        }
    }
}

impl ShuffleCardTransactionData {
    pub fn account_id(&self) -> AccountId {
        self.sender_account_id
    }
    pub fn new(
        sender_account_id: AccountId,
        target_account_id: AccountId,
        player_data: [u64; 4],
    ) -> Self {
        Self {
            sender_account_id,
            target_account_id,
            player_data,
        }
    }
}

impl RemaskTransactionData {
    pub fn account_id(&self) -> AccountId {
        self.sender_account_id
    }
    pub fn new(
        sender_account_id: AccountId,
        target_account_id: AccountId,
        cards: &[[Felt; 4]; 52],
        player_data: [u64; 4],
    ) -> Self {
        Self {
            sender_account_id,
            target_account_id,
            cards: *cards,
            player_data,
        }
    }
}

impl SendCardTransactionData {
    pub fn account_id(&self) -> AccountId {
        self.sender_account_id
    }
    pub fn new(
        sender_account_id: AccountId,
        target_account_id: AccountId,
        cards: &[[Felt; 4]; 2],
    ) -> Self {
        Self {
            sender_account_id,
            target_account_id,
            cards: *cards,
        }
    }
}

impl PlayBetTransactionData {
    pub fn account_id(&self) -> AccountId {
        self.sender_account_id
    }
    pub fn new(
        sender_account_id: AccountId,
        target_account_id: AccountId,
        player_bet: u8,
    ) -> Self {
        Self {
            sender_account_id,
            target_account_id,
            player_bet,
        }
    }
}

impl PlayRaiseTransactionData {
    pub fn account_id(&self) -> AccountId {
        self.sender_account_id
    }
    pub fn new(
        sender_account_id: AccountId,
        target_account_id: AccountId,
        player_bet: u8,
    ) -> Self {
        Self {
            sender_account_id,
            target_account_id,
            player_bet,
        }
    }
}

impl PlayCallTransactionData {
    pub fn account_id(&self) -> AccountId {
        self.sender_account_id
    }
    pub fn new(sender_account_id: AccountId, target_account_id: AccountId) -> Self {
        Self {
            sender_account_id,
            target_account_id,
        }
    }
}

impl PlayFoldTransactionData {
    pub fn account_id(&self) -> AccountId {
        self.sender_account_id
    }
    pub fn new(sender_account_id: AccountId, target_account_id: AccountId) -> Self {
        Self {
            sender_account_id,
            target_account_id,
        }
    }
}

impl PlayCheckTransactionData {
    pub fn account_id(&self) -> AccountId {
        self.sender_account_id
    }
    pub fn new(sender_account_id: AccountId, target_account_id: AccountId) -> Self {
        Self {
            sender_account_id,
            target_account_id,
        }
    }
}

impl SetCardsTransactionData {
    pub fn account_id(&self) -> AccountId {
        self.sender_account_id
    }
    pub fn new(
        sender_account_id: AccountId,
        target_account_id: AccountId,
        cards: &[[Felt; 4]; 52],
    ) -> Self {
        Self {
            sender_account_id,
            target_account_id,
            cards: *cards,
        }
    }
}

impl SendUnmaskedCardsTransactionData {
    pub fn account_id(&self) -> AccountId {
        self.sender_account_id
    }
    pub fn new(
        sender_account_id: AccountId,
        target_account_id: AccountId,
        cards: &[[Felt; 4]; 3],
    ) -> Self {
        Self {
            sender_account_id,
            target_account_id,
            cards: *cards,
        }
    }
}

impl SetCommCardsTransactionData {
    pub fn account_id(&self) -> AccountId {
        self.sender_account_id
    }
    pub fn new(
        sender_account_id: AccountId,
        target_account_id: AccountId,
        cards: &[[Felt; 4]; 3],
        card_slot: u8
    ) -> Self {
        Self {
            sender_account_id,
            target_account_id,
            cards: *cards,
            card_slot
        }
    }
}

impl InterUnmaskTransactionData {
    pub fn account_id(&self) -> AccountId {
        self.sender_account_id
    }
    pub fn new(
        sender_account_id: AccountId,
        target_account_id: AccountId,
        cards: &[[Felt; 4]; 3],
        requester_id: AccountId,
    ) -> Self {
        Self {
            sender_account_id,
            target_account_id,
            cards: *cards,
            requester_id,
        }
    }
}

impl SetHandTransactionData {
    pub fn account_id(&self) -> AccountId {
        self.sender_account_id
    }
    pub fn new(
        sender_account_id: AccountId,
        target_account_id: AccountId,
        cards: &[[Felt; 4]; 2],
        player_hand: u8,
    ) -> Self {
        Self {
            sender_account_id,
            target_account_id,
            cards: *cards,
            player_hand,
        }
    }
}

impl SendCommunityCardsTransactionData {
    pub fn account_id(&self) -> AccountId {
        self.sender_account_id
    }
    pub fn new(
        sender_account_id: AccountId,
        target_account_id: AccountId,
        cards: &[[Felt; 4]; 3],
        current_phase: u8,
    ) -> Self {
        Self {
            sender_account_id,
            target_account_id,
            cards: *cards,
            current_phase,
        }
    }
}

pub trait AzeGameMethods {
    // fn get_tx_executor(&self) -> TransactionExecutor<ClientDataStore<D>>;
    fn store(&self) -> SqliteStore;
    fn get_random_coin(&self) -> RpoRandomCoin;
    fn build_aze_send_card_tx_request(
        &mut self,
        // auth_info: AuthSecretKey,
        transaction_template: AzeTransactionTemplate,
    ) -> Result<TransactionRequest, ClientError>;
    fn build_aze_key_gen_tx_request(
        &mut self,
        // auth_info: AuthSecretKey,
        transaction_template: AzeTransactionTemplate,
    ) -> Result<TransactionRequest, ClientError>;
    fn build_aze_shuffle_card_tx_request(
        &mut self,
        // auth_info: AuthSecretKey,
        transaction_template: AzeTransactionTemplate,
    ) -> Result<TransactionRequest, ClientError>;
    fn build_aze_remask_tx_request(
        &mut self,
        // auth_info: AuthSecretKey,
        transaction_template: AzeTransactionTemplate,
    ) -> Result<TransactionRequest, ClientError>;
    fn build_aze_play_bet_tx_request(
        &mut self,
        // auth_info: AuthSecretKey,
        transaction_template: AzeTransactionTemplate,
    ) -> Result<TransactionRequest, ClientError>;
    fn build_aze_play_raise_tx_request(
        &mut self,
        // auth_info: AuthSecretKey,
        transaction_template: AzeTransactionTemplate,
    ) -> Result<TransactionRequest, ClientError>;
    fn build_aze_play_call_tx_request(
        &mut self,
        // auth_info: AuthSecretKey,
        transaction_template: AzeTransactionTemplate,
    ) -> Result<TransactionRequest, ClientError>;
    fn build_aze_play_fold_tx_request(
        &mut self,
        // auth_info: AuthSecretKey,
        transaction_template: AzeTransactionTemplate,
    ) -> Result<TransactionRequest, ClientError>;
    fn build_aze_play_check_tx_request(
        &mut self,
        // auth_info: AuthSecretKey,
        transaction_template: AzeTransactionTemplate,
    ) -> Result<TransactionRequest, ClientError>;
    fn build_aze_set_cards_tx_request(
        &mut self,
        // auth_info: AuthSecretKey,
        transaction_template: AzeTransactionTemplate,
    ) -> Result<TransactionRequest, ClientError>;
    fn build_aze_send_unmasked_cards_tx_request(
        &mut self,
        // auth_info: AuthSecretKey,
        transaction_template: AzeTransactionTemplate,
    ) -> Result<TransactionRequest, ClientError>;
    fn build_aze_set_community_cards_tx_request(
        &mut self,
        // auth_info: AuthSecretKey,
        transaction_template: AzeTransactionTemplate,
    ) -> Result<TransactionRequest, ClientError>;
    fn build_aze_unmask_tx_request(
        &mut self,
        // auth_info: AuthSecretKey,
        transaction_template: AzeTransactionTemplate,
    ) -> Result<TransactionRequest, ClientError>;
    fn build_aze_inter_unmask_tx_request(
        &mut self,
        // auth_info: AuthSecretKey,
        transaction_template: AzeTransactionTemplate,
    ) -> Result<TransactionRequest, ClientError>;
    fn build_aze_set_hand_tx_request(
        &mut self,
        // auth_info: AuthSecretKey,
        transaction_template: AzeTransactionTemplate,
    ) -> Result<TransactionRequest, ClientError>;
    fn build_send_community_cards_tx_request(
        &mut self,
        // auth_info: AuthSecretKey,
        transaction_template: AzeTransactionTemplate,
    ) -> Result<TransactionRequest, ClientError>;
    fn new_game_account(
        &mut self,
        template: AzeAccountTemplate,
        slot_data: Option<GameStorageSlotData>,
    ) -> Result<(Account, Word), ClientError>;
    fn new_aze_game_account(
        &mut self,
        mutable_code: bool,
        rng: &mut ThreadRng,
        account_storage_mode: AccountStorageMode,
        slot_data: GameStorageSlotData,
    ) -> Result<(Account, Word), ClientError>;
    fn new_aze_player_account(
        &mut self,
        mutable_code: bool,
        rng: &mut ThreadRng,
        account_storage_mode: AccountStorageMode,
    ) -> Result<(Account, Word), ClientError>;
}

pub enum AzeAccountTemplate {
    PlayerAccount {
        mutable_code: bool,
        storage_mode: AccountStorageMode,
    },
    GameAccount {
        // need to change it and he would need to pass whole game storage
        mutable_code: bool,
        storage_mode: AccountStorageMode,
    },
}

pub fn create_aze_client() -> AzeClient {
    let mut current_dir = std::env::current_dir()
        .map_err(|err| err.to_string())
        .unwrap();
    current_dir.push(CLIENT_CONFIG_FILE_NAME);
    let client_config = load_config(current_dir.as_path()).unwrap();
    let store = {
        let sqlite_store = SqliteStore::new((&client_config).into()).unwrap();
        Rc::new(sqlite_store)
    };

    let rng = get_random_coin();

    let authenticator = StoreAuthenticator::new_with_rng(store.clone(), rng);
    AzeClient::new(
        TonicRpcClient::new(&client_config.rpc),
        rng,
        store,
        authenticator,
        true,
    )
}

impl<N: NodeRpcClient, R: FeltRng, S: Store, A: TransactionAuthenticator> AzeGameMethods
    for Client<N, R, S, A>
{
    fn store(&self) -> SqliteStore {
        let mut current_dir = std::env::current_dir()
            .map_err(|err| err.to_string())
            .unwrap();
        current_dir.push(CLIENT_CONFIG_FILE_NAME);
        let client_config = load_config(current_dir.as_path()).unwrap();

        let executor_store = SqliteStore::new((&client_config).into()).unwrap();
        executor_store
    }

    fn new_game_account(
        &mut self,
        template: AzeAccountTemplate,
        slot_data: Option<GameStorageSlotData>,
    ) -> Result<(Account, Word), ClientError> {
        let mut rng = rand::thread_rng();

        let account_and_seed = (match template {
            AzeAccountTemplate::PlayerAccount {
                mutable_code,
                storage_mode,
            } => self.new_aze_player_account(mutable_code, &mut rng, storage_mode),
            AzeAccountTemplate::GameAccount {
                mutable_code,
                storage_mode,
            } => {
                self.new_aze_game_account(mutable_code, &mut rng, storage_mode, slot_data.unwrap())
            }
        })?;

        Ok(account_and_seed)
    }

    fn new_aze_game_account(
        &mut self,
        mutable_code: bool, // will remove it later on
        rng: &mut ThreadRng,
        account_storage_mode: AccountStorageMode,
        slot_data: GameStorageSlotData,
    ) -> Result<(Account, Word), ClientError> {
        // if let AccountStorageMode::OnChain = account_storage_mode {
        //     todo!("Recording the account on chain is not supported yet");
        // }

        let key_pair = SecretKey::with_rng(rng);

        let auth_scheme: AuthScheme = AuthScheme::RpoFalcon512 {
            pub_key: key_pair.public_key(),
        };

        // we need to use an initial seed to create the wallet account
        let init_seed: [u8; 32] = rng.gen();

        let (account, seed) = create_basic_aze_game_account(
            init_seed,
            auth_scheme,
            AccountType::RegularAccountImmutableCode,
            slot_data,
        )
        .unwrap();

        // will do insert account later on since there is some type mismatch due to miden object crate
        self.insert_account(&account, Some(seed), &AuthSecretKey::RpoFalcon512(key_pair))?;
        Ok((account, seed))
    }

    fn new_aze_player_account(
        &mut self,
        mutable_code: bool,
        rng: &mut ThreadRng,
        account_storage_mode: AccountStorageMode,
    ) -> Result<(Account, Word), ClientError> {
        if let AccountStorageMode::OnChain = account_storage_mode {
            todo!("Recording the account on chain is not supported yet");
        }

        let key_pair = SecretKey::with_rng(rng);

        let auth_scheme: AuthScheme = AuthScheme::RpoFalcon512 {
            pub_key: key_pair.public_key(),
        };

        // we need to use an initial seed to create the wallet account
        let init_seed: [u8; 32] = rng.gen();

        let (account, seed) = create_basic_aze_player_account(
            init_seed,
            auth_scheme,
            AccountType::RegularAccountImmutableCode,
        )
        .unwrap();

        // will do insert account later on since there is some type mismatch due to miden object crate
        self.insert_account(&account, Some(seed), &AuthSecretKey::RpoFalcon512(key_pair))?;
        Ok((account, seed))
    }

    // TODO: include note_type as an argument here for now we are hardcoding it
    fn build_aze_send_card_tx_request(
        &mut self,
        // auth_info: AuthSecretKey,
        transaction_template: AzeTransactionTemplate,
    ) -> Result<TransactionRequest, ClientError> {
        let account_id = transaction_template.account_id();
        let account_auth = self.store().get_account_auth(account_id)?;

        let (sender_account_id, target_account_id, cards) = match transaction_template {
            AzeTransactionTemplate::SendCard(SendCardTransactionData {
                sender_account_id,
                target_account_id,
                cards,
            }) => (sender_account_id, target_account_id, cards),
            _ => panic!("Invalid transaction template"),
        };

        let slot1 = cards[0][2].as_int();
        let slot2 = cards[1][2].as_int();

        let random_coin = self.get_random_coin();

        let created_note = create_send_card_note(
            self,
            sender_account_id,
            target_account_id,
            NoteType::Public,
            random_coin,
            cards,
        )?;

        let recipient = created_note
            .recipient()
            .digest()
            .iter()
            .map(|x| x.as_int().to_string())
            .collect::<Vec<_>>()
            .join(".");

        let note_tag = created_note.metadata().tag().inner();

        // TODO: remove this hardcoded note type
        let note_type = NoteType::Public;

        let tx_script = format!(
            "\
            use.miden::account
            use.miden::contracts::auth::basic->auth_tx
            use.miden::contracts::wallets::basic->wallet
            use.miden::tx

            proc.tx_state_change
                push.254 exec.account::get_item
                add.1
                push.254 exec.account::set_item
                dropw dropw
            end

            proc.set_cards
                # => [card_index, rank, suit]
                push.0 dup movup.2
                # => [card_index, 0, 0, rank, suit]
                exec.account::set_item
                dropw dropw
            end

            begin
                dropw
                push.{slot1} swap drop
                call.set_cards
                push.{slot2} swap drop
                call.set_cards
                push.{recipient}
                push.{note_type}
                push.{tag}
                call.tx::create_note 
                drop drop dropw dropw
                call.tx_state_change dropw
                call.auth_tx::auth_tx_rpo_falcon512
                # => []
            end
            ",
            recipient = recipient,
            note_type = Felt::from(note_type as u8),
            tag = Felt::new(note_tag.into()),
            slot1 = slot1,
            slot2 = slot2
        );
        let tx_script = ProgramAst::parse(&tx_script).unwrap();

        let (pubkey_input, advice_map): (Word, Vec<Felt>) = match account_auth {
            AuthSecretKey::RpoFalcon512(key) => (
                key.public_key().into(),
                key.to_bytes()
                    .iter()
                    .map(|a| Felt::new(*a as u64))
                    .collect::<Vec<Felt>>(),
            ),
        };

        let tx_script = {
            let script_inputs = vec![(pubkey_input, advice_map)];
            self.compile_tx_script(tx_script, script_inputs, vec![])?
        };

        Ok(TransactionRequest::new(
            sender_account_id,
            BTreeMap::new(),
            vec![created_note],
            vec![],
            Some(tx_script),
        ))
    }

    fn build_aze_key_gen_tx_request(
        &mut self,
        // auth_info: AuthSecretKey,
        transaction_template: AzeTransactionTemplate,
    ) -> Result<TransactionRequest, ClientError> {
        let account_id = transaction_template.account_id();
        let account_auth = self.store().get_account_auth(account_id)?;

        let (sender_account_id, target_account_id) = match transaction_template {
            AzeTransactionTemplate::GenKey(GenPrivateKeyTransactionData {
                sender_account_id,
                target_account_id,
            }) => (sender_account_id, target_account_id),
            _ => panic!("Invalid transaction template"),
        };

        let random_coin = self.get_random_coin();

        let created_note = create_key_gen_note(
            self,
            sender_account_id,
            target_account_id,
            NoteType::Public,
            random_coin,
        )?;

        let recipient = created_note
            .recipient()
            .digest()
            .iter()
            .map(|x| x.as_int().to_string())
            .collect::<Vec<_>>()
            .join(".");

        let note_tag = created_note.metadata().tag().inner();

        // TODO: remove this hardcoded note type
        let note_type = NoteType::Public;

        let tx_script = ProgramAst::parse(
            &AUTH_SEND_NOTE_SCRIPT
                .replace("{recipient}", &recipient)
                .replace("{note_type}", &Felt::from(note_type as u8).to_string())
                .replace("{tag}", &Felt::new(note_tag.into()).to_string())
        ).unwrap();

        let (pubkey_input, advice_map): (Word, Vec<Felt>) = match account_auth {
            AuthSecretKey::RpoFalcon512(key) => (
                key.public_key().into(),
                key.to_bytes()
                    .iter()
                    .map(|a| Felt::new(*a as u64))
                    .collect::<Vec<Felt>>(),
            ),
        };

        let tx_script = {
            let script_inputs = vec![(pubkey_input, advice_map)];
            self.compile_tx_script(tx_script, script_inputs, vec![])?
        };


        Ok(TransactionRequest::new(
            sender_account_id,
            BTreeMap::new(),
            vec![created_note],
            vec![],
            Some(tx_script),
        ))
    }

    fn build_aze_shuffle_card_tx_request(
        &mut self,
        // auth_info: AuthSecretKey,
        transaction_template: AzeTransactionTemplate,
    ) -> Result<TransactionRequest, ClientError> {
        let account_id = transaction_template.account_id();
        let account_auth = self.store().get_account_auth(account_id)?;

        let (sender_account_id, target_account_id, player_data) = match transaction_template {
            AzeTransactionTemplate::ShuffleCard(ShuffleCardTransactionData {
                sender_account_id,
                target_account_id,
                player_data,
            }) => (sender_account_id, target_account_id, player_data),
            _ => panic!("Invalid transaction template"),
        };

        let random_coin = self.get_random_coin();

        let created_note = create_shuffle_card_note(
            self,
            sender_account_id,
            target_account_id,
            NoteType::Public,
            random_coin,
            player_data,
        )?;

        let recipient = created_note
            .recipient()
            .digest()
            .iter()
            .map(|x| x.as_int().to_string())
            .collect::<Vec<_>>()
            .join(".");

        let note_tag = created_note.metadata().tag().inner();
        let note_type = NoteType::Public;

        let tx_script = ProgramAst::parse(
            &AUTH_SEND_NOTE_SCRIPT
                .replace("{recipient}", &recipient)
                .replace("{note_type}", &Felt::from(note_type as u8).to_string())
                .replace("{tag}", &Felt::new(note_tag.into()).to_string())
        ).unwrap();
    
        let (pubkey_input, advice_map): (Word, Vec<Felt>) = match account_auth {
            AuthSecretKey::RpoFalcon512(key) => (
                key.public_key().into(),
                key.to_bytes()
                    .iter()
                    .map(|a| Felt::new(*a as u64))
                    .collect::<Vec<Felt>>(),
            ),
        };

        let tx_script = {
            let script_inputs = vec![(pubkey_input, advice_map)];
            self.compile_tx_script(tx_script, script_inputs, vec![])?
        };


        Ok(TransactionRequest::new(
            sender_account_id,
            BTreeMap::new(),
            vec![created_note],
            vec![],
            Some(tx_script),
        ))
    }

    fn build_aze_remask_tx_request(
        &mut self,
        // auth_info: AuthSecretKey,
        transaction_template: AzeTransactionTemplate,
    ) -> Result<TransactionRequest, ClientError> {
        let account_id = transaction_template.account_id();
        let account_auth = self.store().get_account_auth(account_id)?;

        let (sender_account_id, target_account_id, cards, player_data) = match transaction_template {
            AzeTransactionTemplate::Remask(RemaskTransactionData {
                sender_account_id,
                target_account_id,
                cards,
                player_data,
            }) => (sender_account_id, target_account_id, cards, player_data),
            _ => panic!("Invalid transaction template"),
        };

        let random_coin = self.get_random_coin();

        let created_note = create_remask_note(
            self,
            sender_account_id,
            target_account_id,
            NoteType::Public,
            random_coin,
            cards,
            player_data,
        )?;

        let recipient = created_note
            .recipient()
            .digest()
            .iter()
            .map(|x| x.as_int().to_string())
            .collect::<Vec<_>>()
            .join(".");

        let note_tag = created_note.metadata().tag().inner();
        let note_type = NoteType::Public;

        let tx_script = ProgramAst::parse(
            &AUTH_SEND_NOTE_SCRIPT
                .replace("{recipient}", &recipient)
                .replace("{note_type}", &Felt::from(note_type as u8).to_string())
                .replace("{tag}", &Felt::new(note_tag.into()).to_string())
        ).unwrap();

        let (pubkey_input, advice_map): (Word, Vec<Felt>) = match account_auth {
            AuthSecretKey::RpoFalcon512(key) => (
                key.public_key().into(),
                key.to_bytes()
                    .iter()
                    .map(|a| Felt::new(*a as u64))
                    .collect::<Vec<Felt>>(),
            ),
        };

        let tx_script = {
            let script_inputs = vec![(pubkey_input, advice_map)];
            self.compile_tx_script(tx_script, script_inputs, vec![])?
        };


        Ok(TransactionRequest::new(
            sender_account_id,
            BTreeMap::new(),
            vec![created_note],
            vec![],
            Some(tx_script),
        ))
    }

    fn build_aze_play_bet_tx_request(
        &mut self,
        // auth_info: AuthSecretKey,
        transaction_template: AzeTransactionTemplate,
    ) -> Result<TransactionRequest, ClientError> {
        let account_id = transaction_template.account_id();
        let account_auth = self.store().get_account_auth(account_id)?;

        let (sender_account_id, target_account_id, player_bet) = match transaction_template {
            AzeTransactionTemplate::PlayBet(PlayBetTransactionData {
                sender_account_id,
                target_account_id,
                player_bet,
            }) => (sender_account_id, target_account_id, player_bet),
            _ => panic!("Invalid transaction template"),
        };

        let random_coin = self.get_random_coin();

        let created_note = create_play_bet_note(
            self,
            sender_account_id,
            target_account_id,
            NoteType::Public,
            random_coin,
            player_bet,
        )?;

        let recipient = created_note
            .recipient()
            .digest()
            .iter()
            .map(|x| x.as_int().to_string())
            .collect::<Vec<_>>()
            .join(".");

        let note_tag = created_note.metadata().tag().inner();
        let note_type = NoteType::Public;

        let tx_script = ProgramAst::parse(
            &AUTH_SEND_NOTE_SCRIPT
                .replace("{recipient}", &recipient)
                .replace("{note_type}", &Felt::from(note_type as u8).to_string())
                .replace("{tag}", &Felt::new(note_tag.into()).to_string())
        ).unwrap();

        let (pubkey_input, advice_map): (Word, Vec<Felt>) = match account_auth {
            AuthSecretKey::RpoFalcon512(key) => (
                key.public_key().into(),
                key.to_bytes()
                    .iter()
                    .map(|a| Felt::new(*a as u64))
                    .collect::<Vec<Felt>>(),
            ),
        };

        let tx_script = {
            let script_inputs = vec![(pubkey_input, advice_map)];
            self.compile_tx_script(tx_script, script_inputs, vec![])?
        };

        

        Ok(TransactionRequest::new(
            sender_account_id,
            BTreeMap::new(),
            vec![created_note],
            vec![],
            Some(tx_script),
        ))
    }

    fn build_aze_play_raise_tx_request(
        &mut self,
        // auth_info: AuthSecretKey,
        transaction_template: AzeTransactionTemplate,
    ) -> Result<TransactionRequest, ClientError> {
        let account_id = transaction_template.account_id();
        let account_auth = self.store().get_account_auth(account_id)?;

        let (sender_account_id, target_account_id, player_bet) = match transaction_template {
            AzeTransactionTemplate::PlayRaise(PlayRaiseTransactionData {
                sender_account_id,
                target_account_id,
                player_bet,
            }) => (sender_account_id, target_account_id, player_bet),
            _ => panic!("Invalid transaction template"),
        };

        let random_coin = self.get_random_coin();

        let created_note = create_play_raise_note(
            self,
            sender_account_id,
            target_account_id,
            NoteType::Public,
            random_coin,
            player_bet,
        )?;

        let recipient = created_note
            .recipient()
            .digest()
            .iter()
            .map(|x| x.as_int().to_string())
            .collect::<Vec<_>>()
            .join(".");

        let note_tag = created_note.metadata().tag().inner();
        let note_type = NoteType::Public;

        let tx_script = ProgramAst::parse(
            &AUTH_SEND_NOTE_SCRIPT
                .replace("{recipient}", &recipient)
                .replace("{note_type}", &Felt::from(note_type as u8).to_string())
                .replace("{tag}", &Felt::new(note_tag.into()).to_string())
        ).unwrap();

        let (pubkey_input, advice_map): (Word, Vec<Felt>) = match account_auth {
            AuthSecretKey::RpoFalcon512(key) => (
                key.public_key().into(),
                key.to_bytes()
                    .iter()
                    .map(|a| Felt::new(*a as u64))
                    .collect::<Vec<Felt>>(),
            ),
        };

        let tx_script = {
            let script_inputs = vec![(pubkey_input, advice_map)];
            self.compile_tx_script(tx_script, script_inputs, vec![])?
        };

        Ok(TransactionRequest::new(
            sender_account_id,
            BTreeMap::new(),
            vec![created_note],
            vec![],
            Some(tx_script),
        ))
    }

    fn build_aze_play_call_tx_request(
        &mut self,
        // auth_info: AuthSecretKey,
        transaction_template: AzeTransactionTemplate,
    ) -> Result<TransactionRequest, ClientError> {
        let account_id = transaction_template.account_id();
        let account_auth = self.store().get_account_auth(account_id)?;

        let (sender_account_id, target_account_id) = match transaction_template {
            AzeTransactionTemplate::PlayCall(PlayCallTransactionData {
                sender_account_id,
                target_account_id,
            }) => (sender_account_id, target_account_id),
            _ => panic!("Invalid transaction template"),
        };

        let random_coin = self.get_random_coin();

        let created_note = create_play_call_note(
            self,
            sender_account_id,
            target_account_id,
            NoteType::Public,
            random_coin,
        )?;

        let recipient = created_note
            .recipient()
            .digest()
            .iter()
            .map(|x| x.as_int().to_string())
            .collect::<Vec<_>>()
            .join(".");

        let note_tag = created_note.metadata().tag().inner();
        let note_type = NoteType::Public;

        let tx_script = ProgramAst::parse(
            &AUTH_SEND_NOTE_SCRIPT
                .replace("{recipient}", &recipient)
                .replace("{note_type}", &Felt::from(note_type as u8).to_string())
                .replace("{tag}", &Felt::new(note_tag.into()).to_string())
        ).unwrap();

        let (pubkey_input, advice_map): (Word, Vec<Felt>) = match account_auth {
            AuthSecretKey::RpoFalcon512(key) => (
                key.public_key().into(),
                key.to_bytes()
                    .iter()
                    .map(|a| Felt::new(*a as u64))
                    .collect::<Vec<Felt>>(),
            ),
        };

        let tx_script = {
            let script_inputs = vec![(pubkey_input, advice_map)];
            self.compile_tx_script(tx_script, script_inputs, vec![])?
        };

        

        Ok(TransactionRequest::new(
            sender_account_id,
            BTreeMap::new(),
            vec![created_note],
            vec![],
            Some(tx_script),
        ))
    }

    fn build_aze_play_fold_tx_request(
        &mut self,
        // auth_info: AuthSecretKey,
        transaction_template: AzeTransactionTemplate,
    ) -> Result<TransactionRequest, ClientError> {
        let account_id = transaction_template.account_id();
        let account_auth = self.store().get_account_auth(account_id)?;

        let (sender_account_id, target_account_id) = match transaction_template {
            AzeTransactionTemplate::PlayFold(PlayFoldTransactionData {
                sender_account_id,
                target_account_id,
            }) => (sender_account_id, target_account_id),
            _ => panic!("Invalid transaction template"),
        };

        let random_coin = self.get_random_coin();

        let created_note = create_play_fold_note(
            self,
            sender_account_id,
            target_account_id,
            NoteType::Public,
            random_coin,
        )?;

        let recipient = created_note
            .recipient()
            .digest()
            .iter()
            .map(|x| x.as_int().to_string())
            .collect::<Vec<_>>()
            .join(".");

        let note_tag = created_note.metadata().tag().inner();
        let note_type = NoteType::Public;

        let tx_script = ProgramAst::parse(
            &AUTH_SEND_NOTE_SCRIPT
                .replace("{recipient}", &recipient)
                .replace("{note_type}", &Felt::from(note_type as u8).to_string())
                .replace("{tag}", &Felt::new(note_tag.into()).to_string())
        ).unwrap();

        let (pubkey_input, advice_map): (Word, Vec<Felt>) = match account_auth {
            AuthSecretKey::RpoFalcon512(key) => (
                key.public_key().into(),
                key.to_bytes()
                    .iter()
                    .map(|a| Felt::new(*a as u64))
                    .collect::<Vec<Felt>>(),
            ),
        };

        let tx_script = {
            let script_inputs = vec![(pubkey_input, advice_map)];
            self.compile_tx_script(tx_script, script_inputs, vec![])?
        };

        Ok(TransactionRequest::new(
            sender_account_id,
            BTreeMap::new(),
            vec![created_note],
            vec![],
            Some(tx_script),
        ))
    }

    fn build_aze_play_check_tx_request(
        &mut self,
        // auth_info: AuthSecretKey,
        transaction_template: AzeTransactionTemplate,
    ) -> Result<TransactionRequest, ClientError> {
        let account_id = transaction_template.account_id();
        let account_auth = self.store().get_account_auth(account_id)?;

        let (sender_account_id, target_account_id) = match transaction_template {
            AzeTransactionTemplate::PlayCheck(PlayCheckTransactionData {
                sender_account_id,
                target_account_id,
            }) => (sender_account_id, target_account_id),
            _ => panic!("Invalid transaction template"),
        };

        let random_coin = self.get_random_coin();

        let created_note = create_play_check_note(
            self,
            sender_account_id,
            target_account_id,
            NoteType::Public,
            random_coin,
        )?;

        let recipient = created_note
            .recipient()
            .digest()
            .iter()
            .map(|x| x.as_int().to_string())
            .collect::<Vec<_>>()
            .join(".");

        let note_tag = created_note.metadata().tag().inner();
        let note_type = NoteType::Public;

        let tx_script = ProgramAst::parse(
            &AUTH_SEND_NOTE_SCRIPT
                .replace("{recipient}", &recipient)
                .replace("{note_type}", &Felt::from(note_type as u8).to_string())
                .replace("{tag}", &Felt::new(note_tag.into()).to_string())
        ).unwrap();

        let (pubkey_input, advice_map): (Word, Vec<Felt>) = match account_auth {
            AuthSecretKey::RpoFalcon512(key) => (
                key.public_key().into(),
                key.to_bytes()
                    .iter()
                    .map(|a| Felt::new(*a as u64))
                    .collect::<Vec<Felt>>(),
            ),
        };

        let tx_script = {
            let script_inputs = vec![(pubkey_input, advice_map)];
            self.compile_tx_script(tx_script, script_inputs, vec![])?
        };

        Ok(TransactionRequest::new(
            sender_account_id,
            BTreeMap::new(),
            vec![created_note],
            vec![],
            Some(tx_script),
        ))
    }

    fn build_aze_set_cards_tx_request(
        &mut self,
        // auth_info: AuthSecretKey,
        transaction_template: AzeTransactionTemplate,
    ) -> Result<TransactionRequest, ClientError> {
        let account_id = transaction_template.account_id();
        let account_auth = self.store().get_account_auth(account_id)?;

        let (sender_account_id, target_account_id, cards) = match transaction_template {
            AzeTransactionTemplate::SetCards(SetCardsTransactionData {
                sender_account_id,
                target_account_id,
                cards,
            }) => (sender_account_id, target_account_id, cards),
            _ => panic!("Invalid transaction template"),
        };

        let random_coin = self.get_random_coin();

        let created_note = create_set_cards_note(
            self,
            sender_account_id,
            target_account_id,
            NoteType::Public,
            random_coin,
            cards,
        )?;

        let recipient = created_note
            .recipient()
            .digest()
            .iter()
            .map(|x| x.as_int().to_string())
            .collect::<Vec<_>>()
            .join(".");

        let note_tag = created_note.metadata().tag().inner();
        let note_type = NoteType::Public;

        let tx_script = ProgramAst::parse(
            &AUTH_SEND_NOTE_SCRIPT
                .replace("{recipient}", &recipient)
                .replace("{note_type}", &Felt::from(note_type as u8).to_string())
                .replace("{tag}", &Felt::new(note_tag.into()).to_string())
        ).unwrap();

        let (pubkey_input, advice_map): (Word, Vec<Felt>) = match account_auth {
            AuthSecretKey::RpoFalcon512(key) => (
                key.public_key().into(),
                key.to_bytes()
                    .iter()
                    .map(|a| Felt::new(*a as u64))
                    .collect::<Vec<Felt>>(),
            ),
        };

        let tx_script = {
            let script_inputs = vec![(pubkey_input, advice_map)];
            self.compile_tx_script(tx_script, script_inputs, vec![])?
        };

        Ok(TransactionRequest::new(
            sender_account_id,
            BTreeMap
            ::new(),
            vec![created_note],
            vec![],
            Some(tx_script),
        ))
    }

    fn build_aze_send_unmasked_cards_tx_request(
        &mut self,
        // auth_info: AuthSecretKey,
        transaction_template: AzeTransactionTemplate,
    ) -> Result<TransactionRequest, ClientError> {
        let account_id = transaction_template.account_id();
        let account_auth = self.store().get_account_auth(account_id)?;

        let (sender_account_id, target_account_id, cards) = match transaction_template {
            AzeTransactionTemplate::SendUnmaskedCards(SendUnmaskedCardsTransactionData {
                sender_account_id,
                target_account_id,
                cards,
            }) => (sender_account_id, target_account_id, cards),
            _ => panic!("Invalid transaction template"),
        };

        let random_coin = self.get_random_coin();

        let created_note = create_send_unmasked_cards_note(
            self,
            sender_account_id,
            target_account_id,
            NoteType::Public,
            random_coin,
            cards,
        )?;

        let recipient = created_note
            .recipient()
            .digest()
            .iter()
            .map(|x| x.as_int().to_string())
            .collect::<Vec<_>>()
            .join(".");

        let note_tag = created_note.metadata().tag().inner();
        let note_type = NoteType::Public;

        let tx_script = format!(
            "\
            use.miden::account
            use.miden::contracts::auth::basic->auth_tx
            use.miden::contracts::wallets::basic->wallet
            use.miden::tx

            proc.tx_state_change
                push.254 exec.account::get_item
                add.1
                push.254 exec.account::set_item
                dropw dropw
            end

            proc.set_requester_data
                # => [0, 0, 0, requester_id]
                push.102 exec.account::set_item
                dropw dropw 
            end

            begin
                dropw
                call.set_requester_data
                push.{recipient}
                push.{note_type}
                push.{tag}
                call.tx::create_note 
                drop drop dropw dropw
                call.tx_state_change dropw
                call.auth_tx::auth_tx_rpo_falcon512
                # => []
            end
            ",
            recipient = recipient,
            note_type = Felt::from(note_type as u8),
            tag = Felt::new(note_tag.into()),
        );
        let tx_script = ProgramAst::parse(&tx_script).unwrap();

        let (pubkey_input, advice_map): (Word, Vec<Felt>) = match account_auth {
            AuthSecretKey::RpoFalcon512(key) => (
                key.public_key().into(),
                key.to_bytes()
                    .iter()
                    .map(|a| Felt::new(*a as u64))
                    .collect::<Vec<Felt>>(),
            ),
        };

        let tx_script = {
            let script_inputs = vec![(pubkey_input, advice_map)];
            self.compile_tx_script(tx_script, script_inputs, vec![])?
        };

        Ok(TransactionRequest::new(
            sender_account_id,
            BTreeMap
            ::new(),
            vec![created_note],
            vec![],
            Some(tx_script),
        ))
    }

    fn build_aze_set_community_cards_tx_request(
        &mut self,
        // auth_info: AuthSecretKey,
        transaction_template: AzeTransactionTemplate,
    ) -> Result<TransactionRequest, ClientError> {
        let account_id = transaction_template.account_id();
        let account_auth = self.store().get_account_auth(account_id)?;

        let (sender_account_id, target_account_id, cards, card_slot) = match transaction_template {
            AzeTransactionTemplate::SetCommCards(SetCommCardsTransactionData {
                sender_account_id,
                target_account_id,
                cards,
                card_slot,
            }) => (sender_account_id, target_account_id, cards, card_slot),
            _ => panic!("Invalid transaction template"),
        };

        let random_coin = self.get_random_coin();

        let created_note = create_set_community_cards_note(
            self,
            sender_account_id,
            target_account_id,
            NoteType::Public,
            random_coin,
            cards,
            card_slot
        )?;

        let recipient = created_note
            .recipient()
            .digest()
            .iter()
            .map(|x| x.as_int().to_string())
            .collect::<Vec<_>>()
            .join(".");

        let note_tag = created_note.metadata().tag().inner();
        let note_type = NoteType::Public;

        let tx_script = ProgramAst::parse(
            &AUTH_SEND_NOTE_SCRIPT
                .replace("{recipient}", &recipient)
                .replace("{note_type}", &Felt::from(note_type as u8).to_string())
                .replace("{tag}", &Felt::new(note_tag.into()).to_string())
        ).unwrap();

        let (pubkey_input, advice_map): (Word, Vec<Felt>) = match account_auth {
            AuthSecretKey::RpoFalcon512(key) => (
                key.public_key().into(),
                key.to_bytes()
                    .iter()
                    .map(|a| Felt::new(*a as u64))
                    .collect::<Vec<Felt>>(),
            ),
        };

        let tx_script = {
            let script_inputs = vec![(pubkey_input, advice_map)];
            self.compile_tx_script(tx_script, script_inputs, vec![])?
        };

        Ok(TransactionRequest::new(
            sender_account_id,
            BTreeMap
            ::new(),
            vec![created_note],
            vec![],
            Some(tx_script),
        ))
    }

    fn build_send_community_cards_tx_request(
        &mut self,
        // auth_info: AuthSecretKey,
        transaction_template: AzeTransactionTemplate,
    ) -> Result<TransactionRequest, ClientError> {
        let account_id = transaction_template.account_id();
        let account_auth = self.store().get_account_auth(account_id)?;

        let (sender_account_id, target_account_id, cards, current_phase) = match transaction_template {
            AzeTransactionTemplate::SendCommunityCards(SendCommunityCardsTransactionData {
                sender_account_id,
                target_account_id,
                cards,
                current_phase,
            }) => (sender_account_id, target_account_id, cards, current_phase),
            _ => panic!("Invalid transaction template"),
        };

        let random_coin = self.get_random_coin();

        let created_note = create_send_community_cards_note(
            self,
            sender_account_id,
            target_account_id,
            NoteType::Public,
            random_coin,
            cards,
            current_phase
        )?;

        let recipient = created_note
            .recipient()
            .digest()
            .iter()
            .map(|x| x.as_int().to_string())
            .collect::<Vec<_>>()
            .join(".");

        let note_tag = created_note.metadata().tag().inner();
        let note_type = NoteType::Public;

        let tx_script = ProgramAst::parse(
            &AUTH_SEND_NOTE_SCRIPT
                .replace("{recipient}", &recipient)
                .replace("{note_type}", &Felt::from(note_type as u8).to_string())
                .replace("{tag}", &Felt::new(note_tag.into()).to_string())
        ).unwrap();

        let (pubkey_input, advice_map): (Word, Vec<Felt>) = match account_auth {
            AuthSecretKey::RpoFalcon512(key) => (
                key.public_key().into(),
                key.to_bytes()
                    .iter()
                    .map(|a| Felt::new(*a as u64))
                    .collect::<Vec<Felt>>(),
            ),
        };

        let tx_script = {
            let script_inputs = vec![(pubkey_input, advice_map)];
            self.compile_tx_script(tx_script, script_inputs, vec![])?
        };

        Ok(TransactionRequest::new(
            sender_account_id,
            BTreeMap
            ::new(),
            vec![created_note],
            vec![],
            Some(tx_script),
        ))
    }

    fn build_aze_unmask_tx_request(
        &mut self,
        // auth_info: AuthSecretKey,
        transaction_template: AzeTransactionTemplate,
    ) -> Result<TransactionRequest, ClientError> {
        let account_id = transaction_template.account_id();
        let account_auth = self.store().get_account_auth(account_id)?;

        let (sender_account_id, target_account_id, cards, card_slot) = match transaction_template {
            AzeTransactionTemplate::SetCommCards(SetCommCardsTransactionData {
                sender_account_id,
                target_account_id,
                cards,
                card_slot,
                ..
            }) => (sender_account_id, target_account_id, cards, card_slot),
            _ => panic!("Invalid transaction template"),
        };

        let random_coin = self.get_random_coin();

        let created_note = create_unmask_note(
            self,
            sender_account_id,
            target_account_id,
            NoteType::Public,
            random_coin,
            cards,
            card_slot
        )?;

        let recipient = created_note
            .recipient()
            .digest()
            .iter()
            .map(|x| x.as_int().to_string())
            .collect::<Vec<_>>()
            .join(".");

        let note_tag = created_note.metadata().tag().inner();
        let note_type = NoteType::Public;

        let tx_script = ProgramAst::parse(
            &AUTH_SEND_NOTE_SCRIPT
                .replace("{recipient}", &recipient)
                .replace("{note_type}", &Felt::from(note_type as u8).to_string())
                .replace("{tag}", &Felt::new(note_tag.into()).to_string())
        ).unwrap();

        let (pubkey_input, advice_map): (Word, Vec<Felt>) = match account_auth {
            AuthSecretKey::RpoFalcon512(key) => (
                key.public_key().into(),
                key.to_bytes()
                    .iter()
                    .map(|a| Felt::new(*a as u64))
                    .collect::<Vec<Felt>>(),
            ),
        };

        let tx_script = {
            let script_inputs = vec![(pubkey_input, advice_map)];
            self.compile_tx_script(tx_script, script_inputs, vec![])?
        };

        Ok(TransactionRequest::new(
            sender_account_id,
            BTreeMap::new(),
            vec![created_note],
            vec![],
            Some(tx_script),
        ))
    }

    fn build_aze_inter_unmask_tx_request(
        &mut self,
        // auth_info: AuthSecretKey,
        transaction_template: AzeTransactionTemplate,
    ) -> Result<TransactionRequest, ClientError> {
        let account_id = transaction_template.account_id();
        let account_auth = self.store().get_account_auth(account_id)?;

        let (sender_account_id, target_account_id, cards, requester_id) = match transaction_template {
            AzeTransactionTemplate::InterUnmask(InterUnmaskTransactionData {
                sender_account_id,
                target_account_id,
                cards,
                requester_id,
                ..
            }) => (sender_account_id, target_account_id, cards, requester_id),
            _ => panic!("Invalid transaction template"),
        };

        let random_coin = self.get_random_coin();

        let created_note = create_inter_unmask_note(
            self,
            sender_account_id,
            target_account_id,
            NoteType::Public,
            random_coin,
            cards,
            requester_id,
        )?;

        let recipient = created_note
            .recipient()
            .digest()
            .iter()
            .map(|x| x.as_int().to_string())
            .collect::<Vec<_>>()
            .join(".");

        let note_tag = created_note.metadata().tag().inner();
        let note_type = NoteType::Public;

        let tx_script = ProgramAst::parse(
            &AUTH_SEND_NOTE_SCRIPT
                .replace("{recipient}", &recipient)
                .replace("{note_type}", &Felt::from(note_type as u8).to_string())
                .replace("{tag}", &Felt::new(note_tag.into()).to_string())
        ).unwrap();

        let (pubkey_input, advice_map): (Word, Vec<Felt>) = match account_auth {
            AuthSecretKey::RpoFalcon512(key) => (
                key.public_key().into(),
                key.to_bytes()
                    .iter()
                    .map(|a| Felt::new(*a as u64))
                    .collect::<Vec<Felt>>(),
            ),
        };

        let tx_script = {
            let script_inputs = vec![(pubkey_input, advice_map)];
            self.compile_tx_script(tx_script, script_inputs, vec![])?
        };

        Ok(TransactionRequest::new(
            sender_account_id,
            BTreeMap::new(),
            vec![created_note],
            vec![],
            Some(tx_script),
        ))
    }

    fn build_aze_set_hand_tx_request(
        &mut self,
        // auth_info: AuthSecretKey,
        transaction_template: AzeTransactionTemplate,
    ) -> Result<TransactionRequest, ClientError> {
        let account_id = transaction_template.account_id();
        let account_auth = self.store().get_account_auth(account_id)?;

        let (sender_account_id, target_account_id, cards, player_hand) = match transaction_template {
            AzeTransactionTemplate::SetHand(SetHandTransactionData {
                sender_account_id,
                target_account_id,
                cards,
                player_hand,
            }) => (sender_account_id, target_account_id, cards, player_hand),
            _ => panic!("Invalid transaction template"),
        };

        let random_coin = self.get_random_coin();

        let created_note = create_set_hand_note(
            self,
            sender_account_id,
            target_account_id,
            NoteType::Public,
            random_coin,
            cards,
            player_hand,
        )?;

        let recipient = created_note
            .recipient()
            .digest()
            .iter()
            .map(|x| x.as_int().to_string())
            .collect::<Vec<_>>()
            .join(".");

        let note_tag = created_note.metadata().tag().inner();
        let note_type = NoteType::Public;

        let tx_script = ProgramAst::parse(
            &AUTH_SEND_NOTE_SCRIPT
                .replace("{recipient}", &recipient)
                .replace("{note_type}", &Felt::from(note_type as u8).to_string())
                .replace("{tag}", &Felt::new(note_tag.into()).to_string())
        ).unwrap();

        let (pubkey_input, advice_map): (Word, Vec<Felt>) = match account_auth {
            AuthSecretKey::RpoFalcon512(key) => (
                key.public_key().into(),
                key.to_bytes()
                    .iter()
                    .map(|a| Felt::new(*a as u64))
                    .collect::<Vec<Felt>>(),
            ),
        };

        let tx_script = {
            let script_inputs = vec![(pubkey_input, advice_map)];
            self.compile_tx_script(tx_script, script_inputs, vec![])?
        };

        Ok(TransactionRequest::new(
            sender_account_id,
            BTreeMap
            ::new(),
            vec![created_note],
            vec![],
            Some(tx_script),
        ))
    }

    fn get_random_coin(&self) -> RpoRandomCoin {
        // TODO: Initialize coin status once along with the client and persist status for retrieval
        let mut rng = rand::thread_rng();
        let coin_seed: [u64; 4] = rng.gen();

        RpoRandomCoin::new(coin_seed.map(Felt::new))
    }
}

//implement a new transaction template
pub enum AzeTransactionTemplate {
    SendCard(SendCardTransactionData),
    PlayBet(PlayBetTransactionData),
    PlayRaise(PlayRaiseTransactionData),
    PlayCall(PlayCallTransactionData),
    PlayFold(PlayFoldTransactionData),
    PlayCheck(PlayCheckTransactionData),
    GenKey(GenPrivateKeyTransactionData),
    ShuffleCard(ShuffleCardTransactionData),
    Remask(RemaskTransactionData),
    SetCards(SetCardsTransactionData),
    SetCommCards(SetCommCardsTransactionData),
    InterUnmask(InterUnmaskTransactionData),
    SendUnmaskedCards(SendUnmaskedCardsTransactionData),
    SetHand(SetHandTransactionData),
    SendCommunityCards(SendCommunityCardsTransactionData),
}

impl AzeTransactionTemplate {
    //returns the executor account id
    pub fn account_id(&self) -> AccountId {
        match self {
            AzeTransactionTemplate::SendCard(p) => p.account_id(),
            AzeTransactionTemplate::PlayBet(p) => p.account_id(),
            AzeTransactionTemplate::PlayRaise(p) => p.account_id(),
            AzeTransactionTemplate::PlayCall(p) => p.account_id(),
            AzeTransactionTemplate::PlayFold(p) => p.account_id(),
            AzeTransactionTemplate::PlayCheck(p) => p.account_id(),
            AzeTransactionTemplate::GenKey(p) => p.account_id(),
            AzeTransactionTemplate::ShuffleCard(p) => p.account_id(),
            AzeTransactionTemplate::Remask(p) => p.account_id(),
            AzeTransactionTemplate::SetCards(p) => p.account_id(),
            AzeTransactionTemplate::SetCommCards(p) => p.account_id(),
            AzeTransactionTemplate::InterUnmask(p) => p.account_id(),
            AzeTransactionTemplate::SendUnmaskedCards(p) => p.account_id(),
            AzeTransactionTemplate::SetHand(p) => p.account_id(),
            AzeTransactionTemplate::SendCommunityCards(p) => p.account_id(),
        }
    }
}

pub(crate) fn prepare_word(word: &Word) -> String {
    word.iter()
        .map(|x| x.as_int().to_string())
        .collect::<Vec<_>>()
        .join(".")
}
