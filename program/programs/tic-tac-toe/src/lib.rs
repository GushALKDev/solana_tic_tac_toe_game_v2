use anchor_lang::prelude::*;
use num_derive::*;
use num_traits::*;

// Declare the program ID.
declare_id!("CWKWcWzgQq6YJKnMvzMahwM7TYqHKXfciBbUkdevU6R7");

const GLOBAL_STATE_SEED: &[u8] = b"global_state";
const GAME_SEED: &[u8] = b"game";

#[program]
pub mod tic_tac_toe {

    use super::*;

    // Method to initialize the global state
    pub fn initialize_global_state(ctx: Context<InitializeGlobalState>) -> Result<()> {
        let global_state = &mut ctx.accounts.global_state;
        global_state.game_count = 1; // Initializes the game counter to 1
        global_state.players_mapping = Vec::new(); // Initialize player mapping vector
        global_state.games_mapping = Vec::new(); // Initialize game mapping vector
        msg!("Global state Counter: {}", global_state.game_count);
        msg!("players_keys: {:?}", global_state.players_mapping);
        msg!("players_values: {:?}", global_state.games_mapping);
        Ok(())
    }

    // Sets up the game and derives a unique PDA account using a global game counter.
    pub fn setup_game<'info>(ctx: Context<'_, '_, 'info, 'info, SetupGame<'info>>) -> Result<()> {
        // Step 1: Extract player key and the remaining accounts
        let player_key = ctx.accounts.player.key();
        let game = &mut ctx.accounts.game;
    
        // Step 2: Extract global_state mutable access AFTER we finish using remaining_accounts
        let global_state = &mut ctx.accounts.global_state;
    
        // If player already has an active game, return error
        if global_state.find_game_from_player(player_key).is_ok() {
            msg!("ERROR: GameAlreadyInProgress");
            return Err(ErrorCode::GameAlreadyInProgress.into());
        }
        
        msg!("Game State {:?}", game.state);

        // Check if there is a waiting game without accessing `global_state` mutably
        if game.state == GameState::Waiting {
            msg!("There is a game WAITING.");

            game.players[1] = Some(player_key);
            game.state = GameState::InProgress;
            
            // Update global state
            global_state.game_count += 1;
            global_state.add_player(player_key, game.key())?;
            
            msg!("Game PDA PubKey: {}", game.key());
            msg!("Player 1: {:?}", game.players[0]);
            msg!("Player 2: {:?}", game.players[1]);
            msg!("Game State: {:?}", game.state);
    
            return Ok(());
        }
        else if game.state == GameState::Uninitialized {
            // Step 3: If no waiting game, set up a new one with mutable access to `global_state`
            msg!("There is NOT a game WAITING.");
            msg!("Setting up a new one.");
            
            game.number = global_state.game_count;
            game.players[0] = Some(player_key);
            game.players[1] = None;
            game.turn = 0;
            game.board = [[None; 3]; 3];
            game.state = GameState::Waiting;
        
            global_state.add_player(player_key, game.key())?;
            
            msg!("Game PDA PubKey: {}", game.key());
            msg!("Game Number: {:?}", game.number);
            msg!("Player 1: {:?}", game.players[0]);
            msg!("Player 2: {:?}", game.players[1]);
            msg!("Game State: {:?}", game.state);

            return Ok(())
        }
        else {
            return Err(ErrorCode::NoUninitializedOrWaitingGame.into());
        }
    }

    // Function to make a move in the game.
    pub fn play(ctx: Context<Play>, tile: Tile) -> Result<()> {
        let global_state = &mut ctx.accounts.global_state;
        let game_pda = &mut ctx.accounts.game;
        let player_key = ctx.accounts.player.key();

        // Check if the player is in an active game
        if let Some(game_address) = global_state.find_game_from_player(player_key).ok() {
            msg!("Game Address: {:?}", game_address);
            // Check if the current game is NOT over
            require!(!game_pda.is_over(), ErrorCode::GameAlreadyOver);
            // Check if the current game is in progress
            require!(game_pda.is_in_progress(), ErrorCode::GameNotInProgress);
            // Make the move if it is the player's turn
            let game_pda_key = game_pda.key().clone();
            return game_pda.play(Some(player_key), global_state, game_pda_key, &tile);
        }
        else {
            return Err(ErrorCode::PlayerHasNotAnActiveGame.into());
        }
    }

    pub fn cancel_game(ctx: Context<CancelGame>) -> Result<()> {
        let global_state = &mut ctx.accounts.global_state;
        let game_pda = &mut ctx.accounts.game;
        let game_pda_key = game_pda.key().clone();

        // Game must be InProgress or Waiting
        if game_pda.state != GameState::InProgress && game_pda.state != GameState::Waiting {
            return Err(ErrorCode::GameNotInProgress.into());
        }
        // Game Waiting
        if game_pda.state == GameState::Waiting {
            game_pda.state = GameState::Canceled;
        }
        // Game InProgress
        else {
            // Check if the signer is a player
            let player_one = match game_pda.players[0] {
                Some(pk) => pk,
                None => return Err(ErrorCode::PlayerNotFound.into()),
            };
            let player_two = match game_pda.players[1] {
                Some(pk) => pk,
                None => return Err(ErrorCode::PlayerNotFound.into()),
            };
            if ctx.accounts.signer.key() == player_one || ctx.accounts.signer.key() == player_two {
                // If player one cancels, player two wins
                if ctx.accounts.signer.key() == player_one {
                    game_pda.state = GameState::Won {
                        winner: player_two,
                    };
                }
                // If player two cancels, player one wins
                else {
                    game_pda.state = GameState::Won {
                        winner: player_one,
                    };
                }
            }
            else {
                return Err(ErrorCode::SignerIsNotPlayer.into());
            }
        }
        game_pda.end_game(global_state, game_pda_key);
        Ok(())
    }

    pub fn close_game_account(ctx: Context<CloseGameAccount>) -> Result<()> {
        // Game state must be canceled or finished
        if !matches!(ctx.accounts.game.state, GameState::Canceled | GameState::Tie | GameState::Won { .. }) {
            return Err(ErrorCode::GameAlreadyInProgress.into());
        }
        // Check if the signer is the player 1
        let player_one = match ctx.accounts.game.players[0] {
            Some(pk) => pk,
            None => return Err(ErrorCode::PlayerNotFound.into()),
        };
        if ctx.accounts.signer.key() == player_one {
            Ok(())
        }
        else {
            return Err(ErrorCode::SignerDidNotOpenTheGameAccount.into());
        }
    }
}

