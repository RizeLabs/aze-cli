# aze-cli

## Commands
- ### aze-cli init
  Initializes a new game.
  
  **Arguments:**
    - `game_type`: Only `Holdem` is supported for now.
    - `player`: Array containing account ids of the players for current game.
    - `small_blind`: Small blind amount for the current game.
    - `buy_in`: Buy in amount for the current game.
    - `config`: An optional `Config.toml` file containing all the above data.

  **Example usage:**
    - Without a `Config.toml`
      ```sh
      aze-cli init -g Holem -p id1 id2 id3 id4 -s 5 -b 1000
    - With a `Config.toml`
      ```sh
      aze-cli init -c ./Config.toml

- ### aze-cli register
  Creates a player account.

  **Arguments:**
    - `identifier`: A string identifier for mapping to its corresponding `AccountId`

  **Example usage:**
  ```sh
  aze-cli register -i John

- ### aze-cli connect
  Connect to the game broadcast server.

  **Arguments:**
    - `ws url`: Broadcast server url, generated when init command is run

  **Example usage:**
  ```sh
  aze-cli connect -u ws://192.168.30.51:12044/ws/0x0ad5hs7gduiu3e
 
- ### aze-cli action
  Performs the player's desired bet action.

  **Example usage:**
  ```sh
  aze-cli action

- ### aze-cli consume-notes
  Starts a cron job in player's current device for automatically consuming game notes.

  **Example usage**
  ```sh
  aze-cli receive-cards

- ### aze-cli peek-hand
  Unmasks the player's cards.

  **Example usage:**
  ```sh
  aze-cli peek-hand

- ### aze-cli commit-hand
  Commits the player's current hand to the game account.

  **Example usage:**
  ```sh
  aze-cli commit-hand
