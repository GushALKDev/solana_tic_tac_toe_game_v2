# Solana Tic-Tac-Toe V2

A decentralized Tic-Tac-Toe game built on the Solana blockchain using Anchor. This project allows two players to compete in a classic game of Tic-Tac-Toe with verifiable moves and game state updates, all managed on-chain.

## Features

- **Decentralized**: The game is fully managed on-chain, with each move and game state stored on the Solana blockchain.
- **Secure Game State**: Each game has a unique account for storing the board, players, and game status, ensuring the integrity of every match.
- **Player Turns**: The program enforces player turns, preventing moves out of sequence.
- **Win and Tie Detection**: Automatically detects when a player has won or if the game ends in a tie.
- **Custom Error Handling**: Provides error messages for common gameplay issues like invalid moves or out-of-turn actions.
- **Multiple Games Support**: A global state allows tracking and managing multiple games simultaneously, each with a unique identifier.

## Differences from V1

- **Enhanced PDA Management**: Improved use of Program Derived Addresses (PDA) for better security and separation of game sessions.
- **Global State Mapping**: The GlobalState account now includes mappings for player-to-game relationships, preventing players from having multiple active games.
- **Manual Account Closure**: V2 introduces a `close_game_account()` function to reclaim lamports, whereas V1 did not manage account closures.
- **Better Error Handling**: More comprehensive error messages for clearer debugging and player guidance.
- **Optimized Game Logic**: Refined checks to ensure valid moves and proper enforcement of game rules.
- **Game Fee**: Introduce a fee system for each game that contributes to a pool or is used for funding game development and maintenance.
- **Wager System**: Both players deposit a predefined amount of SOL or tokens before starting a game. The winner would take the pot minus a small game fee, and in the case of a tie, the amount would be split evenly between both players (minus the fee).

## Game Setup

The game is designed to work with two players who both have public keys on the Solana blockchain. A unique Program Derived Address (PDA) is created for each game, ensuring each game session is separate.

## How to Play

1. **Initialize Global State**: This is only done once to set up the global counter for tracking games.

   ```typescript
   // Example code to initialize global state
   await program.methods
     .initializeGlobalState()
     .accounts({
       globalState: globalStateAddress,
       payer: playerOne.publicKey,
       systemProgram: anchor.web3.SystemProgram.programId,
     })
     .rpc();
   ```

2. **Setup a New Game (Player 1)**: Player 1 can initiate a game. This step prepares the game board and assigns the player.

   ```typescript
   // Example code to set up a new game for Player 1
   await program.methods
     .setupGame()
     .accounts({
       globalState: globalStateAddress,
       game: gameAddress,
       player: playerOne.publicKey,
       systemProgram: anchor.web3.SystemProgram.programId,
     })
     .rpc();
   ```

3. **Setup a New Game (Player 2)**: Player 2 then joins the game. This step assigns Player 2 to the game.

   ```typescript
   // Example code to set up a new game for Player 2
   await program.methods
     .setupGame()
     .accounts({
       globalState: globalStateAddress,
       game: gameAddress,
       player: playerTwo.publicKey,
       systemProgram: anchor.web3.SystemProgram.programId,
     })
     .rpc();
   ```

## Play the Game

Players take turns making moves by specifying the row and column coordinates for their marker.

**Player 1 Move**:

```typescript
// Example code for Player 1 to make a move
await program.methods
  .play({ row: x, column: y }) // Replace x and y with the desired tile coordinates
  .accounts({
    globalState: globalStateAddress,
    game: gameAddress,
    player: playerOne.publicKey,
  })
  .rpc();
```

**Player 2 Move**:

```typescript
// Example code for Player 2 to make a move
await program.methods
  .play({ row: x, column: y }) // Replace x and y with the desired tile coordinates
  .accounts({
    globalState: globalStateAddress,
    game: gameAddress,
    player: playerTwo.publicKey,
  })
  .rpc();
```

### Game Rules:
- Player 1 begins with "X" and Player 2 follows with "O".
- Each move updates the game state on-chain.
- The game detects if a move completes a row, column, or diagonal, resulting in a win for the current player.
- If all tiles are filled without a win, the game ends in a tie.

## Cancel a Game

A player can cancel a game under certain conditions.

```typescript
// Example code to cancel a game
await program.methods
  .cancelGame()
  .accounts({
    globalState: globalStateAddress,
    game: gameAddress,
    signer: playerOne.publicKey,
  })
  .rpc();
```

## Close Game Account

Once a game is canceled or finished, Player 1 can close the account to reclaim lamports.

```typescript
// Example code to close the game account
await program.methods
  .closeGameAccount()
  .accounts({
    game: gameAddress,
    signer: playerOne.publicKey,
    systemProgram: anchor.web3.SystemProgram.programId,
  })
  .rpc();
```

## Program Structure

- **GlobalState**: Tracks the total count of games played to ensure unique game accounts and manages player-to-game mappings.
- **Game**: Stores the state of an individual game, including the board, players, and current turn.
- **GameState Enum**: Manages all possible game outcomes:
  - **Uninitialized**: The game has not been initialized yet.
  - **Waiting**: The game is waiting for Player 2 to join.
  - **InProgress**: The game is actively being played.
  - **Tie**: The game ended in a tie with no winner.
  - **Won**: A player has won the game. The state includes the winner's public key.
  - **Canceled**: The game was canceled before it could be completed.
- **Tile Struct**: Defines the row and column for each move on the board.

## Error Handling

The program provides specific error messages for common gameplay issues:

- **GameAlreadyOver**: Attempt to play in a game that has already ended.
- **NotPlayersTurn**: Player attempted to play out of turn.
- **TileAlreadySet**: Trying to make a move on an already occupied tile.
- **TileOutOfBounds**: Attempted move is outside the board limits.
- **GameAlreadyInProgress**: Attempt to create or join a game when one is already in progress for the player.
- **PlayerNotFound**: A player was not found in the player-to-game mapping.
- **GameNotFound**: The specified game was not found in the game mapping.
- **SignerIsNotPlayer**: The signer is not one of the players in the game.
- **GameNotInProgress**: The game is not currently active and in progress.
- **PlayerHasNotAnActiveGame**: The player does not have an active game in progress.
- **SignerDidNotOpenTheGameAccount**: The signer did not create the game account and cannot close it.

## Running Tests

The project includes extensive unit tests to verify game mechanics and edge cases. Tests cover scenarios such as:

- **Game Initialization**: Validates proper setup of game accounts and initial state.
- **Move Validations**: Ensures moves are valid, detects winning conditions, and handles ties.
- **Turn Enforcement**: Checks that players cannot make moves out of turn.
- **Edge Cases**: Handles scenarios like attempts to move on an occupied tile or outside the board.

### How to Run Tests

1. **Start the Solana local validator**:
   ```bash
   solana-test-validator
   ```

2. **Run the tests**:
   ```bash
   anchor test
   ```

## Potential Enhancements

- **Frontend Integration**: Develop a user-friendly frontend using frameworks like React or Next.js to allow players to interact with the game more easily. This would include wallet connection, game setup, and real-time game updates.
- **Leaderboard**: Create a global leaderboard to rank players based on their win-loss records, incentivizing competitive play.
- **Analytics Dashboard**: Develop an on-chain analytics dashboard for tracking statistics like total games played, win rates, and average game duration.
- **Game History**: Allow players to query past game results and view the full history of moves, which could be useful for analysis or auditing.