impl GlobalState {
    // Adds a player and game PDA to the mapping
    pub fn add_player(&mut self, player: Pubkey, game_pda: Pubkey) -> Result<()> {
        self.players_mapping.push(player);
        self.games_mapping.push(game_pda);
        Ok(())
    }

    // Removes all instances of a player and its associated game PDA from the mapping
    pub fn remove_players_from_game(&mut self, game: Pubkey) {
        // Vector to store indices to be removed
        let indices_to_remove: Vec<usize> = self.games_mapping
            .iter()
            .enumerate()
            .filter_map(|(index, &p)| if p == game { Some(index) } else { None })
            .collect();

        // If no indices were found, return an error
        if indices_to_remove.is_empty() {
            panic!("Game not found in the mapping");
        }

        // Remove elements in reverse order to avoid index mismatch issues
        for &index in indices_to_remove.iter().rev() {
            msg!("Removed player {} from game {}", self.players_mapping[index], self.games_mapping[index]);
            self.players_mapping.remove(index);
            self.games_mapping.remove(index);
        }
    }


    // Finds and returns the game PDA associated with a player
    pub fn find_game_from_player(&self, player: Pubkey) -> Result<Pubkey> {
        if let Some(index) = self.players_mapping.iter().position(|&p| p == player) {
            Ok(self.games_mapping[index])
        } else {
            Err(ErrorCode::PlayerNotFound.into())
        }
    }
}

// Implementation of the game structure.
impl Game {
    // Maximum size of the game account.
    pub const MAXIMUM_SIZE: usize = 8 + (32 * 2) + 1 + (9 * (1 + 1)) + (32 + 1) + 8;

    // Checks if the game is still active.
    pub fn is_in_progress(&self) -> bool {
        self.state == GameState::InProgress
    }

    pub fn is_over(&self) -> bool {
        self.state == GameState::Tie || matches!(self.state, GameState::Won { winner: _ })
    }

    pub fn is_waiting(&self) -> bool {
        self.state == GameState::Waiting
    }

    fn current_signer_index(&self, player: Option<Pubkey>) -> usize {
        match player {
            Some(player_pubkey) => {
                // Aquí `player_pubkey` es de tipo `Pubkey`.
                println!("El valor de la clave pública es: {:?}", player_pubkey);
                
                if let Some(index) = self.players.iter().position(|&p| p == player) {
                    return index // Devolvemos el índice encontrado como `usize`
                }
                
                else {
                    return 2;
                }
            }
            None => {
                return 2;
            }
        }
    }
    
    // Returns the index of the player whose turn it is.
    fn current_turn_index(&self) -> usize {
        ((self.turn) % 2) as usize
    }

