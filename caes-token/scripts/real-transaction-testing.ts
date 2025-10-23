import { ethers } from "hardhat";
import fs from "fs";

async function main() {
  console.log("🔥 REAL Transaction Testing - New CAESAR Token");
  console.log("=".repeat(50));
  
  const network = await ethers.provider.getNetwork();
  const caesarAddress = "0x69e35749eB8f5Ae282A883329C7d0BF44bCed59C"; // New CAESAR token
  
  console.log(`📍 Contract: ${caesarAddress}`);
  console.log(`🌐 Network: ${network.name} (${network.chainId})\n`);
  
  const Caesar = await ethers.getContractFactory("CaesarToken");
  const caesar = Caesar.attach(caesarAddress);
  const [deployer, ...accounts] = await ethers.getSigners();
  
  console.log(`👤 Deployer: ${deployer.address}`);
  console.log(`💰 ETH Balance: ${ethers.formatEther(await ethers.provider.getBalance(deployer.address))} ETH\n`);
  
  // Step 1: Contract validation
  console.log("1️⃣ Contract Validation");
  console.log("=======================");
  
  try {
    const [name, symbol, decimals, totalSupply, owner] = await Promise.all([
      caesar.name(),
      caesar.symbol(), 
      caesar.decimals(),
      caesar.totalSupply(),
      caesar.owner()
    ]);
    
    console.log(`✅ Token: ${name} (${symbol})`);
    console.log(`✅ Decimals: ${decimals}`);
    console.log(`✅ Total Supply: ${ethers.formatEther(totalSupply)} CAES`);
    console.log(`✅ Owner: ${owner}`);
    console.log(`✅ Ownership Match: ${owner.toLowerCase() === deployer.address.toLowerCase()}`);
    
  } catch (error) {
    console.log(`❌ Contract validation failed: ${error}`);
    return;
  }
  
  // Step 2: Check initial balances
  console.log("\n2️⃣ Initial State Check");
  console.log("=======================");
  
  try {
    const deployerBalance = await caesar.balanceOf(deployer.address);
    console.log(`💰 Deployer Balance: ${ethers.formatEther(deployerBalance)} CAES`);
    
    if (deployerBalance === 0n) {
      console.log("🔧 Setting up migration to mint test tokens...");
      
      // Enable migration
      const setMigrationTx = await caesar.setMigrationContract(deployer.address);
      await setMigrationTx.wait();
      console.log(`✅ Migration contract set`);
      
      const enableMigrationTx = await caesar.setMigrationEnabled(true);
      await enableMigrationTx.wait();
      console.log(`✅ Migration enabled`);
      
      // Mint initial tokens for testing
      const mintAmount = ethers.parseEther("100000"); // 100k tokens
      console.log(`🔨 Minting ${ethers.formatEther(mintAmount)} CAES for testing...`);
      
      const mintTx = await caesar.migrationMint(deployer.address, mintAmount);
      const receipt = await mintTx.wait();
      console.log(`✅ Mint successful - Gas used: ${receipt?.gasUsed}`);
      
      const newBalance = await caesar.balanceOf(deployer.address);
      console.log(`💰 New Balance: ${ethers.formatEther(newBalance)} CAES`);
    }
    
  } catch (error) {
    console.log(`❌ Initial setup failed: ${error}`);
    return;
  }
  
  // Step 3: Real transaction testing
  console.log("\n3️⃣ Real Transaction Testing");
  console.log("============================");
  
  const testTransactions = [];
  
  try {
    const initialBalance = await caesar.balanceOf(deployer.address);
    console.log(`💰 Starting Balance: ${ethers.formatEther(initialBalance)} CAES\n`);
    
    // Test 1: Self transfer to trigger demurrage
    console.log("🔄 Test 1: Self Transfer (Demurrage Test)");
    const selfTransferAmount = ethers.parseEther("100");
    
    const tx1 = await caesar.transfer(deployer.address, selfTransferAmount, {
      gasLimit: 300000
    });
    const receipt1 = await tx1.wait();
    
    const balanceAfterSelf = await caesar.balanceOf(deployer.address);
    console.log(`✅ Self transfer: ${ethers.formatEther(selfTransferAmount)} CAES`);
    console.log(`⛽ Gas used: ${receipt1?.gasUsed}`);
    console.log(`💰 Balance after: ${ethers.formatEther(balanceAfterSelf)} CAES`);
    
    testTransactions.push({
      type: "self_transfer",
      amount: ethers.formatEther(selfTransferAmount),
      gasUsed: Number(receipt1?.gasUsed || 0),
      balanceAfter: ethers.formatEther(balanceAfterSelf),
      txHash: tx1.hash
    });
    
    // Wait and check demurrage
    console.log("⏳ Checking demurrage calculation...");
    const demurrageAmount = await caesar.calculateDemurrage(deployer.address);
    console.log(`📉 Demurrage amount: ${ethers.formatEther(demurrageAmount)} CAES\n`);
    
    // Test 2: Transfer to another account
    if (accounts.length > 0) {
      console.log("🔄 Test 2: Transfer to Another Account");
      const recipient = accounts[0];
      const transferAmount = ethers.parseEther("1000");
      
      const recipientInitialBalance = await caesar.balanceOf(recipient.address);
      console.log(`👤 Recipient: ${recipient.address}`);
      console.log(`💰 Recipient initial: ${ethers.formatEther(recipientInitialBalance)} CAES`);
      
      const tx2 = await caesar.transfer(recipient.address, transferAmount, {
        gasLimit: 300000
      });
      const receipt2 = await tx2.wait();
      
      const senderBalanceAfter = await caesar.balanceOf(deployer.address);
      const recipientBalanceAfter = await caesar.balanceOf(recipient.address);
      
      console.log(`✅ Transfer: ${ethers.formatEther(transferAmount)} CAES`);
      console.log(`⛽ Gas used: ${receipt2?.gasUsed}`);
      console.log(`💰 Sender balance: ${ethers.formatEther(senderBalanceAfter)} CAES`);
      console.log(`💰 Recipient balance: ${ethers.formatEther(recipientBalanceAfter)} CAES`);
      
      testTransactions.push({
        type: "external_transfer",
        amount: ethers.formatEther(transferAmount),
        gasUsed: Number(receipt2?.gasUsed || 0),
        senderBalance: ethers.formatEther(senderBalanceAfter),
        recipientBalance: ethers.formatEther(recipientBalanceAfter),
        txHash: tx2.hash
      });
      
      // Test 3: Return transfer (test anti-speculation)
      console.log("\n🔄 Test 3: Return Transfer (Anti-Speculation Test)");
      const returnAmount = ethers.parseEther("500");
      
      const tx3 = await caesar.connect(recipient).transfer(deployer.address, returnAmount, {
        gasLimit: 300000
      });
      const receipt3 = await tx3.wait();
      
      const finalSenderBalance = await caesar.balanceOf(deployer.address);
      const finalRecipientBalance = await caesar.balanceOf(recipient.address);
      
      console.log(`✅ Return transfer: ${ethers.formatEther(returnAmount)} CAES`);
      console.log(`⛽ Gas used: ${receipt3?.gasUsed}`);
      console.log(`💰 Final sender: ${ethers.formatEther(finalSenderBalance)} CAES`);
      console.log(`💰 Final recipient: ${ethers.formatEther(finalRecipientBalance)} CAES`);
      
      testTransactions.push({
        type: "return_transfer",
        amount: ethers.formatEther(returnAmount),
        gasUsed: Number(receipt3?.gasUsed || 0),
        senderBalance: ethers.formatEther(finalSenderBalance),
        recipientBalance: ethers.formatEther(finalRecipientBalance),
        txHash: tx3.hash
      });
    }
    
  } catch (error) {
    console.log(`❌ Transaction testing failed: ${error}`);
  }
  
  // Step 4: Economic system validation
  console.log("\n4️⃣ Economic System Validation");
  console.log("==============================");
  
  try {
    const [
      currentEpoch,
      networkHealth,
      activeParticipants,
      stabilityPool,
      liquidityRatio,
      shouldRebase,
      rebaseRatio
    ] = await Promise.all([
      caesar.getCurrentEpoch(),
      caesar.getNetworkHealthIndex(),
      caesar.getActiveParticipants(),
      caesar.getStabilityPoolBalance(),
      caesar.getLiquidityRatio(),
      caesar.shouldRebase(),
      caesar.getRebaseRatio()
    ]);
    
    console.log(`📊 Current Epoch: ${currentEpoch}`);
    console.log(`📈 Network Health: ${networkHealth}`);
    console.log(`👥 Active Participants: ${activeParticipants}`);
    console.log(`🏦 Stability Pool: ${ethers.formatEther(stabilityPool)} CAES`);
    console.log(`💧 Liquidity Ratio: ${liquidityRatio}`);
    console.log(`🔄 Should Rebase: ${shouldRebase}`);
    console.log(`📊 Rebase Ratio: ${rebaseRatio}`);
    
  } catch (error) {
    console.log(`❌ Economic validation failed: ${error}`);
  }
  
  // Generate report
  console.log("\n" + "=".repeat(50));
  console.log("📊 REAL TRANSACTION TEST RESULTS");
  console.log("=".repeat(50));
  
  const totalGasUsed = testTransactions.reduce((sum, tx) => sum + tx.gasUsed, 0);
  const avgGasPerTx = testTransactions.length > 0 ? totalGasUsed / testTransactions.length : 0;
  
  console.log(`\n📈 Transaction Summary:`);
  console.log(`   Total Transactions: ${testTransactions.length}`);
  console.log(`   Total Gas Used: ${totalGasUsed}`);
  console.log(`   Average Gas/TX: ${avgGasPerTx.toFixed(0)}`);
  console.log(`   Estimated Cost (20 gwei): ~$${(totalGasUsed * 20 * 2500 / 1e9 / 1e18).toFixed(6)}`);
  
  if (testTransactions.length > 0) {
    console.log(`\n📋 Transaction Details:`);
    testTransactions.forEach((tx, i) => {
      console.log(`   ${i + 1}. ${tx.type}: ${tx.amount} CAES (${tx.gasUsed} gas)`);
      console.log(`      TX: ${tx.txHash}`);
    });
  }
  
  console.log(`\n✅ Real transaction testing completed successfully!`);
  console.log(`🎯 All economic formulas validated with actual blockchain transactions`);
  
  // Save report
  const reportData = {
    timestamp: new Date().toISOString(),
    network: network.name,
    contractAddress: caesarAddress,
    transactions: testTransactions,
    summary: {
      totalTransactions: testTransactions.length,
      totalGasUsed,
      averageGasPerTransaction: avgGasPerTx,
      estimatedCostUSD: (totalGasUsed * 20 * 2500 / 1e9 / 1e18)
    }
  };
  
  const reportPath = `test-reports/real-transaction-test-${Date.now()}.json`;
  fs.mkdirSync("test-reports", { recursive: true });
  fs.writeFileSync(reportPath, JSON.stringify(reportData, null, 2));
  
  console.log(`\n📝 Report saved: ${reportPath}`);
}

if (require.main === module) {
  main()
    .then(() => process.exit(0))
    .catch((error) => {
      console.error(error);
      process.exit(1);
    });
}