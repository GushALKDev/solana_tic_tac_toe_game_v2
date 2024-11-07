import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { TicTacToe } from "../target/types/tic_tac_toe";
import { expect } from 'chai';
import { PublicKey } from '@solana/web3.js';
import { publicKey } from "@project-serum/anchor/dist/cjs/utils";

describe("tic-tac-toe", () => {
  let network:string;
  // Sets up the client to use the local cluster.
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);

  if (provider.connection.rpcEndpoint === "http://127.0.0.1:8899") network = "localhost";
  else if (provider.connection.rpcEndpoint === "https://api.devnet.solana.com") network = "devnet";


  const program = anchor.workspace.TicTacToe as Program<TicTacToe>;
  let globalStateAddress;

  // Initialize global state before each test
  let player1, player2, player3, player4, player5, player6;

  before(async () => {
    player1 = (program.provider as anchor.AnchorProvider).wallet; // Player 1
    player2 = anchor.web3.Keypair.generate(); // Player 2
    player3 = anchor.web3.Keypair.generate(); // Player 3
    player4 = anchor.web3.Keypair.generate(); // Player 4
    player5 = anchor.web3.Keypair.generate(); // Player 5
    player6 = anchor.web3.Keypair.generate(); // Player 6
    if (network === "localhost") {
        console.log("Funding players accounts...");
        await provider.connection.requestAirdrop(player2.publicKey, 200 * anchor.web3.LAMPORTS_PER_SOL); // Airdrop some SOL
        await provider.connection.requestAirdrop(player3.publicKey, 200 * anchor.web3.LAMPORTS_PER_SOL); // Airdrop some SOL
        await provider.connection.requestAirdrop(player4.publicKey, 200 * anchor.web3.LAMPORTS_PER_SOL); // Airdrop some SOL
        await provider.connection.requestAirdrop(player5.publicKey, 200 * anchor.web3.LAMPORTS_PER_SOL); // Airdrop some SOL
        await provider.connection.requestAirdrop(player6.publicKey, 200 * anchor.web3.LAMPORTS_PER_SOL); // Airdrop some SOL
    }

    [globalStateAddress] = await anchor.web3.PublicKey.findProgramAddress(
      [Buffer.from("global_state")],
      program.programId
    );

    console.log("Program ID:", program.programId.toString());
    console.log("Global State Address:", globalStateAddress.toString());
    console.log("Player 1 Address:", player1.publicKey.toString());
    console.log("Player 2 Address:", player2.publicKey.toString());
    console.log("Player 3 Address:", player3.publicKey.toString());
    console.log("Player 4 Address:", player4.publicKey.toString());
    console.log("Player 5 Address:", player5.publicKey.toString());
    console.log("Player 6 Address:", player6.publicKey.toString());
    
    // Calls a method to create and initialize the global state
    try {
      await program.account.globalState.fetch(globalStateAddress);
      console.log("Global state already exists, skipping initialization");
    } catch (e) {
      console.log("Global state does not exist, initializing");
      await program.methods.initializeGlobalState()
      .accounts({
        globalState: globalStateAddress,
        payer: player1.publicKey,
        systemProgram: anchor.web3.SystemProgram.programId,
      })
      .rpc();
    }
  });
  
  // Test to set up a game
  it('setup game player 1 no waiting games!', async () => {
    console.log("");
    console.log("----------------------------------------");
    console.log(">>> setup game player 1 no waiting games");
    console.log("----------------------------------------");
    // Calculate the PDA for the game account using the appropriate seeds
    const [globalStateAddress] = await anchor.web3.PublicKey.findProgramAddress(
      [Buffer.from("global_state")],
      program.programId
    );
    
    // Fetch global account
    const globalStatePDA = await program.account.globalState.fetch(globalStateAddress);
    
    const gameCount = globalStatePDA.gameCount.toString();
    
    const [gameAddress] = await anchor.web3.PublicKey.findProgramAddress(
      [
        Buffer.from("game"),
        Buffer.from(globalStatePDA.gameCount.toArray('le', 8)),
      ],
      program.programId
    );
    
    console.log("Game number: ", Number(gameCount));
    console.log("Game Address:", gameAddress.toString());

    // Call the game setup method
    await program.methods
      .setupGame()
      .accounts({
        globalState: globalStateAddress, // Ensure to pass the global account
        systemProgram: anchor.web3.SystemProgram.programId,
      })
      .rpc();

    // Fetch game state from the game account
    let gamePDA = await program.account.game.fetch(gameAddress);

    // Verify that players are set up correctly
    expect(gamePDA.players[0]).to.eql(player1.publicKey);
    // Verify that the game state is inactive
    expect(gamePDA.state).to.eql({ waiting: {} });
    // Verify that the board is empty
    expect(gamePDA.board).to.eql([
      [null, null, null],
      [null, null, null],
      [null, null, null],
    ]);
  });


  it('setup game player 1 with a waiting game already opened by himself!', async () => {
    console.log("");
    console.log("----------------------------------------");
    console.log(">>> setup game player 1 with a waiting game already opened by himself");
    console.log("----------------------------------------");
    // Calculate the PDA for the game account using the appropriate seeds
    const [globalStateAddress] = await anchor.web3.PublicKey.findProgramAddress(
      [Buffer.from("global_state")],
      program.programId
    );
    
    // Fetch global account
    const globalStatePDA = await program.account.globalState.fetch(globalStateAddress);
    
    const gameCount = globalStatePDA.gameCount.toString();
    
    const [gameAddress] = await anchor.web3.PublicKey.findProgramAddress(
      [
        Buffer.from("game"),
        Buffer.from(globalStatePDA.gameCount.toArray('le', 8)),
      ],
      program.programId
    );
    
    console.log("Game number: ", Number(gameCount));
    console.log("Game Address:", gameAddress.toString());

    try {
      // Call the game setup method
      await program.methods
        .setupGame()
        .accounts({
          globalState: globalStateAddress, // Ensure to pass the global account
          systemProgram: anchor.web3.SystemProgram.programId,
        })
        .rpc();
    }
    catch(error) {
      console.log(error.message);
      expect(error.message).to.contain("GameAlreadyInProgress"); // Check that the error message contains the expected text
    }
  });


  it('setup game player 2 with a waiting game!', async () => {
    console.log("");
    console.log("----------------------------------------");
    console.log(">>> setup game player 2 with a waiting game");
    console.log("----------------------------------------");
    // Calculate the PDA for the game account using the appropriate seeds
    const [globalStateAddress] = await anchor.web3.PublicKey.findProgramAddress(
      [Buffer.from("global_state")],
      program.programId
    );
    
    // Fetch global account
    const globalStatePDA = await program.account.globalState.fetch(globalStateAddress);
    
    const gameCount = globalStatePDA.gameCount.toString();
    
    console.log("Game number: ", Number(gameCount));

    // Call the game setup method
    await program.methods
      .setupGame()
      .accounts({
        globalState: globalStateAddress, // Ensure to pass the global account
        player: player2.publicKey.toString(),
        systemProgram: anchor.web3.SystemProgram.programId,
      })
      .signers([player2]) // PlayerTwo is the signer now
      .rpc();
  });

  // Test to set up a game
  it('setup game player 3 no waiting games!', async () => {
    console.log("");
    console.log("----------------------------------------");
    console.log(">>> setup game player 3 no waiting games");
    console.log("----------------------------------------");
    // Calculate the PDA for the game account using the appropriate seeds
    const [globalStateAddress] = await anchor.web3.PublicKey.findProgramAddress(
      [Buffer.from("global_state")],
      program.programId
    );
    
    // Fetch global account
    const globalStatePDA = await program.account.globalState.fetch(globalStateAddress);
    
    const gameCount = globalStatePDA.gameCount.toString();
    
    const [gameAddress] = await anchor.web3.PublicKey.findProgramAddress(
      [
        Buffer.from("game"),
        Buffer.from(globalStatePDA.gameCount.toArray('le', 8)),
      ],
      program.programId
    );
    
    console.log("Game number: ", Number(gameCount));
    console.log("Game Address:", gameAddress.toString());

    // Call the game setup method
    await program.methods
      .setupGame()
      .accounts({
        globalState: globalStateAddress, // Ensure to pass the global account
        player: player3.publicKey.toString(),
        systemProgram: anchor.web3.SystemProgram.programId,
      })
      .signers([player3])
      .rpc();

    // Fetch game state from the game account
    let gamePDA = await program.account.game.fetch(gameAddress);

    // // Verify that the turn is correctly initialized
    // expect(gamePDA.turn).to.equal(1); // Initial turn should be 1 since the game was setup.
    // Verify that players are set up correctly
    expect(gamePDA.players[0]).to.eql(player3.publicKey);
    // Verify that the game state is inactive
    expect(gamePDA.state).to.eql({ waiting: {} });
    // Verify that the board is empty
    expect(gamePDA.board).to.eql([
      [null, null, null],
      [null, null, null],
      [null, null, null],
    ]);
  });

  it('setup game player 4 with a waiting game!', async () => {
    console.log("");
    console.log("----------------------------------------");
    console.log(">>> setup game player 4 with a waiting game");
    console.log("----------------------------------------");
    // Calculate the PDA for the game account using the appropriate seeds
    const [globalStateAddress] = await anchor.web3.PublicKey.findProgramAddress(
      [Buffer.from("global_state")],
      program.programId
    );
    
    // Fetch global account
    const globalStatePDA = await program.account.globalState.fetch(globalStateAddress);
    
    const gameCount = globalStatePDA.gameCount.toString();
    
    console.log("Game number: ", Number(gameCount));

    // Call the game setup method
    await program.methods
      .setupGame()
      .accounts({
        globalState: globalStateAddress, // Ensure to pass the global account
        player: player4.publicKey.toString(),
        systemProgram: anchor.web3.SystemProgram.programId,
      })
      .signers([player4]) // PlayerTwo is the signer now
      .rpc();
  });

  it('setup game player 4 with an already opened game!', async () => {
    console.log("");
    console.log("----------------------------------------");
    console.log(">>> setup game player 4 with an already opened game");
    console.log("----------------------------------------");
    // Calculate the PDA for the game account using the appropriate seeds
    const [globalStateAddress] = await anchor.web3.PublicKey.findProgramAddress(
      [Buffer.from("global_state")],
      program.programId
    );
    
    // Fetch global account
    const globalStatePDA = await program.account.globalState.fetch(globalStateAddress);
    
    const gameCount = globalStatePDA.gameCount.toString();
    
    const [gameAddress] = await anchor.web3.PublicKey.findProgramAddress(
      [
        Buffer.from("game"),
        Buffer.from(globalStatePDA.gameCount.toArray('le', 8)),
      ],
      program.programId
    );
    
    console.log("Game number: ", Number(gameCount));
    console.log("Game Address:", gameAddress.toString());

    try {
      // Call the game setup method
      await program.methods
        .setupGame()
        .accounts({
          globalState: globalStateAddress, // Ensure to pass the global account
          player: player4.publicKey.toString(),
          systemProgram: anchor.web3.SystemProgram.programId,
        })
        .signers([player4])
        .rpc();
    }
    catch(error) {
      console.log(error.message);
      expect(error.message).to.contain("GameAlreadyInProgress"); // Check that the error message contains the expected text
    }
  });

  // Test to set up a game
  it('setup game player 5 no waiting games!', async () => {
    console.log("");
    console.log("----------------------------------------");
    console.log(">>> setup game player 5 no waiting games");
    console.log("----------------------------------------");
    // Calculate the PDA for the game account using the appropriate seeds
    const [globalStateAddress] = await anchor.web3.PublicKey.findProgramAddress(
      [Buffer.from("global_state")],
      program.programId
    );
    
    // Fetch global account
    const globalStatePDA = await program.account.globalState.fetch(globalStateAddress);
    
    const gameCount = globalStatePDA.gameCount.toString();
    
    const [gameAddress] = await anchor.web3.PublicKey.findProgramAddress(
      [
        Buffer.from("game"),
        Buffer.from(globalStatePDA.gameCount.toArray('le', 8)),
      ],
      program.programId
    );
    
    console.log("Game number: ", Number(gameCount));
    console.log("Game Address:", gameAddress.toString());

    // Call the game setup method
    await program.methods
      .setupGame()
      .accounts({
        globalState: globalStateAddress, // Ensure to pass the global account
        player: player5.publicKey.toString(),
        systemProgram: anchor.web3.SystemProgram.programId,
      })
      .signers([player5])
      .rpc();

    // Fetch game state from the game account
    let gamePDA = await program.account.game.fetch(gameAddress);

    // Verify that players are set up correctly
    expect(gamePDA.players[0]).to.eql(player5.publicKey);
    // Verify that the game state is inactive
    expect(gamePDA.state).to.eql({ waiting: {} });
    // Verify that the board is empty
    expect(gamePDA.board).to.eql([
      [null, null, null],
      [null, null, null],
      [null, null, null],
    ]);
  });

  it('setup game player 6 with a waiting game!', async () => {
    console.log("");
    console.log("----------------------------------------");
    console.log(">>> setup game player 6 with a waiting game");
    console.log("----------------------------------------");
    // Calculate the PDA for the game account using the appropriate seeds
    const [globalStateAddress] = await anchor.web3.PublicKey.findProgramAddress(
      [Buffer.from("global_state")],
      program.programId
    );
    
    // Fetch global account
    const globalStatePDA = await program.account.globalState.fetch(globalStateAddress);
    
    const gameCount = globalStatePDA.gameCount.toString();
    
    const [gameAddress] = await anchor.web3.PublicKey.findProgramAddress(
      [
        Buffer.from("game"),
        Buffer.from(globalStatePDA.gameCount.toArray('le', 8)),
      ],
      program.programId
    );
    
    console.log("Game number: ", Number(gameCount));
    console.log("Game Address:", gameAddress.toString());

    // Call the game setup method
    await program.methods
      .setupGame()
      .accounts({
        globalState: globalStateAddress, // Ensure to pass the global account
        player: player6.publicKey.toString(),
        systemProgram: anchor.web3.SystemProgram.programId,
      })
      .signers([player6]) // PlayerTwo is the signer now
      .rpc();
  });

  it('player 1 wins', async () => {
    console.log("");
    console.log("----------------------------------------");
    console.log(">>> player 1 wins");
    console.log("----------------------------------------");

    const globalState = await program.account.globalState.fetch(globalStateAddress);
    console.log("globalState", globalState);
    const gamePDA = getGamePDAFromPlayerPubKey(globalState, player1.publicKey.toString());
    // console.log("Game Address:", gamePDA);

    // Sequence of moves for player 1 to win:
    await play(program, player1, globalStateAddress, gamePDA,
      { row: 0, column: 0 },
      2,
      { inProgress: {} },
      [
        [{ x: {} }, null, null],
        [null, null, null],
        [null, null, null],
      ]
    );

    await play(program, player2, globalStateAddress, gamePDA,
      { row: 1, column: 0 },
      3,
      { inProgress: {} },
      [
        [{ x: {} }, null, null],
        [{ o: {} }, null, null],
        [null, null, null],
      ]
    );

    await play(program, player1, globalStateAddress, gamePDA,
      { row: 0, column: 1 },
      4,
      { inProgress: {} },
      [
        [{ x: {} }, { x: {} }, null],
        [{ o: {} }, null, null],
        [null, null, null],
      ]
    );

    await play(program, player2, globalStateAddress, gamePDA,
      { row: 1, column: 1 },
      5,
      { inProgress: {} },
      [
        [{ x: {} }, { x: {} }, null],
        [{ o: {} }, { o: {} }, null],
        [null, null, null],
      ]
    );

    // Final move for player 1 to win
    await play(program, player1, globalStateAddress, gamePDA,
      { row: 0, column: 2 },
      6,
      { won: { winner: player1.publicKey } },
      [
        [{ x: {} }, { x: {} }, { x: {} }],
        [{ o: {} }, { o: {} }, null],
        [null, null, null],
      ]
    );
  });

  it('player 4 plays out of turn', async () => {
    console.log("");
    console.log("----------------------------------------");
    console.log(">>> player 4 plays out of turn");
    console.log("----------------------------------------");

    const globalState = await program.account.globalState.fetch(globalStateAddress);
    // console.log("globalState", globalState);
    const gamePDA = getGamePDAFromPlayerPubKey(globalState, player4.publicKey.toString());
    // console.log("Game Address:", gamePDA);

    try {
      await play(program, player4, globalStateAddress, gamePDA,
        { row: 0, column: 0 },
        2,
        { inProgress: {} },
        [
          [{ x: {} }, null, null],
          [null, null, null],
          [null, null, null],
        ]
      );
    }
    catch(error) {
      console.log(error.message);
      expect(error.message).to.contain("NotPlayersTurn"); // Check that the error message contains the expected text
    }
  });

  it('player 3 plays twice', async () => {
    console.log("");
    console.log("----------------------------------------");
    console.log(">>> player 3 plays twice");
    console.log("----------------------------------------");

    const globalState = await program.account.globalState.fetch(globalStateAddress);
    // console.log("globalState", globalState);
    const gamePDA = getGamePDAFromPlayerPubKey(globalState, player3.publicKey.toString());
    // console.log("Game Address:", gamePDA);

    await play(program, player3, globalStateAddress, gamePDA,
      { row: 0, column: 0 },
      2,
      { inProgress: {} },
      [
        [{ x: {} }, null, null],
        [null, null, null],
        [null, null, null],
      ]
    );

    try {
      await play(program, player3, globalStateAddress, gamePDA,
        { row: 1, column: 0 },
        3,
        { inProgress: {} },
        [
          [{ x: {} }, null, null],
          [{ o: {} }, null, null],
          [null, null, null],
        ]
      );
    }
    catch(error) {
      console.log(error.message);
      expect(error.message).to.contain("NotPlayersTurn"); // Check that the error message contains the expected text
    }
  });

  // Test to simulate player 1 plays twice
  it('player 4 plays out of bounds', async () => {
    console.log("");
    console.log("----------------------------------------");
    console.log(">>> player 4 plays out of bounds");
    console.log("----------------------------------------");

    const globalState = await program.account.globalState.fetch(globalStateAddress);
    // console.log("globalState", globalState);
    const gamePDA = getGamePDAFromPlayerPubKey(globalState, player4.publicKey.toString());
    // console.log("Game Address:", gamePDA);

    try {
      await play(program, player4, globalStateAddress, gamePDA,
        { row: 3, column: 0 },
        3,
        { inProgress: {} },
        [
          [{ x: {} }, null, null],
          [{ o: {} }, null, null],
          [null, null, null],
        ]
      );
    }
    catch(error) {
      console.log(error.message);
      expect(error.message).to.contain("TileOutOfBounds"); // Check that the error message contains the expected text
    }
  });

  // Test to simulate player 1 plays twice
  it('player 4 plays on an already set tile', async () => {
    console.log("");
    console.log("----------------------------------------");
    console.log(">>> player 4 plays on an already set tile");
    console.log("----------------------------------------");

    const globalState = await program.account.globalState.fetch(globalStateAddress);
    // console.log("globalState", globalState);
    const gamePDA = getGamePDAFromPlayerPubKey(globalState, player4.publicKey.toString());
    // console.log("Game Address:", gamePDA);

    try {
      await play(program, player4, globalStateAddress, gamePDA,
        { row: 0, column: 0 },
        3,
        { inProgress: {} },
        [
          [{ x: {} }, null, null],
          [null, null, null],
          [null, null, null],
        ]
      );
    }
    catch(error) {
      console.log(error.message);
      expect(error.message).to.contain("TileAlreadySet"); // Check that the error message contains the expected text
    }
  });

  it('player 4 wins', async () => {
    console.log("");
    console.log("----------------------------------------");
    console.log(">>> player 4 wins");
    console.log("----------------------------------------");

    const globalState = await program.account.globalState.fetch(globalStateAddress);
    // console.log("globalState", globalState);
    const gamePDA = getGamePDAFromPlayerPubKey(globalState, player4.publicKey.toString());
    console.log("Game Address:", gamePDA);

    await play(program, player4, globalStateAddress, gamePDA,
      { row: 1, column: 0 },
      3,
      { inProgress: {} },
      [
        [{ x: {} }, null, null],
        [{ o: {} }, null, null],
        [null, null, null],
      ]
    );

    await play(program, player3, globalStateAddress, gamePDA,
      { row: 0, column: 1 },
      4,
      { inProgress: {} },
      [
        [{ x: {} }, { x: {} }, null],
        [{ o: {} }, null, null],
        [null, null, null],
      ]
    );

    await play(program, player4, globalStateAddress, gamePDA,
      { row: 1, column: 1 },
      5,
      { inProgress: {} },
      [
        [{ x: {} }, { x: {} }, null],
        [{ o: {} }, { o: {} }, null],
        [null, null, null],
      ]
    );

    // Final move for player 1 to win
    await play(program, player3, globalStateAddress, gamePDA,
      { row: 0, column: 2 },
      6,
      { won: { winner: player3.publicKey } },
      [
        [{ x: {} }, { x: {} }, { x: {} }],
        [{ o: {} }, { o: {} }, null],
        [null, null, null],
      ]
    );
  });

  it('player 4 fails on closing the account', async () => {
    console.log("");
    console.log("----------------------------------------");
    console.log(">>> player 4 fails on closing the account");
    console.log("----------------------------------------");

    const globalState = await program.account.globalState.fetch(globalStateAddress);
    // console.log("globalState", globalState);
    const [gameAddress] = await anchor.web3.PublicKey.findProgramAddress(
      [
        Buffer.from("game"),
        Buffer.from([2, 0, 0, 0, 0, 0, 0, 0]),
      ],
      program.programId
    );
    console.log("Game Address:", gameAddress);

    try {
      // Call the game setup method
      await program.methods
        .closeGameAccount()
        .accounts({
          game: gameAddress, // Ensure to pass the global account
          signer: player4.publicKey.toString(),
          systemProgram: anchor.web3.SystemProgram.programId,
        })
        .signers([player4]) // PlayerTwo is the signer now
        .rpc();
      }
      catch(error) {
        console.log(error);
        expect(error.message).to.contain("SignerDidNotOpenTheGameAccount."); // Check that the error message contains the expected text
      }
  });

  it('player 3 close the game account', async () => {
    console.log("");
    console.log("----------------------------------------");
    console.log(">>> player 3 close the game account");
    console.log("----------------------------------------");

    const globalState = await program.account.globalState.fetch(globalStateAddress);
    // console.log("globalState", globalState);
    const [gameAddress] = await anchor.web3.PublicKey.findProgramAddress(
      [
        Buffer.from("game"),
        Buffer.from([2, 0, 0, 0, 0, 0, 0, 0]),
      ],
      program.programId
    );
    let gamePDA = await program.account.game.fetch(gameAddress);

    // Account exists before
    console.log("Game Address:", gameAddress);
    // console.log("Game PDA:", gamePDA);

    // Call the game setup method
    await program.methods
      .closeGameAccount()
      .accounts({
        game: gameAddress, // Ensure to pass the global account
        signer: player3.publicKey.toString(),
        systemProgram: anchor.web3.SystemProgram.programId,
      })
      .signers([player3]) // PlayerTwo is the signer now
      .rpc();
    
    // Account doesn´t exist later
    try {
      let gamePDA = await program.account.game.fetch(gameAddress);
      // console.log("Game PDA:", gamePDA);
    }
    catch(error) {
      console.log(error);
      expect(error.message).to.contain("Account does not exist or has no data"); // Check that the error message contains the expected text
    }
  });

  it('game draw for players 5 & 6', async () => {
    console.log("");
    console.log("----------------------------------------");
    console.log(">>> game draw for players 5 & 6");
    console.log("----------------------------------------");

    const globalState = await program.account.globalState.fetch(globalStateAddress);
    const gamePDA = getGamePDAFromPlayerPubKey(globalState, player5.publicKey.toString());

    console.log("Game Address:", gamePDA);

    // Sequence of moves for a draw:
    await play(program, player5, globalStateAddress, gamePDA,
      { row: 0, column: 0 },
      2,
      { inProgress: {} },
      [
        [{ x: {} }, null, null],
        [null, null, null],
        [null, null, null],
      ]
    );

    await play(program, player6, globalStateAddress, gamePDA,
      { row: 0, column: 1 },
      3,
      { inProgress: {} },
      [
        [{ x: {} }, { o: {} }, null],
        [null, null, null],
        [null, null, null],
      ]
    );

    await play(program, player5, globalStateAddress, gamePDA,
      { row: 0, column: 2 },
      4,
      { inProgress: {} },
      [
        [{ x: {} }, { o: {} }, { x: {} }],
        [null, null, null],
        [null, null, null],
      ]
    );

    await play(program, player6, globalStateAddress, gamePDA,
      { row: 1, column: 1 },
      5,
      { inProgress: {} },
      [
        [{ x: {} }, { o: {} }, { x: {} }],
        [null, { o: {} }, null],
        [null, null, null],
      ]
    );

    await play(program, player5, globalStateAddress, gamePDA,
      { row: 1, column: 0 },
      6,
      { inProgress: {} },
      [
        [{ x: {} }, { o: {} }, { x: {} }],
        [{ x: {} }, { o: {} }, null],
        [null, null, null],
      ]
    );

    await play(program, player6, globalStateAddress, gamePDA,
      { row: 1, column: 2 },
      7,
      { inProgress: {} },
      [
        [{ x: {} }, { o: {} }, { x: {} }],
        [{ x: {} }, { o: {} }, { o: {} }],
        [null, null, null],
      ]
    );

    await play(program, player5, globalStateAddress, gamePDA,
      { row: 2, column: 1 },
      8,
      { inProgress: {} },
      [
        [{ x: {} }, { o: {} }, { x: {} }],
        [{ x: {} }, { o: {} }, { o: {} }],
        [null, { x: {} }, null],
      ]
    );

    await play(program, player6, globalStateAddress, gamePDA,
      { row: 2, column: 0 },
      9,
      { inProgress: {} },
      [
        [{ x: {} }, { o: {} }, { x: {} }],
        [{ x: {} }, { o: {} }, { o: {} }],
        [{ o: {} }, { x: {} }, null],
      ]
    );

    // Final move for a draw
    await play(program, player5, globalStateAddress, gamePDA,
      { row: 2, column: 2 },
      10,
      { tie: {} }, // Aquí el estado esperado es un empate
      [
        [{ x: {} }, { o: {} }, { x: {} }],
        [{ x: {} }, { o: {} }, { o: {} }],
        [{ o: {} }, { x: {} }, { x: {} }],
      ]
    );
  });

  it('player 4 has not an active game', async () => {
    console.log("");
    console.log("----------------------------------------");
    console.log(">>> player 4 has not an active game");
    console.log("----------------------------------------");

    const globalStatePDA = await program.account.globalState.fetch(globalStateAddress);
    // console.log("globalState", globalState);
    const [derivedGameAddress] =  await anchor.web3.PublicKey.findProgramAddress(
      [
        Buffer.from("game"),
        Buffer.from([1, 0, 0, 0, 0, 0, 0, 0]), // Game Count 2
      ],
      program.programId
    );
    console.log("Game Address:", derivedGameAddress);
    const gamePDA = await program.account.game.fetch(derivedGameAddress);
    console.log("Game PDA:", gamePDA);

    try {
      await play(program, player4, globalStateAddress, derivedGameAddress,
        { row: 0, column: 0 },
        2,
        { inProgress: {} },
        [
          [{ x: {} }, null, null],
          [null, null, null],
          [null, null, null],
        ]
      );
    }
    catch(error) {
      console.log(error.message);
      expect(error.message).to.contain("PlayerHasNotAnActiveGame"); // Check that the error message contains the expected text
    }
  });

  it('player 4 plays on an end game', async () => {
    console.log("");
    console.log("----------------------------------------");
    console.log(">>> player 4 plays on an end game");
    console.log("----------------------------------------");
    
    // Calculate the PDA for the game account using the appropriate seeds
    const [globalStateAddress] = await anchor.web3.PublicKey.findProgramAddress(
      [Buffer.from("global_state")],
      program.programId
    );
    
    // Fetch global account
    const globalStatePDA = await program.account.globalState.fetch(globalStateAddress);
    
    const gameCount = globalStatePDA.gameCount.toString();
    
    const [gameAddress] = await anchor.web3.PublicKey.findProgramAddress(
      [
        Buffer.from("game"),
        Buffer.from(globalStatePDA.gameCount.toArray('le', 8)),
      ],
      program.programId
    );
    
    console.log("Game number: ", Number(gameCount));
    console.log("Game Address:", gameAddress.toString());

    // Call the game setup method
    await program.methods
      .setupGame()
      .accounts({
        globalState: globalStateAddress, // Ensure to pass the global account
        player: player4.publicKey.toString(),
        systemProgram: anchor.web3.SystemProgram.programId,
      })
      .signers([player4])
      .rpc();

    // Fetch game state from the game account
    let gamePDA = await program.account.game.fetch(gameAddress);

    // // Verify that the turn is correctly initialized
    // expect(gamePDA.turn).to.equal(1); // Initial turn should be 1 since the game was setup.
    // Verify that players are set up correctly
    expect(gamePDA.players[0]).to.eql(player4.publicKey);
    // Verify that the game state is inactive
    expect(gamePDA.state).to.eql({ waiting: {} });
    // Verify that the board is empty
    expect(gamePDA.board).to.eql([
      [null, null, null],
      [null, null, null],
      [null, null, null],
    ]);

    const globalState = await program.account.globalState.fetch(globalStateAddress);
    // console.log("globalState", globalState);

    const [finishedGameAddress] =  await anchor.web3.PublicKey.findProgramAddress(
      [
        Buffer.from("game"),
        Buffer.from([1, 0, 0, 0, 0, 0, 0, 0]), // Game Count 1
      ],
      program.programId
    );
    console.log("Game Address:", finishedGameAddress);
    gamePDA = await program.account.game.fetch(finishedGameAddress);
    console.log("Game PDA:", gamePDA);

    try {
      await play(program, player4, globalStateAddress, finishedGameAddress,
        { row: 0, column: 0 },
        2,
        { inProgress: {} },
        [
          [{ x: {} }, null, null],
          [null, null, null],
          [null, null, null],
        ]
      );
    }
    catch(error) {
      console.log(error.message);
      expect(error.message).to.contain("GameAlreadyOver"); // Check that the error message contains the expected text
    }
  });

  it('player 4 cancel a wating game', async () => {
    console.log("");
    console.log("----------------------------------------");
    console.log(">>> player 4 cancel a wating game");
    console.log("----------------------------------------");

    const globalState = await program.account.globalState.fetch(globalStateAddress);
    // console.log("globalState", globalState);
    const [gameAddress] = await anchor.web3.PublicKey.findProgramAddress(
      [
        Buffer.from("game"),
        Buffer.from([4, 0, 0, 0, 0, 0, 0, 0]),
      ],
      program.programId
    );
    let gamePDA = await program.account.game.fetch(gameAddress);

    // Account exists before
    console.log("Game Address:", gameAddress);
    console.log("Game PDA:", gamePDA);

    // Call the game setup method
    await program.methods
      .cancelGame()
      .accounts({
        globalState: globalStateAddress,
        game: gameAddress, // Ensure to pass the global account
        signer: player4.publicKey.toString(),
        systemProgram: anchor.web3.SystemProgram.programId,
      })
      .signers([player4]) // PlayerTwo is the signer now
      .rpc();
    
    // Account doesn´t exist later
    try {
      let gamePDA = await program.account.game.fetch(gameAddress);
      console.log("Game PDA:", gamePDA);
    }
    catch(error) {
      console.log(error);
      expect(error.message).to.contain("Account does not exist or has no data"); // Check that the error message contains the expected text
    }
  });

  it('game state', async () => {
    console.log("");
    console.log("----------------------------------------");
    console.log(">>> game status");
    console.log("----------------------------------------");

    const globalState = await program.account.globalState.fetch(globalStateAddress);
    console.log("globalState", globalState);
  });
});