    // Returns the public key of the current player.
    pub fn current_player(&self, player: Option<Pubkey>) -> Pubkey {
        self.players[self.current_signer_index(player)]
            .expect("Current player should be set")
    }

    // Makes a move on the board.
    pub fn play(&mut self, player:Option<Pubkey>, global_state: &mut GlobalState, game_pubkey:Pubkey, tile: &Tile) -> Result<()> {
        msg!("Current Player address: {:?}", player);
        // msg!("Current Signer index: {:?}", self.current_signer_index(player));
        // msg!("Current Turn index: {:?}", self.current_turn_index());

        let current_turn_index:usize = self.current_turn_index();
        let current_signer_index:usize = self.current_signer_index(player);
        if current_turn_index != current_signer_index {
            return Err(ErrorCode::NotPlayersTurn.into());
        }

        // Check if the board position is valid and empty.
        match tile {
            tile @ Tile { row: 0..=2, column: 0..=2 } => match self.board[tile.row as usize][tile.column as usize] {
                Some(_) => return Err(ErrorCode::TileAlreadySet.into()), // Tile already occupied.
                None => {
                    // Assign the current player's sign to the empty tile.
                    self.board[tile.row as usize][tile.column as usize] =
                        Some(Sign::from_usize(self.current_signer_index(player)).unwrap());
                }
            },
            _ => return Err(ErrorCode::TileOutOfBounds.into()), // Out of board bounds.
        }

        if GameState::InProgress == self.state {
            self.turn += 1;
        }

        // Update the game state after the move.
        self.update_state(player, global_state, game_pubkey);

        msg!("Player {} - Turn {}", current_signer_index + 1, self.turn);
        msg!("Moved to {:?}", tile);

        Ok(())
    }

    // Function to check if three tiles form a winning line.
    fn is_winning_trio(&self, trio: [(usize, usize); 3]) -> bool {
        let [first, second, third] = trio;
        self.board[first.0][first.1].is_some()
            && self.board[first.0][first.1] == self.board[second.0][second.1]
            && self.board[first.0][first.1] == self.board[third.0][third.1]
    }

    // Function to update the game state (if there's a winner or tie).
    fn update_state(&mut self, player:Option<Pubkey>, global_state: &mut GlobalState, game_pubkey:Pubkey) {
        // Check all row and column combinations.
        for i in 0..=2 {
            if self.is_winning_trio([(i, 0), (i, 1), (i, 2)]) {
                self.state = GameState::Won {
                    winner: self.current_player(player),
                };
                self.end_game(global_state, game_pubkey);
                msg!("Player {:?} won the game!", player);
                return;
            }
            if self.is_winning_trio([(0, i), (1, i), (2, i)]) {
                self.state = GameState::Won {
                    winner: self.current_player(player),
                };
                self.end_game(global_state, game_pubkey);
                msg!("Player {:?} won the game!", player);
                return;
            }
        }

        // Check diagonals.
        if self.is_winning_trio([(0, 0), (1, 1), (2, 2)])
            || self.is_winning_trio([(0, 2), (1, 1), (2, 0)])
        {
            self.state = GameState::Won {
                winner: self.current_player(player),
            };
            self.end_game(global_state, game_pubkey);
            msg!("Player {:?} won the game!", player);
            return;
        }

        // If empty tiles remain, the game remains active.
        for row in 0..=2 {
            for column in 0..=2 {
                if self.board[row][column].is_none() {
                    return;
                }
            }
        }
        // If no empty tiles remain and no one has won, the game ends in a tie.
        self.state = GameState::Tie;
        // remove_players();
        self.end_game(global_state, game_pubkey);
        msg!("Game ends in a tie!");
    }

    fn end_game(&mut self, global_state: &mut GlobalState, game_pubkey:Pubkey) {
        // Remove players from global mapping
        global_state.remove_players_from_game(game_pubkey);
        
        // Emit an event to log the game details before closing the account
        let player_one = match self.players[0] {
            Some(pk) => pk,
            None => return,
        };
        let player_two = match self.players[1] {
            Some(pk) => pk,
            None => return,
        };
        let winner_option = match self.state {
            GameState::Won { winner } => Some(winner),
            _ => None,
        };
        let winner = match winner_option {
            Some(pk) => pk,
            None => return,
        };
        emit!(GameFinished {
            player_one: player_one,
            player_two: player_two,
            winner: winner,
        });
    }
}

// Structure representing the global state.
#[account]
pub struct GlobalState {
    pub game_count: u64, // Global game counter to ensure unique accounts.
    pub players_mapping: Vec<Pubkey>, // Vector of player public keys
    pub games_mapping: Vec<Pubkey>, // Vector of game PDAs
}

