import { ethers } from "hardhat";

async function main() {
  const [deployer] = await ethers.getSigners();
  console.log("Address:", deployer.address);
  console.log("Private Key:", deployer.privateKey);
}

main().catch(console.error);