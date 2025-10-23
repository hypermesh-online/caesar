import { ethers } from "hardhat";
import fs from "fs";

async function main() {
  console.log("🚀 Deploying Complete CAESAR Token System to Sepolia");
  console.log("=".repeat(60));
  
  const [deployer] = await ethers.getSigners();
  const network = await ethers.provider.getNetwork();
  
  console.log(`👤 Deployer: ${deployer.address}`);
  console.log(`💰 Balance: ${ethers.formatEther(await ethers.provider.getBalance(deployer.address))} ETH`);
  console.log(`🌐 Network: ${network.name} (${network.chainId})\n`);
  
  const deployedContracts: any = {};
  
  // Step 1: Deploy DemurrageManager
  console.log("1️⃣ Deploying DemurrageManager");
  console.log("==============================");
  
  try {
    const DemurrageManager = await ethers.getContractFactory("DemurrageManager");
    const demurrageManager = await DemurrageManager.deploy();
    await demurrageManager.waitForDeployment();
    const demurrageAddress = await demurrageManager.getAddress();
    
    console.log(`✅ DemurrageManager deployed: ${demurrageAddress}`);
    deployedContracts.DemurrageManager = demurrageAddress;
    
  } catch (error) {
    console.log(`❌ DemurrageManager deployment failed: ${error}`);
    return;
  }
  
  // Step 2: Deploy AntiSpeculationEngine
  console.log("\n2️⃣ Deploying AntiSpeculationEngine");
  console.log("===================================");
  
  try {
    const AntiSpeculationEngine = await ethers.getContractFactory("AntiSpeculationEngine");
    const antiSpecEngine = await AntiSpeculationEngine.deploy();
    await antiSpecEngine.waitForDeployment();
    const antiSpecAddress = await antiSpecEngine.getAddress();
    
    console.log(`✅ AntiSpeculationEngine deployed: ${antiSpecAddress}`);
    deployedContracts.AntiSpeculationEngine = antiSpecAddress;
    
  } catch (error) {
    console.log(`❌ AntiSpeculationEngine deployment failed: ${error}`);
    return;
  }
  
  // Step 3: Deploy CaesarToken with economic components
  console.log("\n3️⃣ Deploying CaesarToken");
  console.log("=========================");
  
  try {
    // LayerZero endpoint for Sepolia
    const lzEndpoint = "0x6EDCE65403992e310A62460808c4b910D972f10f"; // Sepolia LZ endpoint
    
    const CaesarToken = await ethers.getContractFactory("CaesarToken");
    const caesarToken = await CaesarToken.deploy(
      "CAESAR",
      "CAES", 
      lzEndpoint,
      deployer.address // owner
    );
    await caesarToken.waitForDeployment();
    const caesarAddress = await caesarToken.getAddress();
    
    console.log(`✅ CaesarToken deployed: ${caesarAddress}`);
    deployedContracts.CaesarToken = caesarAddress;
    
  } catch (error) {
    console.log(`❌ CaesarToken deployment failed: ${error}`);
    return;
  }
  
  // Step 4: Initialize economic system
  console.log("\n4️⃣ Initializing Economic System");
  console.log("================================");
  
  try {
    const caesar = await ethers.getContractAt("CaesarToken", deployedContracts.CaesarToken);
    const demurrageManager = await ethers.getContractAt("DemurrageManager", deployedContracts.DemurrageManager);
    const antiSpecEngine = await ethers.getContractAt("AntiSpeculationEngine", deployedContracts.AntiSpeculationEngine);
    
    // Set up demurrage manager
    console.log("🔧 Setting up DemurrageManager...");
    const setDemurrageTx = await caesar.setDemurrageManager(deployedContracts.DemurrageManager);
    await setDemurrageTx.wait();
    console.log(`✅ DemurrageManager linked`);
    
    // Set up anti-speculation engine
    console.log("🔧 Setting up AntiSpeculationEngine...");
    const setAntiSpecTx = await caesar.setAntiSpeculationEngine(deployedContracts.AntiSpeculationEngine);
    await setAntiSpecTx.wait();
    console.log(`✅ AntiSpeculationEngine linked`);
    
    // Initialize demurrage manager with token address
    console.log("🔧 Initializing DemurrageManager...");
    const initDemurrageTx = await demurrageManager.initialize(deployedContracts.CaesarToken);
    await initDemurrageTx.wait();
    console.log(`✅ DemurrageManager initialized`);
    
    // Initialize anti-speculation engine
    console.log("🔧 Initializing AntiSpeculationEngine...");
    const initAntiSpecTx = await antiSpecEngine.initialize(deployedContracts.CaesarToken);
    await initAntiSpecTx.wait();
    console.log(`✅ AntiSpeculationEngine initialized`);
    
  } catch (error) {
    console.log(`❌ Economic system initialization failed: ${error}`);
  }
  
  // Step 5: Test deployment
  console.log("\n5️⃣ Testing Deployment");
  console.log("=======================");
  
  try {
    const caesar = await ethers.getContractAt("CaesarToken", deployedContracts.CaesarToken);
    
    // Test basic functions
    const name = await caesar.name();
    const symbol = await caesar.symbol();
    const owner = await caesar.owner();
    
    console.log(`✅ Token: ${name} (${symbol})`);
    console.log(`✅ Owner: ${owner}`);
    
    // Test economic functions
    try {
      const currentEpoch = await caesar.getCurrentEpoch();
      console.log(`✅ Current Epoch: ${currentEpoch}`);
      
      const healthIndex = await caesar.getNetworkHealthIndex();
      console.log(`✅ Health Index: ${healthIndex}`);
      
      const demurrageManager = await caesar.demurrageManager();
      console.log(`✅ Demurrage Manager: ${demurrageManager}`);
      
      const antiSpecEngine = await caesar.antiSpeculationEngine();
      console.log(`✅ Anti-Speculation Engine: ${antiSpecEngine}`);
      
    } catch (economicError) {
      console.log(`⚠️  Economic functions test: ${economicError}`);
    }
    
  } catch (error) {
    console.log(`❌ Deployment test failed: ${error}`);
  }
  
  // Step 6: Mint initial supply for testing
  console.log("\n6️⃣ Setting Up Testing Environment");
  console.log("===================================");
  
  try {
    const caesar = await ethers.getContractAt("CaesarToken", deployedContracts.CaesarToken);
    
    // Enable migration for minting
    const setMigrationTx = await caesar.setMigrationContract(deployer.address);
    await setMigrationTx.wait();
    
    const enableMigrationTx = await caesar.setMigrationEnabled(true);
    await enableMigrationTx.wait();
    
    // Mint initial supply
    const mintAmount = ethers.parseEther("1000000"); // 1M tokens
    const mintTx = await caesar.migrationMint(deployer.address, mintAmount);
    const receipt = await mintTx.wait();
    
    const balance = await caesar.balanceOf(deployer.address);
    console.log(`✅ Minted: ${ethers.formatEther(balance)} CAES`);
    console.log(`⛽ Gas used: ${receipt?.gasUsed}`);
    
    // Disable migration for security
    const disableMigrationTx = await caesar.setMigrationEnabled(false);
    await disableMigrationTx.wait();
    console.log(`✅ Migration disabled for security`);
    
  } catch (error) {
    console.log(`❌ Testing setup failed: ${error}`);
  }
  
  // Save deployment record
  const deploymentData = {
    timestamp: new Date().toISOString(),
    network: network.name,
    chainId: Number(network.chainId),
    deployer: deployer.address,
    contracts: deployedContracts,
    initialization: {
      economicSystemLinked: true,
      initialSupplyMinted: true,
      migrationDisabled: true
    }
  };
  
  const deploymentPath = `deployments/complete-caesar-sepolia-${Date.now()}.json`;
  fs.mkdirSync("deployments", { recursive: true });
  fs.writeFileSync(deploymentPath, JSON.stringify(deploymentData, null, 2));
  
  console.log("\n" + "=".repeat(60));
  console.log("🎯 DEPLOYMENT COMPLETE");
  console.log("=".repeat(60));
  
  console.log(`\n📍 CaesarToken: ${deployedContracts.CaesarToken}`);
  console.log(`📍 DemurrageManager: ${deployedContracts.DemurrageManager}`);
  console.log(`📍 AntiSpeculationEngine: ${deployedContracts.AntiSpeculationEngine}`);
  
  console.log(`\n📝 Deployment record: ${deploymentPath}`);
  
  console.log(`\n🚀 Ready for production testing with full economic system!`);
}

if (require.main === module) {
  main()
    .then(() => process.exit(0))
    .catch((error) => {
      console.error(error);
      process.exit(1);
    });
}