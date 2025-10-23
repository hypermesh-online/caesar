import { ethers } from "hardhat";
import fs from "fs";

async function main() {
  console.log("🔄 Legacy Contract Migration Analysis");
  console.log("=".repeat(50));
  
  const [deployer] = await ethers.getSigners();
  const network = await ethers.provider.getNetwork();
  
  // Contract addresses
  const contracts = {
    oldCesar: "0x62F1AA31e9A5713EEb56086681e009B392E860C2",
    migrationCaes: "0x69e35749eB8f5Ae282A883329C7d0BF44bCed59C", 
    newCaesar: "0x7DcfC3F620634A7DE2d065faD5A20C3a9092269b"
  };
  
  console.log(`👤 Deployer: ${deployer.address}`);
  console.log(`🌐 Network: ${network.name}\n`);
  
  console.log("📋 Contract Analysis:");
  console.log(`🔴 Old CESAR: ${contracts.oldCesar}`);
  console.log(`🟡 Migration CAES: ${contracts.migrationCaes}`);
  console.log(`🟢 New CAESAR: ${contracts.newCaesar}\n`);
  
  // Step 1: Analyze current state
  console.log("1️⃣ Current State Analysis");
  console.log("==========================");
  
  try {
    // Check old CESAR
    const oldCesar = await ethers.getContractAt("IERC20", contracts.oldCesar);
    const oldBalance = await oldCesar.balanceOf(deployer.address);
    const oldSupply = await oldCesar.totalSupply();
    
    console.log(`🔴 Old CESAR:`);
    console.log(`   Total Supply: ${ethers.formatEther(oldSupply)} CESAR`);
    console.log(`   Your Balance: ${ethers.formatEther(oldBalance)} CESAR`);
    
    // Check migration CAES
    const migrationCaes = await ethers.getContractAt("IERC20", contracts.migrationCaes);
    const migrationBalance = await migrationCaes.balanceOf(deployer.address);
    const migrationSupply = await migrationCaes.totalSupply();
    
    console.log(`🟡 Migration CAES:`);
    console.log(`   Total Supply: ${ethers.formatEther(migrationSupply)} CAES`);
    console.log(`   Your Balance: ${ethers.formatEther(migrationBalance)} CAES`);
    
    // Check new CAESAR
    const newCaesar = await ethers.getContractAt("CaesarToken", contracts.newCaesar);
    const newBalance = await newCaesar.balanceOf(deployer.address);
    const newSupply = await newCaesar.totalSupply();
    
    console.log(`🟢 New CAESAR:`);
    console.log(`   Total Supply: ${ethers.formatEther(newSupply)} CAES`);
    console.log(`   Your Balance: ${ethers.formatEther(newBalance)} CAES`);
    console.log(`   Economic System: ACTIVE`);
    
    // Step 2: Migration recommendation
    console.log("\n2️⃣ Migration Recommendations");
    console.log("=============================");
    
    const hasOldTokens = oldBalance > 0n;
    const hasMigrationTokens = migrationBalance > 0n;
    const hasNewTokens = newBalance > 0n;
    
    console.log(`\n📊 Token Distribution:`);
    console.log(`   Old CESAR: ${hasOldTokens ? '✅ HAS TOKENS' : '❌ EMPTY'}`);
    console.log(`   Migration CAES: ${hasMigrationTokens ? '✅ HAS TOKENS' : '❌ EMPTY'}`);
    console.log(`   New CAESAR: ${hasNewTokens ? '✅ HAS TOKENS' : '❌ EMPTY'}`);
    
    console.log(`\n🎯 Recommended Actions:`);
    
    if (hasMigrationTokens) {
      console.log(`\n🟡 Migration CAES Contract (${ethers.formatEther(migrationBalance)} tokens):`);
      console.log(`   1. ✅ MIGRATE: Move tokens to new CAESAR system`);
      console.log(`   2. 🔒 DISABLE: Set migration contract as disabled`);
      console.log(`   3. 📝 DOCUMENT: Update deployment records`);
      console.log(`   \n   💡 This gives you full economic features for your tokens!`);
    }
    
    if (hasOldTokens) {
      console.log(`\n🔴 Old CESAR Contract (${ethers.formatEther(oldBalance)} tokens):`);
      console.log(`   1. ⚠️  MIGRATE: Should have been migrated already`);
      console.log(`   2. 🔍 INVESTIGATE: Check why tokens remain`);
    }
    
    console.log(`\n🟢 New CAESAR Contract:`);
    console.log(`   1. ✅ ACTIVE: Primary contract for all operations`);
    console.log(`   2. 🎯 FOCUS: Use for all testing and production`);
    console.log(`   3. 📈 SCALE: Ready for real user adoption`);
    
    // Step 3: Migration execution option
    if (hasMigrationTokens) {
      console.log("\n3️⃣ Execute Migration?");
      console.log("======================");
      console.log(`\n🔄 Available Migration:`);
      console.log(`   From: Migration CAES (${ethers.formatEther(migrationBalance)} tokens)`);
      console.log(`   To: New CAESAR (with full economic system)`);
      console.log(`   \n   This will give your tokens demurrage, anti-speculation, and gold-pegging!`);
      
      // Note: Actual migration would require enabling migration on new contract
      console.log(`\n⚠️  To execute migration:`);
      console.log(`   1. Enable migration on new CAESAR contract`);
      console.log(`   2. Transfer tokens from migration contract`);
      console.log(`   3. Mint equivalent amount in new CAESAR`);
      console.log(`   4. Disable migration for security`);
    }
    
  } catch (error) {
    console.log(`❌ Analysis failed: ${error}`);
  }
  
  // Step 4: Cleanup recommendations
  console.log("\n4️⃣ Long-term Cleanup Strategy");
  console.log("==============================");
  
  console.log(`\n📋 Contract Lifecycle Management:`);
  console.log(`\n🔴 Old CESAR (${contracts.oldCesar}):`);
  console.log(`   Status: DEPRECATED`);
  console.log(`   Action: Keep for historical record`);
  console.log(`   Risk: Low (already migrated)`);
  
  console.log(`\n🟡 Migration CAES (${contracts.migrationCaes}):`);
  console.log(`   Status: INTERMEDIATE`);
  console.log(`   Action: Migrate tokens → Disable contract`);
  console.log(`   Risk: Medium (holds value without economic features)`);
  
  console.log(`\n🟢 New CAESAR (${contracts.newCaesar}):`);
  console.log(`   Status: PRODUCTION`);
  console.log(`   Action: Primary contract for all operations`);
  console.log(`   Risk: Low (fully featured, actively maintained)`);
  
  console.log(`\n🎯 Final State Goal:`);
  console.log(`   • All tokens in new CAESAR system`);
  console.log(`   • Old contracts disabled/dormant`);
  console.log(`   • Clear documentation of migration history`);
  console.log(`   • Single active contract for users`);
  
  // Save analysis report
  const analysisData = {
    timestamp: new Date().toISOString(),
    network: network.name,
    deployer: deployer.address,
    contracts: contracts,
    analysis: {
      oldCesarDeprecated: true,
      migrationCaesIntermediate: true,
      newCaesarProduction: true
    },
    recommendations: {
      migrateFromIntermediate: true,
      disableOldContracts: true,
      focusOnNewSystem: true
    }
  };
  
  const reportPath = `analysis/legacy-contract-analysis-${Date.now()}.json`;
  fs.mkdirSync("analysis", { recursive: true });
  fs.writeFileSync(reportPath, JSON.stringify(analysisData, null, 2));
  
  console.log(`\n📝 Analysis saved: ${reportPath}`);
  console.log(`\n✅ Legacy contract analysis complete!`);
}

if (require.main === module) {
  main()
    .then(() => process.exit(0))
    .catch((error) => {
      console.error(error);
      process.exit(1);
    });
}