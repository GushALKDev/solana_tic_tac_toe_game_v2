use anchor_lang::prelude::*;
use num_derive::*;
use num_traits::*;
use anchor_lang::solana_program::system_instruction;
use anchor_lang::solana_program::program::{invoke, invoke_signed};


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
        let system_program = &ctx.accounts.system_program.to_account_info();
        let global_state = &mut ctx.accounts.global_state;
        let game_pda = &mut ctx.accounts.game;
        let game_account_info = &game_pda.to_account_info();
        let player = &mut ctx.accounts.player;

        let (_game_pda_key, bump) = Pubkey::find_program_address(
            &[GAME_SEED, &game_pda.number.to_le_bytes()],
            ctx.program_id,
        );

        // Check if the player is in an active game
        if let Some(game_address) = global_state.find_game_from_player(player.key()).ok() {
            msg!("Game Address: {:?}", game_address);

            // Make the move if it is the player's turn
            return game_pda.play(player, global_state, game_account_info, system_program, &tile, bump);
        } else {
            msg!("ERROR: PlayerHasNotAnActiveGame");
            return Err(ErrorCode::PlayerHasNotAnActiveGame.into());
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
impl<'info> Game {
    // Tamaño máximo de la cuenta de juego.
    pub const MAXIMUM_SIZE: usize = (32 * 2) + 1 + (9 * (1 + 1)) + (32 + 1) + 8;

    // Verifica si el juego está activo.
    pub fn is_active(&self) -> bool {
        self.state == GameState::InProgress
    }

    fn current_signer_index(&self, player: Option<Pubkey>) -> usize {
        match player {
            Some(player_pubkey) => {
                println!("El valor de la clave pública es: {:?}", player_pubkey);
                
                if let Some(index) = self.players.iter().position(|&p| p == player) {
                    return index; // Devuelve el índice encontrado como `usize`
                } else {
                    return 2;
                }
            }
            None => 2,
        }
    }
    
    // Devuelve el índice del jugador cuyo turno es.
    fn current_turn_index(&self) -> usize {
        (self.turn % 2) as usize
    }

    // Devuelve la clave pública del jugador actual.
    pub fn current_player(&self, player: Option<Pubkey>) -> Pubkey {
        self.players[self.current_signer_index(player)]
            .expect("Current player should be set")
    }

    // Realiza un movimiento en el tablero.
    pub fn play(
        &mut self,
        player: &mut Signer<'info>,
        global_state: &mut GlobalState,
        game_account_info: &AccountInfo<'info>,
        system_program: &AccountInfo<'info>,
        tile: &Tile,
        bump: u8, // Añadir el bump como parámetro
    ) -> Result<()> {
        msg!("Current Player address: {:?}", player);
    
        let current_turn_index: usize = self.current_turn_index();
        let current_signer_index: usize = self.current_signer_index(Some(player.key()));
        if current_turn_index != current_signer_index {
            return Err(ErrorCode::NotPlayersTurn.into());
        }
        require!(self.is_active(), ErrorCode::GameAlreadyOver);
    
        // Verifica si la posición en el tablero es válida y está vacía.
        match tile {
            tile @ Tile { row: 0..=2, column: 0..=2 } => match self.board[tile.row as usize][tile.column as usize] {
                Some(_) => return Err(ErrorCode::TileAlreadySet.into()), // Casilla ya ocupada.
                None => {
                    // Asigna el signo del jugador actual a la casilla vacía.
                    self.board[tile.row as usize][tile.column as usize] =
                        Some(Sign::from_usize(self.current_signer_index(Some(player.key()))).unwrap());
                }
            },
            _ => return Err(ErrorCode::TileOutOfBounds.into()), // Fuera de los límites del tablero.
        }
    
        if GameState::InProgress == self.state {
            self.turn += 1;
        }
    
        // Actualiza el estado del juego después del movimiento.
        self.update_state(player, global_state, game_account_info, system_program, bump);
    
        msg!("Player {} - Turn {}", current_signer_index + 1, self.turn);
        msg!("Moved to {:?}", tile);
    
        Ok(())
    }

    // Función para actualizar el estado del juego (si hay un ganador o empate).
    fn update_state(
        &mut self,
        player: &mut Signer<'info>,
        global_state: &mut GlobalState,
        game_account_info: &AccountInfo<'info>,
        system_program: &AccountInfo<'info>,
        bump: u8, // Añadir el bump como parámetro
    ) {
        // Verificar todas las combinaciones de filas y columnas.
        for i in 0..=2 {
            if self.is_winning_trio([(i, 0), (i, 1), (i, 2)]) {
                self.state = GameState::Won {
                    winner: self.current_player(Some(player.key())),
                };
                self.finish_game(player, global_state, game_account_info, system_program, bump).ok();
                msg!("Player {:?} won the game!", player);
                return;
            }
            if self.is_winning_trio([(0, i), (1, i), (2, i)]) {
                self.state = GameState::Won {
                    winner: self.current_player(Some(player.key())),
                };
                self.finish_game(player, global_state, game_account_info, system_program, bump).ok();
                msg!("Player {:?} won the game!", player);
                return;
            }
        }
    
        // Verificar diagonales.
        if self.is_winning_trio([(0, 0), (1, 1), (2, 2)])
            || self.is_winning_trio([(0, 2), (1, 1), (2, 0)])
        {
            self.state = GameState::Won {
                winner: self.current_player(Some(player.key())),
            };
            self.finish_game(player, global_state, game_account_info, system_program, bump).ok();
            msg!("Player {:?} won the game!", player);
            return;
        }
    
        // Si quedan casillas vacías, el juego continúa activo.
        for row in 0..=2 {
            for column in 0..=2 {
                if self.board[row][column].is_none() {
                    return;
                }
            }
        }
        // Si no quedan casillas vacías y nadie ha ganado, el juego termina en empate.
        self.state = GameState::Tie;
        global_state.remove_players_from_game(game_account_info.key());
        msg!("Game ends in a tie!");
    }    

    // Función para cerrar el juego y transferir lamports de la cuenta del juego.
    pub fn finish_game(
        &mut self,
        player: &Signer<'info>,
        global_state: &mut GlobalState,
        game_account_info: &AccountInfo<'info>,
        system_program: &AccountInfo<'info>,
        bump: u8, // Añadir el bump como parámetro
    ) -> Result<()> {
        msg!("Entering finish_game");
        msg!("Using bump: {}", bump);
    
        // Verificar que el jugador es el propietario del juego
        if self.players[0] != Some(player.key()) {
            msg!("ERROR: Player not the owner of the game account.");
            return Err(ErrorCode::PlayerNotOpenedTheGameAccount.into());
        }
    
        // Determinar el ganador y emitir evento de finalización de juego
        let winner_option = match self.state {
            GameState::Won { winner } => Some(winner),
            _ => None,
        };
    
        let winner = match winner_option {
            Some(pk) => pk,
            None => {
                msg!("ERROR: Winner not found.");
                return Err(ErrorCode::WinnerNotFound.into());
            }
        };
    
        emit!(GameFinished {
            player_one: self.players[0].unwrap(),
            player_two: self.players[1].unwrap(),
            winner,
        });
    
        msg!("Game Account preparation for manual close completed.");
    
        // Remover al jugador del mapping en `global_state`
        global_state.remove_players_from_game(game_account_info.key());
    
        // Transferir lamports de `game` a `player` usando el bump como firma
        let seeds: &[&[u8]] = &[GAME_SEED, &self.number.to_le_bytes(), &[bump]];
        msg!("Seeds for invoke_signed: {:?}", seeds);
    
        let lamports: u64 = **game_account_info.lamports.borrow();
        if let Err(err) = invoke_signed(
            &system_instruction::transfer(
                game_account_info.key,
                player.key,
                lamports,
            ),
            &[
                game_account_info.clone(),
                player.to_account_info().clone(),
                system_program.clone(),
            ],
            &[seeds],
        ) {
            msg!("ERROR during invoke_signed: {:?}", err);
            return Err(err.into());
        }
    
        // Reducir el saldo de lamports a cero y desactivar la cuenta
        **game_account_info.try_borrow_mut_lamports()? = 0;
    
        // Limpiar el contenido de la cuenta `Game` (opcional)
        *self = Game {
            number: 0,
            players: [None, None],
            turn: 0,
            board: [[None; 3]; 3],
            state: GameState::Uninitialized,
        };
    
        msg!("Game Account manually closed.");
        
        Ok(())
    }
    

    // Función para verificar si tres casillas forman una línea ganadora.
    fn is_winning_trio(&self, trio: [(usize, usize); 3]) -> bool {
        let [first, second, third] = trio;
        self.board[first.0][first.1].is_some()
            && self.board[first.0][first.1] == self.board[second.0][second.1]
            && self.board[first.0][first.1] == self.board[third.0][third.1]
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

// Account setup for `play` instruction.
#[derive(Accounts)]
pub struct Play<'info> {
    #[account(mut)]
    pub global_state: Account<'info, GlobalState>, // Estado global que contiene el contador de juegos.
    #[account(mut, seeds = [GAME_SEED, &game.number.to_le_bytes()], bump, signer)] // Configuración correcta de la semilla.
    pub game: Account<'info, Game>, // Nueva cuenta PDA para el juego.
    #[account(mut)]
    pub player: Signer<'info>,      // Jugador que hace el movimiento.
    pub system_program: Program<'info, System>, // Programa del sistema.
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
    #[msg("Player not opened the game account.")]
    PlayerNotOpenedTheGameAccount,
    #[msg("Player not found in mapping.")]
    WinnerNotFound,
    #[msg("Game not found in mapping.")]
    GameNotFound,
    #[msg("Account not found.")]
    AccountNotFound,
    #[msg("Game already in progress.")]
    GameAlreadyInProgress,
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