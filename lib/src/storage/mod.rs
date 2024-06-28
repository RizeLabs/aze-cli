#[derive(Clone)]
pub struct GameStorageSlotData {
    small_blind_amt: u8,
    buy_in_amt: u8,
    player_count: u8,
    current_turn_index: u8,
    highest_bet: u8,
    player_balance: u8,
    player_account_ids: Vec<u64>,
}

impl GameStorageSlotData {
    pub fn new(
        small_blind_amt: u8,
        buy_in_amt: u8,
        player_count: u8,
        current_turn_index: u8,
        highest_bet: u8,
        player_balance: u8,
        player_account_ids: Vec<u64>,
    ) -> Self {
        Self {
            small_blind_amt,
            buy_in_amt,
            player_count,
            current_turn_index,
            highest_bet,
            player_balance,
            player_account_ids,
        }
    }

    pub fn small_blind_amt(&self) -> u8 {
        self.small_blind_amt
    }

    pub fn buy_in_amt(&self) -> u8 {
        self.buy_in_amt
    }

    pub fn player_count(&self) -> u8 {
        self.player_count
    }

    pub fn flop_index(&self) -> u8 {
        self.player_count * 2 + 1
    }

    pub fn current_turn_index(&self) -> u8 {
        self.current_turn_index
    }

    pub fn highest_bet(&self) -> u8 {
        self.highest_bet
    }

    pub fn player_balance(&self) -> u8 {
        self.player_balance
    }

    pub fn player_account_ids(&self) -> Vec<u64> {
        self.player_account_ids.clone()
    }
}
