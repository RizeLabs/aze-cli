use crate::client::AzeClient;
use crate::constants::{ BUY_IN_AMOUNT, TRANSFER_AMOUNT, FLOP_NO_OF_CARDS, SUIT_SIZE };
use crate::executor::execute_tx_and_sync;
use miden_client::client::Client;
use miden_client::{
    client::{
        rpc::NodeRpcClient,
        transactions::transaction_request::{TransactionRequest, TransactionTemplate},
    },
    store::Store,
};
use miden_lib::transaction::TransactionKernel;
use miden_objects::{
    accounts::{Account, AccountCode, AccountId, AccountStorage, StorageSlotType},
    assembly::{ModuleAst, ProgramAst},
    assets::{Asset, AssetVault, FungibleAsset},
    crypto::rand::{FeltRng, RpoRandomCoin},
    notes::{
        Note, NoteAssets, NoteExecutionHint, NoteInputs, NoteMetadata, NoteRecipient, NoteScript,
        NoteTag, NoteType,
    },
    transaction::{InputNote, TransactionArgs},
    Felt, FieldElement, NoteError, Word, ZERO,
};
use miden_tx::TransactionAuthenticator;
use std::rc::Rc;

pub fn create_send_card_note<
    R: FeltRng,
    N: NodeRpcClient,
    S: Store,
    A: TransactionAuthenticator,
>(
    client: &mut Client<N, R, S, A>,
    sender_account_id: AccountId,
    target_account_id: AccountId,
    note_type: NoteType,
    mut rng: RpoRandomCoin,
    cards: [[Felt; 4]; 2],
) -> Result<Note, NoteError> {
    let note_script = include_str!("../../contracts/notes/game/deal.masm");
    // TODO: hide it under feature flag debug (.with_debug_mode(true))
    let script_ast = ProgramAst::parse(note_script).unwrap();
    let note_script = client.compile_note_script(script_ast, vec![]).unwrap();

    let card_1 = cards[0];
    let card_2 = cards[1];

    let mut inputs = vec![card_1[0], card_1[1], Felt::ZERO, Felt::ZERO];
    inputs = [inputs, vec![card_2[0], card_2[1], Felt::ZERO, Felt::ZERO]].concat();
    
    let note_inputs = NoteInputs::new(inputs).unwrap();
    let tag: u64 = target_account_id.into();
    let tag: NoteTag = ((tag + tag) as u32).into();
    let serial_num = rng.draw_word();
    let aux = ZERO;

    // TODO: For now hardcoding notes to be public, + Also find out what encrypted notes means
    let metadata = NoteMetadata::new(sender_account_id, NoteType::Public, tag, aux)?;
    let vault = NoteAssets::new(vec![])?;
    let recipient = NoteRecipient::new(serial_num, note_script, note_inputs);

    Ok(Note::new(vault, metadata, recipient))
}

pub fn create_key_gen_note<
    R: FeltRng,
    N: NodeRpcClient,
    S: Store,
    A: TransactionAuthenticator,
>(
    client: &mut Client<N, R, S, A>,
    sender_account_id: AccountId,
    target_account_id: AccountId,
    note_type: NoteType,
    mut rng: RpoRandomCoin,
) -> Result<Note, NoteError> {
    let note_script = include_str!("../../contracts/notes/game/genkey.masm");
    // TODO: hide it under feature flag debug (.with_debug_mode(true))
    let script_ast = ProgramAst::parse(note_script).unwrap();
    let note_script = client.compile_note_script(script_ast, vec![]).unwrap();

    let note_inputs = NoteInputs::new(vec![]).unwrap();
    let tag: u64 = target_account_id.into();
    let tag: NoteTag = ((tag + tag) as u32).into();
    let serial_num = rng.draw_word();
    let aux = ZERO;

    // TODO: For now hardcoding notes to be public, + Also find out what encrypted notes means
    let metadata = NoteMetadata::new(sender_account_id, NoteType::Public, tag, aux)?;
    let vault = NoteAssets::new(vec![])?;
    let recipient = NoteRecipient::new(serial_num, note_script, note_inputs);

    Ok(Note::new(vault, metadata, recipient))
}

