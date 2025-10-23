import { ethers } from "hardhat";

async function main() {
  console.log("🔍 Diagnosing CAESAR Token Deployment Issues");
  console.log("=".repeat(50));
  
  const caesarAddress = "0x69e35749eB8f5Ae282A883329C7d0BF44bCed59C";
  const Caesar = await ethers.getContractFactory("CaesarToken");
  const caesar = Caesar.attach(caesarAddress);
  
  console.log(`📍 Analyzing contract: ${caesarAddress}\n`);
  
  // Basic contract info
  console.log("1️⃣ Basic Contract Information");
  console.log("==============================");
  
  try {
    const [name, symbol, owner] = await Promise.all([
      caesar.name(),
      caesar.symbol(),
      caesar.owner()
    ]);
    
    console.log(`✅ Name: ${name}`);
    console.log(`✅ Symbol: ${symbol}`);
    console.log(`✅ Owner: ${owner}`);
    
  } catch (error) {
    console.log(`❌ Basic info failed: ${error}`);
    return;
  }
  
  // Check economic system components
  console.log("\n2️⃣ Economic System Components");
  console.log("==============================");
  
  try {
    console.log("🔍 Checking demurrage manager...");
    const demurrageManager = await caesar.demurrageManager();
    console.log(`📋 Demurrage Manager: ${demurrageManager}`);
    console.log(`✅ Active: ${demurrageManager !== ethers.ZeroAddress}`);
  } catch (error) {
    console.log(`❌ Demurrage manager check failed: ${error}`);
  }
  
  try {
    console.log("\n🔍 Checking anti-speculation engine...");
    const antiSpecEngine = await caesar.antiSpeculationEngine();
    console.log(`📋 Anti-Speculation Engine: ${antiSpecEngine}`);
    console.log(`✅ Active: ${antiSpecEngine !== ethers.ZeroAddress}`);
  } catch (error) {
    console.log(`❌ Anti-speculation engine check failed: ${error}`);
  }
  
  // Test individual economic functions
  console.log("\n3️⃣ Economic Function Testing");
  console.log("=============================");
  
  const [deployer] = await ethers.getSigners();
  
  try {
    console.log("🔍 Testing getCurrentEpoch...");
    const currentEpoch = await caesar.getCurrentEpoch();
    console.log(`✅ Current Epoch: ${currentEpoch}`);
  } catch (error) {
    console.log(`❌ getCurrentEpoch failed: ${error}`);
  }
  
  try {
    console.log("\n🔍 Testing calculateDemurrage...");
    const demurrage = await caesar.calculateDemurrage(deployer.address);
    console.log(`✅ Demurrage: ${ethers.formatEther(demurrage)} CAES`);
  } catch (error) {
    console.log(`❌ calculateDemurrage failed: ${error}`);
  }
  
  try {
    console.log("\n🔍 Testing getNetworkHealthIndex...");
    const healthIndex = await caesar.getNetworkHealthIndex();
    console.log(`✅ Health Index: ${healthIndex}`);
  } catch (error) {
    console.log(`❌ getNetworkHealthIndex failed: ${error}`);
  }
  
  try {
    console.log("\n🔍 Testing getActiveParticipants...");
    const participants = await caesar.getActiveParticipants();
    console.log(`✅ Active Participants: ${participants}`);
  } catch (error) {
    console.log(`❌ getActiveParticipants failed: ${error}`);
  }
  
  try {
    console.log("\n🔍 Testing getStabilityPoolBalance...");
    const stabilityPool = await caesar.getStabilityPoolBalance();
    console.log(`✅ Stability Pool: ${ethers.formatEther(stabilityPool)} CAES`);
  } catch (error) {
    console.log(`❌ getStabilityPoolBalance failed: ${error}`);
  }
  
  try {
    console.log("\n🔍 Testing getLiquidityRatio...");
    const liquidityRatio = await caesar.getLiquidityRatio();
    console.log(`✅ Liquidity Ratio: ${liquidityRatio}`);
  } catch (error) {
    console.log(`❌ getLiquidityRatio failed: ${error}`);
  }
  
  // Check if LayerZero functions work
  console.log("\n4️⃣ LayerZero Integration Testing");
  console.log("=================================");
  
  try {
    console.log("🔍 Testing endpoint...");
    const endpoint = await caesar.endpoint();
    console.log(`✅ LayerZero Endpoint: ${endpoint}`);
  } catch (error) {
    console.log(`❌ LayerZero endpoint failed: ${error}`);
  }
  
  // Check contract bytecode to see what's deployed
  console.log("\n5️⃣ Contract Deployment Verification");
  console.log("====================================");
  
  try {
    const code = await ethers.provider.getCode(caesarAddress);
    console.log(`✅ Contract has bytecode: ${code.length > 2 ? 'YES' : 'NO'}`);
    console.log(`📊 Bytecode size: ${(code.length - 2) / 2} bytes`);
    
    if (code === "0x") {
      console.log(`❌ ERROR: No contract deployed at ${caesarAddress}`);
    }
  } catch (error) {
    console.log(`❌ Bytecode check failed: ${error}`);
  }
  
  console.log("\n" + "=".repeat(50));
  console.log("🎯 DIAGNOSIS COMPLETE");
  console.log("=".repeat(50));
}

if (require.main === module) {
  main()
    .then(() => process.exit(0))
    .catch((error) => {
      console.error(error);
      process.exit(1);
    });
}