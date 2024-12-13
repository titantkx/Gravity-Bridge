//SPDX-License-Identifier: Apache-2.0
pragma solidity ^0.8.10;

library Bech32 {
	bytes constant CHARSET = "qpzry9x8gf2tvdw0s3jn54khce6mua7l";

	// bytes constant CHARSET_REV = [
	// 	15, // '0' 0f 0
	// 	255, // '1' ff 1 do not have
	// 	10, // '2' 0a 2
	// 	17, // '3' 11 3
	// 	21, // '4' 15 4
	// 	20, // '5' 14 5
	// 	26, // '6' 1a 6
	// 	30, // '7' 1e 7
	// 	7, // '8' 07 8
	// 	5, // '9' 05 9
	// 	29, // 'a' 1d 10
	// 	255, // 'b' ff   do not have
	// 	24, // 'c' 18
	// 	13, // 'd' 0d
	// 	25, // 'e' 19
	// 	9, // 'f' 09
	// 	8, // 'g' 08
	// 	23, // 'h' 17
	// 	255, // 'i' ff do not have
	// 	18, // 'j' 12
	// 	22, // 'k' 16
	// 	31, // 'l' 1f
	// 	27, // 'm' 1b
	// 	19, // 'n' 13
	// 	255, // 'o' ff do not have
	// 	1, // 'p' 01
	// 	0, // 'q' 00
	// 	3, // 'r' 03
	// 	16, // 's' 10
	// 	11, // 't' 0b
	// 	28, // 'u' 1c
	// 	12, // 'v' 0c
	// 	14, // 'w' 0e
	// 	6, // 'x' 06
	// 	4 // 'y' 04
	// 	2 // 'z' 02
	// ];

	bytes constant CHARSET_REV =
		hex"0fff0a1115141a1e07051dff180d19090817ff12161f1b13ff010003100b1c0c0e060402";

	function bytes1ToString(bytes1 _byte) public pure returns (string memory) {
		bytes memory byteArray = new bytes(1);
		byteArray[0] = _byte;
		return string(byteArray);
	}

	///
	/// @param data The string data to decode . array of 8-bit values.
	/// @return ret The decoded data array of 5-bit values.
	function decodeStringData(bytes memory data) internal pure returns (uint[] memory ret) {
		ret = new uint[](data.length);
		for (uint i = 0; i < data.length; i++) {
			if (data[i] >= "a" && data[i] <= "z") {
				uint idx = uint8(data[i]) - 0x57;
				ret[i] = uint8(CHARSET_REV[idx]);
				require(ret[i] != 255, "Invalid character");
			} else if (data[i] >= "0" && data[i] <= "9") {
				uint idx = uint8(data[i]) - 0x30;
				ret[i] = uint8(CHARSET_REV[idx]);
				require(ret[i] != 255, "Invalid character");
			} else {
				revert("Invalid character");
			}
		}
		return ret;
	}

	///
	/// @param values The values to polymod. Array of 5-bit values.
	/// @return The result of the polymod operation. 30-bit value.
	function polymod(uint[] memory values) internal pure returns (uint) {
		uint32[5] memory GENERATOR = [0x3b6a57b2, 0x26508e6d, 0x1ea119fa, 0x3d4233dd, 0x2a1462b3];
		uint chk = 1;
		for (uint p = 0; p < values.length; p++) {
			uint top = chk >> 25;
			chk = ((chk & 0x1ffffff) << 5) ^ values[p];
			for (uint i = 0; i < 5; i++) {
				if ((top >> i) & 1 == 1) {
					chk ^= GENERATOR[i];
				}
			}
		}
		return chk;
	}

	///
	/// @param hrp Human readable part. array of 8-bit values.
	/// return array of 5-bit values. [hrp high bits, 0, hrp 5 low bits]
	function hrpExpand(uint[] memory hrp) internal pure returns (uint[] memory) {
		uint[] memory ret = new uint[](hrp.length + hrp.length + 1);
		for (uint p = 0; p < hrp.length; p++) {
			ret[p] = hrp[p] >> 5;
		}
		ret[hrp.length] = 0;
		for (uint p = 0; p < hrp.length; p++) {
			ret[p + hrp.length + 1] = hrp[p] & 31;
		}
		return ret;
	}

	function concat(uint[] memory left, uint[] memory right) internal pure returns (uint[] memory) {
		uint[] memory ret = new uint[](left.length + right.length);

		uint i = 0;
		for (; i < left.length; i++) {
			ret[i] = left[i];
		}

		uint j = 0;
		while (j < right.length) {
			ret[i++] = right[j++];
		}

		return ret;
	}

	/// @dev Extend an array by a number of elements with a given value
	/// @param array  The array to extend
	/// @param val  The value to extend the array with
	/// @param num  The number of times to extend the array
	function extend(uint[] memory array, uint val, uint num) internal pure returns (uint[] memory) {
		uint[] memory ret = new uint[](array.length + num);

		uint i = 0;
		for (; i < array.length; i++) {
			ret[i] = array[i];
		}

		uint j = 0;
		while (j < num) {
			ret[i++] = val;
			j++;
		}

		return ret;
	}

	///
	/// @param hrp  Human readable part. array of 8-bit values.
	/// @param data  Data part. array of 5-bit values.
	/// @return The checksum. Array of 5-bit values.
	function createChecksum(
		uint[] memory hrp,
		uint[] memory data
	) internal pure returns (uint[] memory) {
		uint[] memory values = extend(concat(hrpExpand(hrp), data), 0, 6);
		//last value to xor after polymod is `bech32` => `1` ; `bech32m` => `0x2bc830a3`
		uint mod = polymod(values) ^ 1;
		uint[] memory ret = new uint[](6);
		for (uint p = 0; p < 6; p++) {
			ret[p] = (mod >> (5 * (5 - p))) & 31;
		}
		return ret;
	}

	///
	/// @param hrp Human readable part. array of 8-bit values.
	/// @param data  Data part. array of 5-bit values.
	/// @return The bech32 encoded string of data and checksum.
	function encode(uint[] memory hrp, uint[] memory data) internal pure returns (bytes memory) {
		uint[] memory combined = concat(data, createChecksum(hrp, data));

		bytes memory ret = new bytes(combined.length);
		for (uint p = 0; p < combined.length; p++) {
			ret[p] = CHARSET[combined[p]];
		}

		return ret;
	}

	/// @dev Convert data to a different bit length
	/// @param data  The data to convert
	/// @param inBits  The number of bits per input value
	/// @param outBits  The number of bits per output value
	function convert(
		uint[] memory data,
		uint inBits,
		uint outBits
	) internal pure returns (uint[] memory) {
		uint value = 0;
		uint bits = 0;
		uint maxV = (1 << outBits) - 1;

		uint[] memory ret = new uint[](32);
		uint j = 0;
		for (uint i = 0; i < data.length; ++i) {
			value = (value << inBits) | data[i];
			bits += inBits;

			while (bits >= outBits) {
				bits -= outBits;
				ret[j] = (value >> bits) & maxV;
				j += 1;
			}
		}

		return ret;
	}

	// high level functions
	//
	//

	function encodeToString(
		string memory hrp,
		uint[] memory data
	) internal pure returns (string memory) {
		uint[] memory words = convert(data, 8, 5);

		uint[] memory version = new uint[](1);
		version[0] = 0;

		bytes memory hrpBytes = bytes(hrp);
		uint[] memory hrpUInts = new uint[](hrpBytes.length);
		for (uint i = 0; i < hrpBytes.length; i++) {
			hrpUInts[i] = uint(uint8(hrpBytes[i]));
		}

		bytes memory dataAndChecksum = encode(hrpUInts, concat(version, words));

		return string(abi.encodePacked(hrp, dataAndChecksum));
	}

	///
	/// @param bech32Bytes  The bech32 encoded string
	/// @return hrp : hrp string . array of 8-bit values.
	/// @return data : data encoded string . array of 8-bit values.
	/// @return checksum : data encoded string . array of 8-bit values.
	function splitBech32(
		bytes memory bech32Bytes
	) internal pure returns (bytes memory hrp, bytes memory data, bytes memory checksum) {
		require(bech32Bytes.length >= 8, "bech32String is too short");

		// only accept 0-9 A-Z and a-z
		for (uint i = 0; i < bech32Bytes.length; i++) {
			require(
				(bech32Bytes[i] >= "0" && bech32Bytes[i] <= "9") ||
					(bech32Bytes[i] >= "A" && bech32Bytes[i] <= "Z") ||
					(bech32Bytes[i] >= "a" && bech32Bytes[i] <= "z"),
				"bech32String contains invalid characters"
			);
			// convert to lower case
			if (bech32Bytes[i] >= "A" && bech32Bytes[i] <= "Z") {
				bech32Bytes[i] = bytes1(uint8(bech32Bytes[i]) + 32);
			}
		}

		// Find the first '1' character
		uint i1 = 0;
		for (; i1 < bech32Bytes.length; i1++) {
			if (bech32Bytes[i1] == "1") {
				break;
			}
		}

		require(i1 < bech32Bytes.length, "bech32String does not contain '1' character");
		require(i1 >= 1, "HRP must be at least 1 character long");
		require(i1 + 7 < bech32Bytes.length, "bech32String is too short");

		// Split the HRP and data  `[HRP]1[data][checksum:6]`

		bytes memory hrpBytes = new bytes(i1);
		for (uint j = 0; j < i1; j++) {
			hrpBytes[j] = bech32Bytes[j];
		}
		hrp = hrpBytes;

		bytes memory dataBytes = new bytes(bech32Bytes.length - i1 - 1 - 6);
		for (uint j = 0; j < dataBytes.length; j++) {
			dataBytes[j] = bech32Bytes[i1 + 1 + j];
		}
		data = dataBytes;

		bytes memory checksumBytes = new bytes(6);
		for (uint j = 0; j < 6; j++) {
			checksumBytes[j] = bech32Bytes[bech32Bytes.length - 6 + j];
		}
		checksum = checksumBytes;

		return (hrp, data, checksum);
	}

	///
	/// @param hrp : hrp string . array of 8-bit values.
	/// @param data : data encoded string . array of 8-bit values.
	/// @param checksum : data encoded string . array of 8-bit values.
	function isValid(
		bytes memory hrp,
		bytes memory data,
		bytes memory checksum
	) internal pure returns (bool) {
		uint[] memory dataDecoded = decodeStringData(data);
		uint[] memory checksumDecoded = decodeStringData(checksum);

		bytes memory hrpBytes = bytes(hrp);
		uint[] memory hrpUInts = new uint[](hrpBytes.length);
		for (uint i = 0; i < hrpBytes.length; i++) {
			hrpUInts[i] = uint(uint8(hrpBytes[i]));
		}

		uint[] memory calculatedChecksum = createChecksum(hrpUInts, dataDecoded);

		for (uint i = 0; i < 6; i++) {
			if (calculatedChecksum[i] != checksumDecoded[i]) {
				return false;
			}
		}

		return true;
	}
}