pub fn create_shuffle_card_note<R: FeltRng, N: NodeRpcClient, S: Store, A: TransactionAuthenticator>(
    client: &mut Client<N, R, S, A>,
    sender_account_id: AccountId,
    target_account_id: AccountId,
    note_type: NoteType,
    mut rng: RpoRandomCoin,
    player_data: [u64; 4],
) -> Result<Note, NoteError> {
    let note_script = include_str!("../../contracts/notes/game/shuffle.masm");
    let script_ast = ProgramAst::parse(note_script).unwrap();
    let note_script = client.compile_note_script(script_ast, vec![]).unwrap();

    let mut cards = vec![];
    for card_number in 1..53 {
            cards = [cards, vec![Felt::from(card_number as u8)]].concat();
    }
    let player_data = player_data.iter().map(|x| Felt::new(*x)).collect::<Vec<Felt>>();
    let inputs = [cards, player_data].concat();
    
    
    let note_inputs = NoteInputs::new(inputs).unwrap();
    let tag: u64 = target_account_id.into();
    let tag: NoteTag = ((tag + tag) as u32).into();
    let serial_num = rng.draw_word();
    let aux = ZERO;

    let metadata = NoteMetadata::new(sender_account_id, NoteType::Public, tag, aux)?;
    let vault = NoteAssets::new(vec![])?;
    let recipient = NoteRecipient::new(serial_num, note_script, note_inputs);

    Ok(Note::new(vault, metadata, recipient))
}

pub fn create_remask_note<R: FeltRng, N: NodeRpcClient, S: Store, A: TransactionAuthenticator>(
    client: &mut Client<N, R, S, A>,
    sender_account_id: AccountId,
    target_account_id: AccountId,
    note_type: NoteType,
    mut rng: RpoRandomCoin,
    cards: [[Felt; 4]; 52],
    player_data: [u64; 4],
) -> Result<Note, NoteError> {
    let note_script = include_str!("../../contracts/notes/game/remask.masm");
    let script_ast = ProgramAst::parse(note_script).unwrap();
    let note_script = client.compile_note_script(script_ast, vec![]).unwrap();

    let mut encrypted_cards = vec![];
    for card in cards.iter() {
        encrypted_cards = [encrypted_cards, vec![card[1]]].concat();
    }
    encrypted_cards = [encrypted_cards, vec![cards[0][0]]].concat();
    let player_data = player_data.iter().map(|x| Felt::new(*x)).collect::<Vec<Felt>>();
    let inputs = [encrypted_cards, vec![Felt::ZERO, Felt::ZERO, Felt::ZERO], player_data].concat();
    

    let note_inputs = NoteInputs::new(inputs).unwrap();
    let tag: u64 = target_account_id.into();
    let tag: NoteTag = ((tag + tag) as u32).into();
    let serial_num = rng.draw_word();
    let aux = ZERO;

    let metadata = NoteMetadata::new(sender_account_id, NoteType::Public, tag, aux)?;
    let vault = NoteAssets::new(vec![])?;
    let recipient = NoteRecipient::new(serial_num, note_script, note_inputs);

    Ok(Note::new(vault, metadata, recipient))
}

pub fn create_play_bet_note<R: FeltRng, N: NodeRpcClient, S: Store, A: TransactionAuthenticator>(
    client: &mut Client<N, R, S, A>,
    sender_account_id: AccountId,
    target_account_id: AccountId,
    note_type: NoteType,
    mut rng: RpoRandomCoin,
    player_bet: u8,
) -> Result<Note, NoteError> {
    let note_script = include_str!("../../contracts/notes/game/bet.masm");
    let script_ast = ProgramAst::parse(note_script).unwrap();
    let note_script = client.compile_note_script(script_ast, vec![]).unwrap();

    let inputs = vec![Felt::from(player_bet)];
    let note_inputs = NoteInputs::new(inputs).unwrap();
    let tag: u64 = target_account_id.into();
    let tag: NoteTag = ((tag + tag) as u32).into();
    let serial_num = rng.draw_word();
    let aux = ZERO;

    let metadata = NoteMetadata::new(sender_account_id, NoteType::Public, tag, aux)?;
    let vault = NoteAssets::new(vec![])?;
    let recipient = NoteRecipient::new(serial_num, note_script, note_inputs);

    Ok(Note::new(vault, metadata, recipient))
}

