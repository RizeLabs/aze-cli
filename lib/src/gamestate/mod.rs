use crate::{ 
    client::{ create_aze_client, AzeClient }, 
    constants::CURRENT_TURN_INDEX_SLOT 
};
use aze_types::actions::ActionType;
use miden_objects::accounts::AccountId;
use serde::{ Deserialize, Serialize };

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
pub struct Check_Action {
    pub action_type: ActionType,
    pub amount: Option<u64>, // Only used for Raise, others will be None
}

#[derive(Debug, Clone)]
struct Player {
    id: u64,
    balance: u64,
    current_bet: u64,
    has_folded: bool,
}

#[derive(Debug, Clone)]
pub struct PokerGame {
    players: Vec<Player>,
    small_blind: u64,
    big_blind: u64,
    pot: u64,
    current_bet: u64,
    current_player_index: usize,
}

impl PokerGame {
    pub fn new(
        player_ids: Vec<u64>,
        initial_balances: Vec<u64>,
        small_blind: u64,
        big_blind: u64
    ) -> Self {
        let players = player_ids
            .into_iter()
            .zip(initial_balances.into_iter())
            .map(|(id, balance)| Player {
                id,
                balance,
                current_bet: 0,
                has_folded: false,
            })
            .collect();

        PokerGame {
            players,
            small_blind,
            big_blind,
            pot: 0,
            current_bet: 0,
            current_player_index: 0,
        }
    }

    pub fn check_move(&mut self, check_action: Check_Action, player_id: u64, game_id: u64) -> bool {
        let player = &mut self.players[self.current_player_index];
        // get current turn from slots
        let mut client: AzeClient = create_aze_client();
        let game_account_id = AccountId::try_from(game_id).unwrap();
        let game_account = client.get_account(game_account_id).unwrap().0;

        let current_turn_player_id = game_account
            .storage()
            .get_item(CURRENT_TURN_INDEX_SLOT)
            .as_elements()[0]
            .as_int();
        let current_player = game_account
            .storage()
            .get_item(current_turn_player_id as u8)
            .as_elements()[0]
            .as_int();

        if player_id != current_player {
            eprintln!("Not your turn");
            return false;
        }
        if player.has_folded {
            eprintln!("Player has already folded");
            return false;
        }

        match check_action.action_type {
            ActionType::Fold => {
                player.has_folded = true;
            }
            ActionType::Check => {
                if player.current_bet < self.current_bet {
                    eprintln!("Cannot check, must call or raise");
                    return false;
                }
            }
            ActionType::Call => {
                let call_amount = self.current_bet - player.current_bet;
                if player.balance < call_amount {
                    eprintln!("Not enough balance to call");
                    return false;
                }
                player.balance -= call_amount;
                player.current_bet += call_amount;
                self.pot += call_amount;
            }
            ActionType::Raise => {
                if let Some(amount) = check_action.amount {
                    let total_bet = self.current_bet + amount;
                    if player.balance < total_bet {
                        eprintln!("Not enough balance to raise");
                        return false;
                    }
                    player.balance -= total_bet - player.current_bet;
                    player.current_bet = total_bet;
                    self.pot += total_bet - self.current_bet;
                    self.current_bet = total_bet;
                } else {
                    eprintln!("Raise amount not specified");
                    return false;
                }
            }
            ActionType::SmallBlind => {
                if self.current_player_index != 0 {
                    eprintln!("Only P1 can post the small blind");
                    return false;
                }
                let small_blind_amount = self.small_blind;
                if player.balance < small_blind_amount {
                    eprintln!("Not enough balance to post the small blind");
                    return false;
                }
                player.balance -= small_blind_amount;
                player.current_bet = small_blind_amount;
                self.pot += small_blind_amount;
                self.current_bet = small_blind_amount;
            }
            ActionType::BigBlind => {
                if self.current_player_index != 1 {
                    eprintln!("Only P2 can post the big blind");
                    return false;
                }
                let big_blind_amount = self.big_blind;
                if player.balance < big_blind_amount {
                    eprintln!("Not enough balance to post the big blind");
                    return false;
                }
                player.balance -= big_blind_amount;
                player.current_bet = big_blind_amount;
                self.pot += big_blind_amount;
                self.current_bet = big_blind_amount;
            }
        }

        self.current_player_index = (self.current_player_index + 1) % self.players.len();
        while self.players[self.current_player_index].has_folded {
            self.current_player_index = (self.current_player_index + 1) % self.players.len();
        }

        true
    }
}