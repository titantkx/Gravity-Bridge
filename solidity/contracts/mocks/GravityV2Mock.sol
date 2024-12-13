//SPDX-License-Identifier: Unlicense
pragma solidity ^0.8.10;

import { Initializable } from "@openzeppelin/contracts-upgradeable/proxy/utils/Initializable.sol";
import { UUPSUpgradeable } from "@openzeppelin/contracts-upgradeable/proxy/utils/UUPSUpgradeable.sol";
import { OwnableUpgradeable } from "@openzeppelin/contracts-upgradeable/access/OwnableUpgradeable.sol";
import { ReentrancyGuardUpgradeable } from "@openzeppelin/contracts-upgradeable/security/ReentrancyGuardUpgradeable.sol";
import "@openzeppelin/contracts/token/ERC20/IERC20.sol";
import "@openzeppelin/contracts/token/ERC20/utils/SafeERC20.sol";
import "@openzeppelin/contracts/utils/Address.sol";
import "@openzeppelin/contracts/utils/cryptography/ECDSA.sol";

import "../CosmosToken.sol";

import { MalformedCurrentValidatorSet, InsufficientPower, ValsetArgs } from "../Gravity.sol";

contract GravityV2Mock is
	Initializable,
	UUPSUpgradeable,
	OwnableUpgradeable,
	ReentrancyGuardUpgradeable
{
	using SafeERC20 for IERC20;

	string public constant VERSION = "1.0.0-t";

	// The number of 'votes' required to execute a valset
	// update or batch execution, set to 2/3 of 2^32
	uint256 constant constant_powerThreshold = 2863311530;

	// These are updated often
	bytes32 public state_lastValsetCheckpoint;
	mapping(address => uint256) public state_lastBatchNonces;
	mapping(bytes32 => uint256) public state_invalidationMapping;
	uint256 public state_lastValsetNonce;
	// event nonce zero is reserved by the Cosmos module as a special
	// value indicating that no events have yet been submitted
	uint256 public state_lastEventNonce;

	// This is set once at initialization
	bytes32 public state_gravityId;

	// contract address of tkx ERC20 token
	address public state_tkxContractAddress;

	// mapping of bech32 prefix to tkx exchange contract address at destination chain
	mapping(string => string) public state_prefixeToTkxExchangeContractAddress;

	// TransactionBatchExecutedEvent and SendToCosmosEvent both include the field _eventNonce.
	// This is incremented every time one of these events is emitted. It is checked by the
	// Cosmos module to ensure that all events are received in order, and that none are lost.
	//
	// ValsetUpdatedEvent does not include the field _eventNonce because it is never submitted to the Cosmos
	// module. It is purely for the use of relayers to allow them to successfully submit batches.
	event TransactionBatchExecutedEvent(
		uint256 indexed _batchNonce,
		address indexed _token,
		uint256 _eventNonce
	);
	event SendToCosmosEvent(
		address indexed _tokenContract,
		address indexed _sender,
		string _destination,
		uint256 _amount,
		uint256 _eventNonce,
		string _memo
	);
	event ERC20DeployedEvent(
		// FYI: Can't index on a string without doing a bunch of weird stuff
		string _cosmosDenom,
		address indexed _tokenContract,
		string _name,
		string _symbol,
		uint8 _decimals,
		uint256 _eventNonce
	);
	event ValsetUpdatedEvent(
		uint256 indexed _newValsetNonce,
		uint256 _eventNonce,
		uint256 _rewardAmount,
		address _rewardToken,
		address[] _validators,
		uint256[] _powers
	);
	event LogicCallEvent(
		bytes32 _invalidationId,
		uint256 _invalidationNonce,
		bytes _returnData,
		uint256 _eventNonce
	);

	// Make a new checkpoint from the supplied validator set
	// A checkpoint is a hash of all relevant information about the valset. This is stored by the contract,
	// instead of storing the information directly. This saves on storage and gas.
	// The format of the checkpoint is:
	// h(gravityId, "checkpoint", valsetNonce, validators[], powers[])
	// Where h is the keccak256 hash function.
	// The validator powers must be decreasing or equal. This is important for checking the signatures on the
	// next valset, since it allows the caller to stop verifying signatures once a quorum of signatures have been verified.
	function makeCheckpoint(
		ValsetArgs memory _valsetArgs,
		bytes32 _gravityId
	) private pure returns (bytes32) {
		// bytes32 encoding of the string "checkpoint"
		bytes32 methodName = 0x636865636b706f696e7400000000000000000000000000000000000000000000;

		bytes32 checkpoint = keccak256(
			abi.encode(
				_gravityId,
				methodName,
				_valsetArgs.valsetNonce,
				_valsetArgs.validators,
				_valsetArgs.powers,
				_valsetArgs.rewardAmount,
				_valsetArgs.rewardToken
			)
		);

		return checkpoint;
	}

	constructor() {}

	function _authorizeUpgrade(address newImplementation) internal override onlyOwner {}

	function initialize(
		// A unique identifier for this gravity instance to use in signatures
		bytes32 _gravityId,
		// The validator set, not in valset args format since many of it's
		// arguments would never be used in this case
		address[] memory _validators,
		uint256[] memory _powers
	) public initializer {
		__Ownable_init();

		state_lastValsetNonce = 0;
		// event nonce zero is reserved by the Cosmos module as a special
		// value indicating that no events have yet been submitted
		state_lastEventNonce = 1;

		// CHECKS

		// Check that validators, powers, and signatures (v,r,s) set is well-formed
		if (_validators.length != _powers.length || _validators.length == 0) {
			revert MalformedCurrentValidatorSet();
		}

		// Check cumulative power to ensure the contract has sufficient power to actually
		// pass a vote
		uint256 cumulativePower = 0;
		for (uint256 i = 0; i < _powers.length; i++) {
			cumulativePower = cumulativePower + _powers[i];
			if (cumulativePower > constant_powerThreshold) {
				break;
			}
		}
		if (cumulativePower <= constant_powerThreshold) {
			revert InsufficientPower({
				cumulativePower: cumulativePower,
				powerThreshold: constant_powerThreshold
			});
		}

		ValsetArgs memory _valset;
		_valset = ValsetArgs(_validators, _powers, 0, 0, address(0));

		bytes32 newCheckpoint = makeCheckpoint(_valset, _gravityId);

		// ACTIONS

		state_gravityId = _gravityId;
		state_lastValsetCheckpoint = newCheckpoint;

		// LOGS

		emit ValsetUpdatedEvent(
			state_lastValsetNonce,
			state_lastEventNonce,
			0,
			address(0),
			_validators,
			_powers
		);
	}
}