pub fn create_play_raise_note<
    R: FeltRng,
    N: NodeRpcClient,
    S: Store,
    A: TransactionAuthenticator,
>(
    client: &mut Client<N, R, S, A>,
    sender_account_id: AccountId,
    target_account_id: AccountId,
    note_type: NoteType,
    mut rng: RpoRandomCoin,
    player_bet: u8,
) -> Result<Note, NoteError> {
    let note_script = include_str!("../../contracts/notes/game/raise.masm");
    let script_ast = ProgramAst::parse(note_script).unwrap();
    let note_script = client.compile_note_script(script_ast, vec![]).unwrap();

    let inputs = vec![Felt::from(player_bet)];
    let note_inputs = NoteInputs::new(inputs).unwrap();
    let tag: u64 = target_account_id.into();
    let tag: NoteTag = ((tag + tag) as u32).into();
    let serial_num = rng.draw_word();
    let aux = ZERO;

    let metadata = NoteMetadata::new(sender_account_id, NoteType::Public, tag, aux)?;
    let vault = NoteAssets::new(vec![])?;
    let recipient = NoteRecipient::new(serial_num, note_script, note_inputs);

    Ok(Note::new(vault, metadata, recipient))
}

pub fn create_play_call_note<
    R: FeltRng,
    N: NodeRpcClient,
    S: Store,
    A: TransactionAuthenticator,
>(
    client: &mut Client<N, R, S, A>,
    sender_account_id: AccountId,
    target_account_id: AccountId,
    note_type: NoteType,
    mut rng: RpoRandomCoin,
) -> Result<Note, NoteError> {
    let note_script = include_str!("../../contracts/notes/game/call.masm");
    let script_ast = ProgramAst::parse(note_script).unwrap();
    let note_script = client.compile_note_script(script_ast, vec![]).unwrap();

    let note_inputs = NoteInputs::new(vec![]).unwrap();
    let tag: u64 = target_account_id.into();
    let tag: NoteTag = ((tag + tag) as u32).into();
    let serial_num = rng.draw_word();
    let aux = ZERO;

    let metadata = NoteMetadata::new(sender_account_id, NoteType::Public, tag, aux)?;
    let vault = NoteAssets::new(vec![])?;
    let recipient = NoteRecipient::new(serial_num, note_script, note_inputs);

    Ok(Note::new(vault, metadata, recipient))
}

pub fn create_play_fold_note<
    R: FeltRng,
    N: NodeRpcClient,
    S: Store,
    A: TransactionAuthenticator,
>(
    client: &mut Client<N, R, S, A>,
    sender_account_id: AccountId,
    target_account_id: AccountId,
    note_type: NoteType,
    mut rng: RpoRandomCoin,
) -> Result<Note, NoteError> {
    let note_script = include_str!("../../contracts/notes/game/fold.masm");
    let script_ast = ProgramAst::parse(note_script).unwrap();
    let note_script = client.compile_note_script(script_ast, vec![]).unwrap();

    let note_inputs = NoteInputs::new(vec![]).unwrap();
    let tag: u64 = target_account_id.into();
    let tag: NoteTag = ((tag + tag) as u32).into();
    let serial_num = rng.draw_word();
    let aux = ZERO;

    let metadata = NoteMetadata::new(sender_account_id, NoteType::Public, tag, aux)?;
    let vault = NoteAssets::new(vec![])?;
    let recipient = NoteRecipient::new(serial_num, note_script, note_inputs);

    Ok(Note::new(vault, metadata, recipient))
}

pub fn create_play_check_note<
    R: FeltRng,
    N: NodeRpcClient,
    S: Store,
    A: TransactionAuthenticator,
