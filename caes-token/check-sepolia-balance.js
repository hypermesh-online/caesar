const { ethers } = require("ethers");
require('dotenv').config();

async function main() {
  console.log("🔍 Checking Sepolia balances...\n");
  
  const provider = new ethers.JsonRpcProvider(process.env.ETH_TESTNET_RPC || "https://sepolia.infura.io/v3/269cd6aa735d4e7dbfa0deeef9fe3969");
  
  const addresses = [
    "0xfD33Cf15893DaC5a0ACFdE12f06DAC63a330b331", // Main deployer
    "0x08CF6C943C42d9cF56A73e47e095c33716c28595"  // Test wallet
  ];
  
  for (const address of addresses) {
    const balance = await provider.getBalance(address);
    const ethBalance = ethers.formatEther(balance);
    console.log(`${address}: ${ethBalance} ETH`);
  }
  
  console.log("\n💡 Need at least 0.1 ETH for deployment");
  console.log("🚰 Get more from: https://sepoliafaucet.com or https://faucet.sepolia.dev");
}

main().catch(console.error);