const anchor = require('@project-serum/anchor');
const { PublicKey, SYSVAR_RENT_PUBKEY, SystemProgram, Keypair, Transaction } = anchor.web3;
const { Token, TOKEN_PROGRAM_ID, AccountLayout } = require('@solana/spl-token');

const DECIMALS = 9;
const createMint = async (connection, payer, authority) => {
    return await Token.createMint(
        connection, payer, 
        authority.publicKey, // mint authority
        authority.publicKey, // freeze authority
        DECIMALS, TOKEN_PROGRAM_ID,
    );
}

const createTokenAccount = async (connection, payer, mint, owner) => {
	let keypair = Keypair.generate();
	let tx = new Transaction();
  	tx.add(
		SystemProgram.createAccount({
			fromPubkey: payer.publicKey,
			newAccountPubkey: keypair.publicKey,
			space: AccountLayout.span,
			lamports: await Token.getMinBalanceRentForExemptAccount(connection),
			programId: TOKEN_PROGRAM_ID,
		}),
		Token.createInitAccountInstruction(
			TOKEN_PROGRAM_ID, // program id, always token program id
			mint, // mint
			keypair.publicKey, // token account public key
			owner // token account authority
		)
  	);
	await connection.sendTransaction(tx, [payer, keypair]);
	return keypair.publicKey;
}