>(
    client: &mut Client<N, R, S, A>,
    sender_account_id: AccountId,
    target_account_id: AccountId,
    note_type: NoteType,
    mut rng: RpoRandomCoin,
) -> Result<Note, NoteError> {
    let note_script = include_str!("../../contracts/notes/game/check.masm");
    let script_ast = ProgramAst::parse(note_script).unwrap();
    let note_script = client.compile_note_script(script_ast, vec![]).unwrap();

    let note_inputs = NoteInputs::new(vec![]).unwrap();
    let tag: u64 = target_account_id.into();
    let tag: NoteTag = ((tag + tag) as u32).into();
    let serial_num = rng.draw_word();
    let aux = ZERO;

    let metadata = NoteMetadata::new(sender_account_id, NoteType::Public, tag, aux)?;
    let vault = NoteAssets::new(vec![])?;
    let recipient = NoteRecipient::new(serial_num, note_script, note_inputs);

    Ok(Note::new(vault, metadata, recipient))
}

pub fn create_set_cards_note< 
    R: FeltRng,
    N: NodeRpcClient,
    S: Store,
    A: TransactionAuthenticator,
>(
    client: &mut Client<N, R, S, A>,
    sender_account_id: AccountId,
    target_account_id: AccountId,
    note_type: NoteType,
    mut rng: RpoRandomCoin,
    cards: [[Felt; 4]; 52],
) -> Result<Note, NoteError> {
    let note_script = include_str!("../../contracts/notes/game/set_cards.masm");
    let script_ast = ProgramAst::parse(note_script).unwrap();
    let note_script = client.compile_note_script(script_ast, vec![]).unwrap();

    let mut inputs = vec![];
    for card in cards.iter() {
        inputs = [inputs, vec![card[1]]].concat();
    }
    inputs = [inputs, vec![cards[0][0]]].concat();
    

    let note_inputs = NoteInputs::new(inputs).unwrap();
    let tag: u64 = target_account_id.into();
    let tag: NoteTag = ((tag + tag) as u32).into();
    let serial_num = rng.draw_word();
    let aux = ZERO;

    let metadata = NoteMetadata::new(sender_account_id, NoteType::Public, tag, aux)?;
    let vault = NoteAssets::new(vec![])?;
    let recipient = NoteRecipient::new(serial_num, note_script, note_inputs);

    Ok(Note::new(vault, metadata, recipient))
}

pub fn create_set_community_cards_note< 
    R: FeltRng,
    N: NodeRpcClient,
    S: Store,
    A: TransactionAuthenticator,
>(
    client: &mut Client<N, R, S, A>,
    sender_account_id: AccountId,
    target_account_id: AccountId,
    note_type: NoteType,
    mut rng: RpoRandomCoin,
    cards: [[Felt; 4]; 3],
    card_slot: u8,
) -> Result<Note, NoteError> {
    let note_script = include_str!("../../contracts/notes/game/set_community_cards.masm");
    let script_ast = ProgramAst::parse(note_script).unwrap();
    let note_script = client.compile_note_script(script_ast, vec![]).unwrap();

    let mut inputs = vec![];
    for card in cards.iter().take(FLOP_NO_OF_CARDS as usize) {
        inputs = [inputs, vec![card[0], Felt::ZERO, Felt::ZERO, Felt::ZERO]].concat();
    }
    inputs = [inputs, vec![Felt::from(card_slot)]].concat();

    let note_inputs = NoteInputs::new(inputs).unwrap();
    let tag: u64 = target_account_id.into();
    let tag: NoteTag = ((tag + tag) as u32).into();
    let serial_num = rng.draw_word();
    let aux = ZERO;

    let metadata = NoteMetadata::new(sender_account_id, NoteType::Public, tag, aux)?;
    let vault = NoteAssets::new(vec![])?;
    let recipient = NoteRecipient::new(serial_num, note_script, note_inputs);

    Ok(Note::new(vault, metadata, recipient))
}