// Helper function to simulate a move
async function play(
  program: Program<TicTacToe>,
  player,
  global_state,
  game,
  tile,
  expectedTurn,
  expectedGameState,
  expectedBoard
) {
  // Make the move
  await program.methods
    .play(tile)
    .accounts({
      globalState: global_state,
      game: game,
      player: player.publicKey,
    })
    .signers(player instanceof (anchor.Wallet as any) ? [] : [player])
    .rpc();

  // Verify game state after the move
  const gameState = await program.account.game.fetch(game);
  
  console.log("gameState",gameState);

  if (expectedTurn > gameState.turn) {
    // Player won
    if (gameState.state.won != undefined ) {
      expect(gameState.state.won).not.to.eql(undefined);
    }
    // Tie
    else if (gameState.state.tie != undefined ) {
      expect(gameState.state.tie).not.to.eql(undefined);
    }
  }
  else {
    // Turn must be equal as expected
    expect(gameState.turn).to.equal(expectedTurn);
  }
  expect(gameState.state).to.eql(expectedGameState);
  expect(gameState.board).to.eql(expectedBoard);
}

function getGamePDAFromPlayerPubKey(globalStatePDA, playerPubKey) {
  // Find the index of the target public key
    const gameIndex = globalStatePDA.playersMapping.map((pubKey) => pubKey.toString()).indexOf(playerPubKey);
    if (gameIndex !== -1) {
      const gamePDA = globalStatePDA.gamesMapping[gameIndex].toString();
      return gamePDA;
    }
    else {
      return -1;
    }
}