describe('farms', () => {

	// Configure the client to use the local cluster.
	const provider = anchor.Provider.env();
	const connection = provider.connection;
	const payer = provider.wallet.payer;
	anchor.setProvider(provider);

	const program = anchor.workspace.Farms;

	// PDA functions

	const findRewarderPDA = async (manager) => {
		return await PublicKey.findProgramAddress(
			[Buffer.from("rewarder"), manager.toBuffer()], 
			program.programId
		);
	};

	const findCrop = async (manager, num) => {
		let farmNumBuffer = Buffer.alloc(8);
		farmNumBuffer.writeBigUInt64LE(BigInt(num), 0, 8);
		return await PublicKey.findProgramAddress(
			[Buffer.from("crop"), manager.toBuffer(), farmNumBuffer], 
			program.programId
		);
	};

	const findPlot = async (manager, farmer, num) => {
		let farmNumBuffer = Buffer.alloc(8);
		farmNumBuffer.writeBigUInt64LE(BigInt(num), 0, 8);
		return await PublicKey.findProgramAddress(
			[Buffer.from("plot"), manager.toBuffer(), farmer.toBuffer(), farmNumBuffer], 
			program.programId
		);
	};

	const managerA = Keypair.generate();

	var depositMintA;
	var depositMintB;
	var depositMintC;

	var rewardMintA;
	var rewardMintB;
	var rewardMintC;

	before(async () => {
		depositMintA = await createMint(connection, payer, payer);
		depositMintB = await createMint(connection, payer, payer);
		depositMintC = await createMint(connection, payer, payer);
		rewardMintA = await createMint(connection, payer, payer);
		rewardMintB = await createMint(connection, payer, payer);
		rewardMintC = await createMint(connection, payer, payer);
	});

	it('appoint manager', async () => {

		const [rewarderPda, rewarderBump] = await findRewarderPDA(managerA.publicKey);
		await program.rpc.appoint(rewarderBump, {
			accounts: {
				manager: managerA.publicKey,
				rewarderPda,
				signer: payer.publicKey,
				systemProgram: SystemProgram.programId,
				rent: SYSVAR_RENT_PUBKEY
			},
			signers: [managerA]
		});

	});

	const cultivateCrop = async (manager, depositMint, rewardMint, id) => {
		const [rewarderPda, ] = await findRewarderPDA(manager.publicKey);

		const depositTreasuryKeypair = Keypair.generate();
		const rewardTreasuryKeypair = Keypair.generate();

		const depositFee = new anchor.BN(0);
		const withdrawFee = new anchor.BN(0);
		const endTimestamp = new anchor.BN(0);
		const rewardRate = new anchor.BN(0);
		const [crop, seed] = await findCrop(manager.publicKey, id);

		await program.rpc.cultivate(
			seed,
			depositFee, 
			withdrawFee,
			endTimestamp,
			rewardRate,
			{
				accounts: {
					manager: manager.publicKey,
					rewarderPda,
					crop,
					depositTreasury: depositTreasuryKeypair.publicKey,
					rewardTreasury: rewardTreasuryKeypair.publicKey,
					depositMint: depositMint.publicKey,
					rewardMint: rewardMint.publicKey,
					owner: payer.publicKey,
					tokenProgram: TOKEN_PROGRAM_ID,
					systemProgram: SystemProgram.programId,
					rent: SYSVAR_RENT_PUBKEY
				},
				signers: [depositTreasuryKeypair, rewardTreasuryKeypair]
			}
		)
	}

	it('cultivate crop', async () => {
		await cultivateCrop(managerA, depositMintA, rewardMintA, 0);
		await cultivateCrop(managerA, depositMintB, rewardMintB, 1);
		await cultivateCrop(managerA, depositMintC, rewardMintC, 2);
	});

	const tillPlot = async (manager, farmer, id) => {
		const [plot, seed] = await findPlot(manager, farmer, id);
		await program.rpc.till(seed, new anchor.BN(id), {
			accounts: {
				manager,
				plot,
				farmer,
				systemProgram: SystemProgram.programId,
				rent: SYSVAR_RENT_PUBKEY
			}
		})
	}

	it('till plot', async () => {
		await tillPlot(managerA.publicKey, payer.publicKey, 0);
		await tillPlot(managerA.publicKey, payer.publicKey, 1);
		await tillPlot(managerA.publicKey, payer.publicKey, 2);
	})

	const sow = async (manager, farmer, amount, id) => {

		const [crop, ] = await findCrop(manager, id);
		const [plot, ] = await findPlot(manager, farmer, id);

		let cropData = await program.account.crop.fetch(crop);
		// let fromTokenAccount = await createTokenAccount(connection, payer, cropData.depositMint, farmer);
		let mint = new Token(connection, cropData.depositMint, TOKEN_PROGRAM_ID, payer);
		let fromTokenAccount = await mint.createAccount(farmer);
		await mint.mintTo(fromTokenAccount, payer.publicKey, [], amount);

		await program.rpc.sow(new anchor.BN(amount), {
			accounts: {
				crop,
				plot,
				depositTreasury: cropData.depositTreasury,
				fromTokenAccount,
				farmer,
				tokenProgram: TOKEN_PROGRAM_ID
			}
		});

	}

	const uproot = async (manager, farmer, amount, id) => {

		const [rewarderPda, ] = await findRewarderPDA(manager);
		const [crop, ] = await findCrop(manager, id);
		const [plot, ] = await findPlot(manager, farmer, id);

		let cropData = await program.account.crop.fetch(crop);

		let depositMint = new Token(connection, cropData.depositMint, TOKEN_PROGRAM_ID, payer);
		let depositTokenAccount = await depositMint.createAccount(farmer);

		let rewardMint = new Token(connection, cropData.rewardMint, TOKEN_PROGRAM_ID, payer);
		let rewardTokenAccount = await rewardMint.createAccount(farmer);

		let { rewardTreasury, depositTreasury } = cropData;

		await program.rpc.uproot(new anchor.BN(amount), {
			accounts: {
				manager,
				rewarderPda,
				crop,
				plot,
				depositTreasury,
				rewardTreasury,
				depositTokenAccount,
				rewardTokenAccount,
				farmer,
				tokenProgram: TOKEN_PROGRAM_ID
			}
		});

	}

	it("sows", async () => {
		await sow(managerA.publicKey, payer.publicKey, 1, 0);
		await uproot(managerA.publicKey, payer.publicKey, 1, 0);
	});

	// it("Can transfer ownership", async () => {
	// 	const [authority, ] = await findAuthority();
	// 	const tx = await program.rpc.transferOwnership(payer.publicKey, {
	// 		accounts: {
	// 			authority,
	// 			owner: payer.publicKey,
	// 		},
	// 	});
	// });

	// it("Creates new farm", async () => {
	// 	const [authority, ] = await findAuthority();
	// 	const [farm, farmBump] = await findFarm(0);
	// 	const depositMintToken = await createMint(connection, payer, payer);
	// 	const feeTokenAccountKeypair = Keypair.generate();
	// 	const depositTreasuryKeypair = Keypair.generate();
	// 	const tx = await program.rpc.createFarm(farmBump, {
	// 		accounts: {
	// 			authority,
	// 			farm,
	// 			feeTokenAccount: feeTokenAccountKeypair.publicKey,
	// 			depositTreasury: depositTreasuryKeypair.publicKey,
	// 			depositMint: depositMintToken.publicKey,
	// 			owner: payer.publicKey,
	// 			tokenProgram: TOKEN_PROGRAM_ID,
	// 			systemProgram: SystemProgram.programId,
	// 			rent: SYSVAR_RENT_PUBKEY
	// 		},
	// 		signers: [depositTreasuryKeypair, feeTokenAccountKeypair]
	// 	});
	// })

});