pub fn create_send_unmasked_cards_note<R: FeltRng, N: NodeRpcClient, S: Store, A: TransactionAuthenticator>(
    client: &mut Client<N, R, S, A>,
    sender_account_id: AccountId,
    target_account_id: AccountId,
    note_type: NoteType,
    mut rng: RpoRandomCoin,
    cards: [[Felt; 4]; 3],
) -> Result<Note, NoteError> {
    let note_script = include_str!("../../contracts/notes/game/send_unmasked_cards.masm");
    let script_ast = ProgramAst::parse(note_script).unwrap();
    let note_script = client.compile_note_script(script_ast, vec![]).unwrap();

    let mut unmasked_cards = vec![];
    for card in cards.iter() {
        unmasked_cards = [unmasked_cards, vec![card[0], card[1], Felt::ZERO, Felt::ZERO]].concat();
    }

    let note_inputs = NoteInputs::new(unmasked_cards).unwrap();
    let tag: u64 = target_account_id.into();
    let tag: NoteTag = ((tag + tag) as u32).into();
    let serial_num = rng.draw_word();
    let aux = ZERO;

    let metadata = NoteMetadata::new(sender_account_id, NoteType::Public, tag, aux)?;
    let vault = NoteAssets::new(vec![])?;
    let recipient = NoteRecipient::new(serial_num, note_script, note_inputs);

    Ok(Note::new(vault, metadata, recipient))
}

pub fn create_unmask_note<R: FeltRng, N: NodeRpcClient, S: Store, A: TransactionAuthenticator>(
    client: &mut Client<N, R, S, A>,
    sender_account_id: AccountId,
    target_account_id: AccountId,
    note_type: NoteType,
    mut rng: RpoRandomCoin,
    cards: [[Felt; 4]; 3],
    card_slot: u8,
) -> Result<Note, NoteError> {
    let note_script = include_str!("../../contracts/notes/game/unmask.masm");
    let script_ast = ProgramAst::parse(note_script).unwrap();
    let note_script = client.compile_note_script(script_ast, vec![]).unwrap();

    let mut encrypted_cards = vec![];
    for card in cards.iter() {
        encrypted_cards = [encrypted_cards, vec![card[0], card[1], Felt::ZERO, Felt::ZERO]].concat();
    }
    let inputs = [encrypted_cards, vec![Felt::from(card_slot)]].concat();
    

    let note_inputs = NoteInputs::new(inputs).unwrap();
    let tag: u64 = target_account_id.into();
    let tag: NoteTag = ((tag + tag) as u32).into();
    let serial_num = rng.draw_word();
    let aux = ZERO;

    let metadata = NoteMetadata::new(sender_account_id, NoteType::Public, tag, aux)?;
    let vault = NoteAssets::new(vec![])?;
    let recipient = NoteRecipient::new(serial_num, note_script, note_inputs);

    Ok(Note::new(vault, metadata, recipient))
}

pub fn create_inter_unmask_note<
    R: FeltRng,
    N: NodeRpcClient,
    S: Store,
    A: TransactionAuthenticator,
>(
    client: &mut Client<N, R, S, A>,
    sender_account_id: AccountId,
    target_account_id: AccountId,
    note_type: NoteType,
    mut rng: RpoRandomCoin,
    cards: [[Felt; 4]; 3],
    requester_id: AccountId,
) -> Result<Note, NoteError> {
    let note_script = include_str!("../../contracts/notes/game/inter_unmask.masm");
    let script_ast = ProgramAst::parse(note_script).unwrap();
    let note_script = client.compile_note_script(script_ast, vec![]).unwrap();

    let mut encrypted_cards = vec![];
    for card in cards.iter() {
        encrypted_cards = [encrypted_cards, vec![card[0], card[1], Felt::ZERO, Felt::ZERO]].concat();
    }
    encrypted_cards = [encrypted_cards, vec![Felt::from(requester_id)]].concat();
    

    let note_inputs = NoteInputs::new(encrypted_cards).unwrap();
    let tag: u64 = target_account_id.into();
    let tag: NoteTag = ((tag + tag) as u32).into();
    let serial_num = rng.draw_word();
    let aux = ZERO;

    let metadata = NoteMetadata::new(sender_account_id, NoteType::Public, tag, aux)?;
    let vault = NoteAssets::new(vec![])?;
    let recipient = NoteRecipient::new(serial_num, note_script, note_inputs);

    Ok(Note::new(vault, metadata, recipient))
}

pub fn create_set_hand_note<
    R: FeltRng,
    N: NodeRpcClient,
    S: Store,
    A: TransactionAuthenticator,
