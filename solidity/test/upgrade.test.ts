import { SignerWithAddress } from "@nomiclabs/hardhat-ethers/signers";
import chai from "chai";
import { solidity } from "ethereum-waffle";
import { ethers } from "hardhat";
import { deployContracts, upgradeProxy } from "../test-utils";
import {
  examplePowers,
  getSignerAddresses,
  ZeroAddress
} from "../test-utils/pure";
import { Gravity } from "../typechain";

chai.use(solidity);
const { expect } = chai;

describe("Test Upgradeable function", function () {
  let contract: Gravity;
  let signers: SignerWithAddress[];
  let gravityId: string;
  let valset0: {
    powers: number[];
    validators: SignerWithAddress[];
    valsetNonce: number;
    rewardAmount: number;
    rewardToken: string;
  };

  beforeEach(async function () {
    // DEPLOY CONTRACTS
    // ================
    signers = await ethers.getSigners();
    gravityId = ethers.utils.formatBytes32String("foo");

    valset0 = {
      // This is the power distribution on the Cosmos hub as of 7/14/2020
      powers: examplePowers(),
      validators: signers.slice(0, examplePowers().length),
      valsetNonce: 0,
      rewardAmount: 0,
      rewardToken: ZeroAddress
    };

    const {
      gravity,
      testERC20,
      checkpoint: deployCheckpoint
    } = await deployContracts(gravityId, valset0.validators, valset0.powers);

    contract = gravity;
  });

  it("can not recall initialize", async function () {
    await expect(
      contract.initialize(
        gravityId,
        await getSignerAddresses(valset0.validators),
        valset0.powers
      )
    ).to.be.revertedWith("Initializable: contract is already initialized");
  });

  it("can upgrade", async function () {
    const newContract = await upgradeProxy(contract.address, "GravityV2Mock");
    expect(await newContract.VERSION()).to.be.equal("1.0.0-t");
  });

  it("not owner can not upgrade", async function () {
    await expect(upgradeProxy(contract.address, "GravityV2Mock", signers[1])).to
      .be.reverted;
  });
});
