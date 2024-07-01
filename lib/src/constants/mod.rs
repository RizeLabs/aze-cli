pub const DEFAULT_AUTH_SCRIPT: &str = "
    use.miden::contracts::auth::basic->auth_tx

    begin
        call.auth_tx::auth_tx_rpo_falcon512
    end
";

pub const AUTH_SEND_NOTE_SCRIPT: &str = include_str!("../../asm/tx_scripts/auth_send_note.masm");
pub const CLIENT_CONFIG_FILE_NAME: &str = "miden-client.toml";
pub const BUY_IN_AMOUNT: u64 = 1000;
pub const SMALL_BUY_IN_AMOUNT: u8 = 1;
pub const TRANSFER_AMOUNT: u64 = 59;
pub const SMALL_BLIND_AMOUNT: u8 = 5;
pub const PLAYER_INITIAL_BALANCE: u8 = 30;
pub const HIGHEST_BET: u8 = SMALL_BLIND_AMOUNT;
pub const NO_OF_PLAYERS: u8 = 4;
pub const FLOP_NO_OF_CARDS: u8 = 3;
pub const FLOP_INDEX: u8 = NO_OF_PLAYERS * 2 + 1;
pub const PLAYER_BET_OFFSET: u8 = 3;
pub const IS_FOLD_OFFSET: u8 = 10;
pub const HAND_OFFSET: u8 = 11;
pub const PLAYER_STATS_SLOTS: u8 = 13;
pub const SMALL_BLIND_SLOT: u8 = 54;
pub const FIRST_PLAYER_INDEX: u8 = 64;
pub const LAST_PLAYER_INDEX: u8 = FIRST_PLAYER_INDEX + (NO_OF_PLAYERS - 1) * PLAYER_STATS_SLOTS;
pub const RAISER_INDEX_SLOT: u8 = 58;
pub const CURRENT_TURN_INDEX_SLOT: u8 = 60;
pub const HIGHEST_BET_SLOT: u8 = 61;
pub const CURRENT_PHASE_SLOT: u8 = 62;
pub const CHECK_COUNTER_SLOT: u8 = 63;
pub const PLAYER_BALANCE_SLOT: u8 = 68;
pub const PLAYER_HANDS: u8 = 75;
pub const POT_VALUE: u8 = 73;
pub const COMMUNITY_CARDS: [u8;5] = [116, 117, 118, 119, 120];
pub const FLOP_SLOT: u8 = 116;

// PLAYER ACCOUNT
pub const PLAYER_FILE_PATH: &str = "Player.toml";
pub const SECRET_KEY_SLOT: u8 = 53;
pub const DEFAULT_SKEY: u8 = 8;
pub const MASKING_FACTOR_SLOT: u8 = 55;
pub const DEFAULT_MASKING_FACTOR: u8 = 9;
pub const PLAYER_DATA_SLOT: u8 = 56;
pub const PHASE_DATA_SLOT: u8 = 57;
pub const DEFAULT_ACTION_TYPE: u64 = 1;
pub const PLAYER_CARD1_SLOT: u8 = 100;
pub const PLAYER_CARD2_SLOT: u8 = 101;
pub const REQUESTER_SLOT: u8 = 102;
pub const TEMP_CARD_SLOT: u8 = 103;