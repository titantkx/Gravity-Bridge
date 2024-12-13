import { SignerWithAddress } from "@nomiclabs/hardhat-ethers/signers";
import chai from "chai";
import { solidity } from "ethereum-waffle";
import { ethers } from "hardhat";
import { deployContracts } from "../test-utils";
import { examplePowers, ZeroAddress } from "../test-utils/pure";
import { Gravity } from "../typechain";

chai.use(solidity);
const { expect } = chai;

describe("Test owner config function", function () {
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

  it("not owner can not setTkxContractAddress", async function () {
    await expect(
      contract.connect(signers[1]).setTkxContractAddress(ZeroAddress)
    ).to.be.revertedWith("Ownable: caller is not the owner");
  });

  it("not owner can not setPrefixToTkxExchangeContractAddress", async function () {
    await expect(
      contract
        .connect(signers[1])
        .setPrefixToTkxExchangeContractAddress(
          "titan",
          "titan14hj2tavq8fpesdwxxcu44rty3hh90vhujrvcmstl4zr3txmfvw9sws52xt"
        )
    ).to.be.revertedWith("Ownable: caller is not the owner");
  });

  it("owner can setTkxContractAddress", async function () {
    const tkxContractAddress = "0x0412C7c846bb6b7DC462CF6B453f76D8440b2609";
    await contract.setTkxContractAddress(tkxContractAddress);
    expect(await contract.state_tkxContractAddress()).to.eq(tkxContractAddress);
  });

  it("owner can not set zero address as tkxContractAddress", async function () {
    await expect(
      contract.setTkxContractAddress(ZeroAddress)
    ).to.be.revertedWith("invalid tkx contract address");
  });

  it("owner can setPrefixToTkxExchangeContractAddress", async function () {
    const prefix = "titan";
    const tkxExchangeContractAddress =
      "titan14hj2tavq8fpesdwxxcu44rty3hh90vhujrvcmstl4zr3txmfvw9sws52xt";
    await contract.setPrefixToTkxExchangeContractAddress(
      prefix,
      tkxExchangeContractAddress
    );
    const res = await contract.state_prefixeToTkxExchangeContractAddress(
      prefix
    );
    console.log(res);
  });
});
