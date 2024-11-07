// checkGlobalState.js
process.env.ANCHOR_WALLET = process.env.HOME + '/.config/solana/id.json';

const anchor = require('@project-serum/anchor');
const { Connection, PublicKey, clusterApiUrl } = require('@solana/web3.js');
const idl = require('../target/idl/tic_tac_toe.json'); // Asegúrate de que la ruta al IDL sea correcta
const programId = new PublicKey('GxyzppqnU8SuQKggxorknHp9NeLuUNA4hU69av6YzunW'); // Coloca aquí tu program ID
const network = clusterApiUrl('devnet');

// Configuración de conexión y proveedor de Anchor
const connection = new Connection(network, "processed");
const wallet = anchor.Wallet.local();
const provider = new anchor.AnchorProvider(connection, wallet, {
    preflightCommitment: "processed",
});
anchor.setProvider(provider);

(async () => {
    try {
        console.log("Starting...");
        const program = new anchor.Program(idl, programId, provider);

        // Calcula la dirección de la cuenta global_state (PDA)
        const [globalStateAddress] = await PublicKey.findProgramAddress(
            [Buffer.from("global_state")],
            program.programId
        );

        console.log("globalStateAddress:", globalStateAddress);

        // Verifica si la cuenta ya está inicializada
        const accountInfo = await connection.getAccountInfo(globalStateAddress);
        if (accountInfo) {
            console.log("La cuenta global_state ya está inicializada.");
            return;
        }

        console.log("Inicializando la cuenta global_state...");

        // Llama a la función initialize_global_state del programa sin argumentos
        await program.rpc.initializeGlobalState({
            accounts: {
                globalState: globalStateAddress,
                payer: provider.wallet.publicKey,
                systemProgram: anchor.web3.SystemProgram.programId,
            },
            signers: [],
        });

        console.log("¡Cuenta global_state inicializada con éxito!");

    } catch (error) {
        console.error("Error al inicializar la cuenta global_state:", error);
    }
})();