// Structure representing each game's state.
#[account]
pub struct Game {
    number: u64,
    players: [Option<Pubkey>; 2], // Public keys of the players (64 bytes).
    turn: u8,             // Current turn number (1 byte).
    board: [[Option<Sign>; 3]; 3], // Board state (9 positions with 2 bytes per cell).
    state: GameState,               // Current game state (won, tie, active).
}

// Enum for possible game states.
#[derive(AnchorSerialize, AnchorDeserialize, Clone, PartialEq, Eq, Debug)]
pub enum GameState {
    Uninitialized,          // The game is not initialized yet
    Waiting,                // The game is waiting for player 2.
    InProgress,             // The game is in progress.
    Tie,                    // The game ended in a tie.
    Won { winner: Pubkey }, // Someone has won the game.
    Canceled,               // Canceled
}

// Enum representing player signs (X or O).
#[derive(AnchorSerialize, AnchorDeserialize, FromPrimitive, ToPrimitive, Copy, Clone, PartialEq, Eq)]
pub enum Sign {
    X,
    O,
}

// Structure representing a tile on the board.
#[derive(AnchorSerialize, AnchorDeserialize, Debug)]
pub struct Tile {
    row: u8,    // Tile row (0-2).
    column: u8, // Tile column (0-2).
}

// Account setup for `initialize_global_state` instruction.
#[derive(Accounts)]
pub struct InitializeGlobalState<'info> {
    #[account(init, payer = payer, space = 8 + 32 + 4 + 1024 + 1, seeds = [GLOBAL_STATE_SEED], bump)]
    pub global_state: Account<'info, GlobalState>,
    #[account(mut)]
    pub payer: Signer<'info>,
    pub system_program: Program<'info, System>,
}

// Account setup for `setup_game` instruction.
#[derive(Accounts)]
pub struct SetupGame<'info> {
    #[account(mut)]
    pub global_state: Account<'info, GlobalState>, // Global state containing the game counter.
    #[account(init_if_needed, payer = player, space = Game::MAXIMUM_SIZE, seeds = [GAME_SEED, &global_state.game_count.to_le_bytes()], bump)]
    pub game: Account<'info, Game>, // New PDA account for the game.
    #[account(mut)]
    pub player: Signer<'info>,  // Player.
    pub system_program: Program<'info, System>, // Use of the system program.
}

#[derive(Accounts)]
pub struct CancelGame<'info> {
    #[account(mut)]
    pub global_state: Account<'info, GlobalState>, // Global state containing the game counter.
    #[account(mut)]
    pub game: Account<'info, Game>,
    #[account(mut)]
    pub signer: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct CloseGameAccount<'info> {
    #[account(mut, close = signer)]
    pub game: Account<'info, Game>,
    #[account(mut)]
    pub signer: Signer<'info>,
    pub system_program: Program<'info, System>,
}

// Account setup for `play` instruction.
#[derive(Accounts)]
pub struct Play<'info> {
    #[account(mut)]
    pub global_state: Account<'info, GlobalState>, // Global state containing the game counter.
    #[account(mut)]
    pub game: Account<'info, Game>, // New PDA account for the game.
    #[account(mut)]
    pub player: Signer<'info>,      // Player making the move.
}

#[event]
pub struct GameFinished {
    pub player_one: Pubkey,
    pub player_two: Pubkey,
    pub winner: Pubkey,
}

// Definition of possible errors.
#[error_code]
pub enum ErrorCode {
    #[msg("Player not found in mapping.")]
    PlayerNotFound,
    #[msg("Player did not open the game account.")]
    SignerDidNotOpenTheGameAccount,
    #[msg("The signer is not a player.")]
    SignerIsNotPlayer,
    #[msg("Player not found in mapping.")]
    WinnerNotFound,
    #[msg("Game not found in mapping.")]
    GameNotFound,
    #[msg("Account not found.")]
    AccountNotFound,
    #[msg("Game already in progress.")]
    GameAlreadyInProgress,
    #[msg("Game not in progress.")]
    GameNotInProgress,
    #[msg("Attempt to play in a game that has already ended..")]
    GameAlreadyOver,
    #[msg("Not the current player's turn.")]
    NotPlayersTurn,
    #[msg("Attempt to play on an occupied tile.")]
    TileAlreadySet,
    #[msg("Attempt to play outside the board limits.")]
    TileOutOfBounds,
    #[msg("No uinitilized or waiting game.")]
    NoUninitializedOrWaitingGame,
    #[msg("Player has not an active game.")]
    PlayerHasNotAnActiveGame,
}