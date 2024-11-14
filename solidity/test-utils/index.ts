import { Signer } from "ethers";
import hre, { ethers, upgrades } from "hardhat";

import { Gravity } from "../typechain/Gravity";
import { TestERC20A } from "../typechain/TestERC20A";
import { getSignerAddresses, makeCheckpoint, ZeroAddress } from "./pure";

type DeployContractsOptions = {
  corruptSig?: boolean;
};

export async function deployContracts(
  gravityId = "foo",
  validators: Signer[],
  powers: number[],
  opts?: DeployContractsOptions
) {
  // enable automining for these tests
  await ethers.provider.send("evm_setAutomine", [true]);

  const TestERC20 = await ethers.getContractFactory("TestERC20A");
  const testERC20 = (await TestERC20.deploy()) as TestERC20A;

  const Gravity = await ethers.getContractFactory("Gravity");

  const valAddresses = await getSignerAddresses(validators);

  const checkpoint = makeCheckpoint(
    valAddresses,
    powers,
    0,
    0,
    ZeroAddress,
    gravityId
  );

  const gravity = (await hre.upgrades.deployProxy(
    Gravity,
    [gravityId, await getSignerAddresses(validators), powers],
    {
      kind: "uups",
      unsafeAllow: []
    }
  )) as Gravity;

  await gravity.deployed();

  return { gravity, testERC20, checkpoint };
}

export async function upgradeProxy(
  proxyAddress: string,
  contractName: string,
  caller?: Signer
): Promise<Gravity> {
  let Contract = await ethers.getContractFactory(contractName);
  if (caller) {
    Contract = Contract.connect(caller);
  }
  // upgrades.silenceWarnings();
  const contract = await upgrades.upgradeProxy(proxyAddress, Contract, {
    kind: "uups",
    unsafeAllow: []
  });

  await contract.deployed();
  return contract as Gravity;
}
