use crate::accounts::{ commit_hand };
use aze_lib::constants::{ PLAYER_CARD1_SLOT, PLAYER_CARD2_SLOT };
use clap::Parser;
use miden_objects::{ 
    accounts::AccountId,
    Felt, FieldElement
};
use dialoguer::{Input, Select};

#[derive(Debug, Clone, Parser)]
pub struct CommitHandCmd {
    #[arg(short, long, default_value_t = 0)]
    player_id: u64,

    #[arg(short, long, default_value_t = 0)]
    game_id: u64,
}

impl CommitHandCmd {
    pub async fn execute(&self) -> Result<(), String> {
        let sender_account_id = AccountId::try_from(self.player_id).unwrap();
        let game_account_id = AccountId::try_from(self.game_id).unwrap();

        let player_hand = Select::new()
            .with_prompt("What is your hand?")
            .item("Royal Flush")
            .item("Straight Flush")
            .item("Four of a Kind")
            .item("Full House")
            .item("Flush")
            .item("Straight")
            .item("Three of a Kind")
            .item("Two Pair")
            .item("One Pair")
            .item("High Card")
            .interact()
            .expect("Failed to get action type");
        
        commit_hand(sender_account_id, game_account_id, player_hand as u8 + 1).await;

        Ok(())
    }
}