>(
    client: &mut Client<N, R, S, A>,
    sender_account_id: AccountId,
    target_account_id: AccountId,
    note_type: NoteType,
    mut rng: RpoRandomCoin,
    cards: [[Felt; 4]; 2],
    player_hand: u8,
) -> Result<Note, NoteError> {
    let note_script = include_str!("../../contracts/notes/game/set_hand.masm");
    let script_ast = ProgramAst::parse(note_script).unwrap();
    let note_script = client.compile_note_script(script_ast, vec![]).unwrap();

    let card1 = SUIT_SIZE * (cards[0][0].as_int() - 1) + cards[0][1].as_int();
    let card2 = SUIT_SIZE * (cards[1][0].as_int() - 1) + cards[1][1].as_int();
    let inputs = vec![Felt::new(card1), Felt::new(card2), Felt::from(player_hand)];

    let note_inputs = NoteInputs::new(inputs).unwrap();
    let tag: u64 = target_account_id.into();
    let tag: NoteTag = ((tag + tag) as u32).into();
    let serial_num = rng.draw_word();
    let aux = ZERO;

    let metadata = NoteMetadata::new(sender_account_id, NoteType::Public, tag, aux)?;
    let vault = NoteAssets::new(vec![])?;
    let recipient = NoteRecipient::new(serial_num, note_script, note_inputs);

    Ok(Note::new(vault, metadata, recipient))
}

pub fn create_send_community_cards_note< 
    R: FeltRng,
    N: NodeRpcClient,
    S: Store,
    A: TransactionAuthenticator,
>(
    client: &mut Client<N, R, S, A>,
    sender_account_id: AccountId,
    target_account_id: AccountId,
    note_type: NoteType,
    mut rng: RpoRandomCoin,
    cards: [[Felt; 4]; 3],
    current_phase: u8,
) -> Result<Note, NoteError> {
    let note_script = include_str!("../../contracts/notes/game/send_community_cards.masm");
    let script_ast = ProgramAst::parse(note_script).unwrap();
    let note_script = client.compile_note_script(script_ast, vec![]).unwrap();

    let mut community_cards = vec![];
    for card in cards.iter() {
        community_cards = [community_cards, vec![card[0], card[1], Felt::ZERO, Felt::ZERO]].concat();
    }
    community_cards = [community_cards, vec![Felt::from(current_phase)]].concat();

    let note_inputs = NoteInputs::new(community_cards).unwrap();
    let tag: u64 = target_account_id.into();
    let tag: NoteTag = ((tag + tag) as u32).into();
    let serial_num = rng.draw_word();
    let aux = ZERO;

    let metadata = NoteMetadata::new(sender_account_id, NoteType::Public, tag, aux)?;
    let vault = NoteAssets::new(vec![])?;
    let recipient = NoteRecipient::new(serial_num, note_script, note_inputs);

    Ok(Note::new(vault, metadata, recipient))
}

// TODO: remove this function after testing
pub async fn mint_note(
    client: &mut AzeClient,
    basic_account_id: AccountId,
    faucet_account_id: AccountId,
    note_type: NoteType,
) -> InputNote {
    let (regular_account, _seed) = client.get_account(basic_account_id).unwrap();

    // Create a Mint Tx for 1000 units of our fungible asset
    let fungible_asset = FungibleAsset::new(faucet_account_id, BUY_IN_AMOUNT).unwrap();
    let tx_template =
        TransactionTemplate::MintFungibleAsset(fungible_asset, basic_account_id, note_type);

    
    let tx_request = client.build_transaction_request(tx_template).unwrap();
    let _ = execute_tx_and_sync(client, tx_request.clone()).await;

    // Check that note is committed and return it
    
    let note_id = tx_request.expected_output_notes()[0].id();
    let note = client.get_input_note(note_id).unwrap();
    note.try_into().unwrap()
}
// TODO: remove it after testing the flow
pub async fn consume_notes(
    client: &mut AzeClient,
    account_id: AccountId,
    input_notes: &[InputNote],
) {
    let tx_template =
        TransactionTemplate::ConsumeNotes(account_id, input_notes.iter().map(|n| n.id()).collect());
    
    let tx_request: TransactionRequest = client.build_transaction_request(tx_template).unwrap();
    execute_tx_and_sync(client, tx_request).await;
}