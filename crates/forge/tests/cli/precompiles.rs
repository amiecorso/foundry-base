//! Contains various tests for `forge test` with precompiles.

use foundry_evm_networks::{NetworkConfigs, base::upgrade::BaseUpgrade};
use foundry_test_utils::str;

forgetest_init!(precompile_trace_decoding, |prj, cmd| {
    prj.add_test(
        "PrecompileTrace.t.sol",
        r#"
// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;

import "forge-std/Test.sol";

contract PrecompileCaller {
    constructor() {
        // 0x01 - ECRECOVER
        {
            bytes32 hash = keccak256("test message");
            uint8 v = 27;
            bytes32 r = bytes32(uint256(1));
            bytes32 s = bytes32(uint256(2));
            address(0x01).staticcall(abi.encode(hash, v, r, s));
        }

        // 0x02 - SHA256
        address(0x02).staticcall(abi.encodePacked("hello"));

        // 0x03 - RIPEMD160
        address(0x03).staticcall(abi.encodePacked("hello"));

        // 0x04 - IDENTITY (datacopy)
        address(0x04).staticcall(abi.encodePacked("hello"));

        // 0x05 - MODEXP: compute 2^3 mod 5 = 3
        {
            bytes memory modexpInput = abi.encodePacked(
                uint256(1),  // base length
                uint256(1),  // exponent length
                uint256(1),  // modulus length
                uint8(2),    // base = 2
                uint8(3),    // exponent = 3
                uint8(5)     // modulus = 5
            );
            address(0x05).staticcall(modexpInput);
        }

        // 0x06 - BN254 ADD (ecadd): P + O = P
        {
            uint256 g1x = 1;
            uint256 g1y = 2;
            uint256 zerox = 0;
            uint256 zeroy = 0;
            address(0x06).staticcall(abi.encode(g1x, g1y, zerox, zeroy));
        }

        // 0x07 - BN254 MUL (ecmul): 1 * G = G
        {
            uint256 g1x = 1;
            uint256 g1y = 2;
            uint256 scalar = 1;
            address(0x07).staticcall(abi.encode(g1x, g1y, scalar));
        }

        // 0x08 - BN254 PAIRING: empty input returns success (1)
        address(0x08).staticcall("");

        // 0x09 - BLAKE2F
        {
            bytes memory blake2fInput = new bytes(213);
            blake2fInput[3] = 0x0c; // 12 rounds
            bytes8[8] memory iv = [
                bytes8(0x6a09e667f3bcc908),
                bytes8(0xbb67ae8584caa73b),
                bytes8(0x3c6ef372fe94f82b),
                bytes8(0xa54ff53a5f1d36f1),
                bytes8(0x510e527fade682d1),
                bytes8(0x9b05688c2b3e6c1f),
                bytes8(0x1f83d9abfb41bd6b),
                bytes8(0x5be0cd19137e2179)
            ];
            for (uint256 i = 0; i < 8; i++) {
                for (uint256 j = 0; j < 8; j++) {
                    blake2fInput[4 + i * 8 + j] = iv[i][j];
                }
            }
            blake2fInput[212] = 0x01;
            address(0x09).staticcall(blake2fInput);
        }

        // 0x0B - BLS12-381 G1 ADD (two points at infinity)
        address(0x0B).staticcall(new bytes(256));

        // 0x0C - BLS12-381 G1 MSM
        address(0x0C).staticcall(new bytes(160));

        // 0x0D - BLS12-381 G2 ADD (two points at infinity)
        address(0x0D).staticcall(new bytes(512));

        // 0x0E - BLS12-381 G2 MSM
        address(0x0E).staticcall(new bytes(288));

        // 0x0F - BLS12-381 PAIRING (G1 + G2 infinity points)
        address(0x0F).staticcall(new bytes(384));

        // 0x10 - BLS12-381 MAP FP TO G1
        address(0x10).staticcall(new bytes(64));

        // 0x11 - BLS12-381 MAP FP2 TO G2
        address(0x11).staticcall(new bytes(128));

        // 0x100 - P256VERIFY (secp256r1)
        address(0x100).staticcall(new bytes(160));
    }
}

contract PrecompileTraceTest is Test {
    function test_precompile_traces() public {
        new PrecompileCaller();
    }
}
   "#,
    );

    cmd.args(["test", "--mt", "test_precompile_traces", "-vvvv", "--evm-version", "osaka"])
        .assert_success()
        .stdout_eq(str![[r#"
...
Ran 1 test for test/PrecompileTrace.t.sol:PrecompileTraceTest
[PASS] test_precompile_traces() ([GAS])
Traces:
  [..] PrecompileTraceTest::test_precompile_traces()
    ├─ [..] → new PrecompileCaller@[..]
    │   ├─ [..] PRECOMPILES::ecrecover(0xea83cdcdd06bf61e414054115a551e23133711d0507dcbc07a4bab7dc4581935, 27, 1, 2) [staticcall]
    │   │   └─ ← [Return] 0xBe038042508C42Df7b2A529cd4Cc0a9447c7D2b6
    │   ├─ [..] PRECOMPILES::sha256(0x68656c6c6f) [staticcall]
    │   │   └─ ← [Return] 0x2cf24dba5fb0a30e26e83b2ac5b9e29e1b161e5c1fa7425e73043362938b9824
    │   ├─ [..] PRECOMPILES::ripemd(0x68656c6c6f) [staticcall]
    │   │   └─ ← [Return] 0x000000000000000000000000108f07b838241261
    │   ├─ [..] PRECOMPILES::identity(0x68656c6c6f) [staticcall]
    │   │   └─ ← [Return] 0x68656c6c6f
    │   ├─ [..] PRECOMPILES::modexp(1, 1, 1, 0x02, 0x03, 0x05) [staticcall]
    │   │   └─ ← [Return] 0x03
    │   ├─ [..] PRECOMPILES::ecadd(1, 2, 0, 0) [staticcall]
    │   │   └─ ← [Return] (1, 2)
    │   ├─ [..] PRECOMPILES::ecmul(1, 2, 1) [staticcall]
    │   │   └─ ← [Return] (1, 2)
    │   ├─ [..] PRECOMPILES::ecpairing() [staticcall]
    │   │   └─ ← [Return] true
    │   ├─ [..] PRECOMPILES::blake2f(12, [633244976228469098, 4298627039875721147, 3168446158426304060, 17381112106731261861, 15096882533739138641, 2264253069420660123, 7763433881832358687, 8728396173323133019], [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0], [0, 0], 1) [staticcall]
    │   │   └─ ← [Return] 0x1a48bfec594a1b13bb024be345656b8af895d662ccbc3f39fb5ecf2ef05942b5acace594cb81cdff6044b5bfaabfea105168676ce5753f6bb559ce3f92ad4850
    │   ├─ [..] PRECOMPILES::bls12G1Add(0x0000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000, 0x0000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000) [staticcall]
    │   │   └─ ← [Return] 0x0000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000
    │   ├─ [..] PRECOMPILES::bls12G1Msm(0x00000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000) [staticcall]
    │   │   └─ ← [Return] 0x0000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000
    │   ├─ [..] PRECOMPILES::bls12G2Add(0x00000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000, 0x00000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000) [staticcall]
    │   │   └─ ← [Return] 0x00000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000
    │   ├─ [..] PRECOMPILES::bls12G2Msm(0x000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000) [staticcall]
    │   │   └─ ← [Return] 0x00000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000
    │   ├─ [..] PRECOMPILES::bls12PairingCheck(0x000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000) [staticcall]
    │   │   └─ ← [Return] true
    │   ├─ [..] PRECOMPILES::bls12MapFpToG1(0x00000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000) [staticcall]
    │   │   └─ ← [Return] 0x0000000000000000000000000000000011a9a0372b8f332d5c30de9ad14e50372a73fa4c45d5f2fa5097f2d6fb93bcac592f2e1711ac43db0519870c7d0ea41500000000000000000000000000000000092c0f994164a0719f51c24ba3788de240ff926b55f58c445116e8bc6a47cd63392fd4e8e22bdf9feaa96ee773222133
    │   ├─ [..] PRECOMPILES::bls12MapFp2ToG2(0x0000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000) [staticcall]
    │   │   └─ ← [Return] 0x00000000000000000000000000000000018320896ec9eef9d5e619848dc29ce266f413d02dd31d9b9d44ec0c79cd61f18b075ddba6d7bd20b7ff27a4b324bfce000000000000000000000000000000000a67d12118b5a35bb02d2e86b3ebfa7e23410db93de39fb06d7025fa95e96ffa428a7a27c3ae4dd4b40bd251ac658892000000000000000000000000000000000260e03644d1a2c321256b3246bad2b895cad13890cbe6f85df55106a0d334604fb143c7a042d878006271865bc359410000000000000000000000000000000004c69777a43f0bda07679d5805e63f18cf4e0e7c6112ac7f70266d199b4f76ae27c6269a3ceebdae30806e9a76aadf5c
    │   ├─ [..] P256VERIFY::fulfillBasicOrder_efficient_6GL6yc() [staticcall]
    │   │   └─ ← [Return]
    │   └─ ← [Return] 62 bytes of code
    └─ ← [Stop]

Suite result: ok. 1 passed; 0 failed; 0 skipped; [ELAPSED]

Ran 1 test suite [ELAPSED]: 1 tests passed, 0 failed, 0 skipped (1 total tests)

"#]]);
});

// tests transfer using celo precompile.
// <https://github.com/foundry-rs/foundry/issues/11622>
forgetest_init!(celo_transfer, |prj, cmd| {
    prj.update_config(|config| {
        config.networks = NetworkConfigs::with_celo();
    });

    prj.add_test(
        "CeloTransfer.t.sol",
        r#"
import "forge-std/Test.sol";

interface IERC20 {
    function balanceOf(address account) external view returns (uint256);
    function transfer(address to, uint256 amount) external returns (bool);
}

contract CeloTransferTest is Test {
    IERC20 celo = IERC20(0x471EcE3750Da237f93B8E339c536989b8978a438);
    IERC20 usdc = IERC20(0xcebA9300f2b948710d2653dD7B07f33A8B32118C);
    IERC20 usdt = IERC20(0x48065fbBE25f71C9282ddf5e1cD6D6A887483D5e);

    address binanceAccount = 0xf6436829Cf96EA0f8BC49d300c536FCC4f84C4ED;
    address recipient = makeAddr("recipient");

    function setUp() public {
        vm.createSelectFork("https://forno.celo.org");
    }

    function testCeloBalance() external {
        console2.log("recipient balance before", celo.balanceOf(recipient));
        vm.prank(binanceAccount);
        celo.transfer(recipient, 100);
        console2.log("recipient balance after", celo.balanceOf(recipient));
        assertEq(celo.balanceOf(recipient), 100);
    }
}
   "#,
    );

    cmd.args(["test", "--mt", "testCeloBalance", "-vvv"]).assert_success().stdout_eq(str![[r#"
[COMPILING_FILES] with [SOLC_VERSION]
[SOLC_VERSION] [ELAPSED]
Compiler run successful!

Ran 1 test for test/CeloTransfer.t.sol:CeloTransferTest
[PASS] testCeloBalance() ([GAS])
Logs:
  recipient balance before 0
  recipient balance after 100

Suite result: ok. 1 passed; 0 failed; 0 skipped; [ELAPSED]

Ran 1 test suite [ELAPSED]: 1 tests passed, 0 failed, 0 skipped (1 total tests)

"#]]);
});

// Verifies the Base RIP-7212 P256VERIFY precompile is registered at 0x0100 and
// returns the expected `0x...01` success word for a known-valid signature.
forgetest_init!(base_p256verify, |prj, cmd| {
    prj.update_config(|config| {
        config.networks = NetworkConfigs::with_base();
    });

    prj.add_test(
        "BaseP256Verify.t.sol",
        r#"
import "forge-std/Test.sol";

contract BaseP256VerifyTest is Test {
    // Known-valid P256 test vector from
    // https://github.com/daimo-eth/p256-verifier/tree/master/test-vectors
    // (also exercised by revm-precompile::secp256r1::test_sig_verify::ok_1).
    bytes constant VALID_INPUT =
        hex"4cee90eb86eaa050036147a12d49004b6b9c72bd725d39d4785011fe190f0b4da73bd4903f0ce3b639bbbf6e8e80d16931ff4bcf5993d58468e8fb19086e8cac36dbcd03009df8c59286b162af3bd7fcc0450c9aa81be5d10d312af6c66b1d604aebd3099c618202fcfe16ae7770b0c49ab5eadf74b754204a3bb6060e44eff37618b065f9832de4ca6ca971a7a1adc826d0f7c00181a5fb2ddf79ae00b4e10e";

    function testBaseP256Verify() public view {
        (bool ok, bytes memory ret) = address(0x100).staticcall(VALID_INPUT);
        assertTrue(ok, "P256VERIFY staticcall reverted");
        assertEq(ret.length, 32, "P256VERIFY expected 32-byte output");
        bytes32 word;
        assembly {
            word := mload(add(ret, 32))
        }
        assertEq(word, bytes32(uint256(1)), "P256VERIFY expected success word");
    }
}
   "#,
    );

    cmd.args(["test", "--mt", "testBaseP256Verify", "-vvv"]).assert_success().stdout_eq(str![[r#"
[COMPILING_FILES] with [SOLC_VERSION]
[SOLC_VERSION] [ELAPSED]
Compiler run successful!

Ran 1 test for test/BaseP256Verify.t.sol:BaseP256VerifyTest
[PASS] testBaseP256Verify() ([GAS])
Suite result: ok. 1 passed; 0 failed; 0 skipped; [ELAPSED]

Ran 1 test suite [ELAPSED]: 1 tests passed, 0 failed, 0 skipped (1 total tests)

"#]]);
});

// Verifies the Jovian BLS12-381 pairing input cap is enforced: a staticcall
// with an oversized input must fail.
forgetest_init!(base_bls12_381_pairing_jovian_size_limit, |prj, cmd| {
    prj.update_config(|config| {
        config.networks = NetworkConfigs::with_base().with_base_hardfork(BaseUpgrade::Jovian);
    });

    prj.add_test(
        "BaseBls12381PairingJovianLimit.t.sol",
        r#"
import "forge-std/Test.sol";

contract BaseBls12381PairingJovianLimitTest is Test {
    // JOVIAN_BLS12_381_PAIRING_MAX = 156_672 bytes; one byte over must revert.
    uint256 constant JOVIAN_PAIRING_MAX = 156_672;
    address constant BLS12_381_PAIRING = address(0x0f);

    function testBaseBls12381PairingJovianRejectsOversizedInput() public view {
        bytes memory oversized = new bytes(JOVIAN_PAIRING_MAX + 1);
        (bool ok, ) = BLS12_381_PAIRING.staticcall(oversized);
        assertFalse(ok, "Jovian BLS12-381 pairing must reject oversized input");
    }
}
   "#,
    );

    cmd.args(["test", "--mt", "testBaseBls12381PairingJovianRejectsOversizedInput", "-vvv"])
        .assert_success()
        .stdout_eq(str![[r#"
[COMPILING_FILES] with [SOLC_VERSION]
[SOLC_VERSION] [ELAPSED]
Compiler run successful!

Ran 1 test for test/BaseBls12381PairingJovianLimit.t.sol:BaseBls12381PairingJovianLimitTest
[PASS] testBaseBls12381PairingJovianRejectsOversizedInput() ([GAS])
Suite result: ok. 1 passed; 0 failed; 0 skipped; [ELAPSED]

Ran 1 test suite [ELAPSED]: 1 tests passed, 0 failed, 0 skipped (1 total tests)

"#]]);
});

// End-to-end integration test for the Base app-layer TokenFactory precompile
// against a live vibenet fork (chain id 84_538_453 / 0x509f455).
//
// Vibenet currently has none of the app-layer features (TokenFactory,
// PolicyRegistry, ActivationRegistry features) flipped on, so the test
// activates `TOKEN_FACTORY` locally on the fork via vm.prank(ACTIVATION_ADMIN)
// before calling createToken. The end-state assertion is the deterministic
// address derivation defined in the Rust precompile
// (factory/variant.rs::TokenVariant::compute_address): the returned token
// address must equal `0xb2 || 9*0x00 || variant || decimals ||
// lower8(keccak256(abi.encode(caller, salt)))`.
//
// This test exercises the full vendored stack end-to-end:
//   - ActivationRegistry sstore (admin gate, set_activated path)
//   - TokenFactory dispatch (ABI decode, ensure_activated, decode_create_params)
//   - TokenVariant::compute_address (address derivation)
//   - EvmPrecompileStorageProvider::set_code (bytecode install)
//   - B20Token initialize (capability and supply-cap setup) and emit_event
//     for TokenCreated
//
// Requires network access to https://rpc.vibes.base.org/; skip in CI without
// outbound access. The cargo test name includes `integration` so it can be
// filtered.
forgetest_init!(base_token_factory_integration, |prj, cmd| {
    prj.update_config(|config| {
        config.networks = NetworkConfigs::with_base().with_base_hardfork(BaseUpgrade::Beryl);
    });

    prj.add_test(
        "BaseTokenFactoryIntegration.t.sol",
        r#"
import "forge-std/Test.sol";

interface IActivationRegistry {
    function isActivated(bytes32 feature) external view returns (bool);
    function admin() external view returns (address);
    function activate(bytes32 feature) external;
}

interface ITokenFactory {
    enum TokenVariant { NONE, DEFAULT, STABLECOIN, SECURITY }

    struct B20CreateParams {
        uint8 version;
        string name;
        string symbol;
        address initialAdmin;
        uint8 decimals;
    }

    function createToken(
        TokenVariant variant,
        bytes32 salt,
        bytes calldata params,
        bytes[] calldata initCalls
    ) external returns (address token);

    function isB20(address token) external view returns (bool);
    function getTokenVariant(address token) external view returns (TokenVariant);
}

contract BaseTokenFactoryIntegrationTest is Test {
    address constant TOKEN_FACTORY_ADDR = 0xb20F00000000000000000000000000000000000f;
    address constant ACTIVATION_REGISTRY_ADDR = 0x84530000000000000000000000000000000000ff;
    address constant ACTIVATION_ADMIN = 0xCB00000000000000000000000000000000000000;

    bytes32 constant FEATURE_TOKEN_FACTORY =
        0xceff857b4173841a3aef07ca52b183282fe74fe117e8f9dda0dcb3ddafd18a5b;

    IActivationRegistry constant ACTIVATION_REGISTRY = IActivationRegistry(ACTIVATION_REGISTRY_ADDR);
    ITokenFactory constant TOKEN_FACTORY = ITokenFactory(TOKEN_FACTORY_ADDR);

    function setUp() public {
        vm.createSelectFork("https://rpc.vibes.base.org/");
    }

    function _expectedTokenAddress(address creator, uint8 variant, uint8 decimals, bytes32 salt)
        internal
        pure
        returns (address)
    {
        bytes32 hash = keccak256(abi.encode(creator, salt));
        bytes memory addrBytes = new bytes(20);
        addrBytes[0] = 0xb2;
        addrBytes[10] = bytes1(variant);
        addrBytes[11] = bytes1(decimals);
        for (uint256 i = 0; i < 8; i++) {
            addrBytes[12 + i] = hash[i];
        }
        return address(bytes20(_bytesToBytes20(addrBytes)));
    }

    function _bytesToBytes20(bytes memory b) internal pure returns (bytes20 out) {
        require(b.length == 20, "len");
        assembly { out := mload(add(b, 32)) }
    }

    function test_TokenFactory_createToken_returnsExpectedAddress() public {
        assertEq(block.chainid, 84_538_453, "expected vibenet fork");
        assertEq(ACTIVATION_REGISTRY.admin(), ACTIVATION_ADMIN, "activation admin mismatch");

        if (!ACTIVATION_REGISTRY.isActivated(FEATURE_TOKEN_FACTORY)) {
            bytes memory activateData =
                abi.encodeCall(IActivationRegistry.activate, (FEATURE_TOKEN_FACTORY));
            vm.prank(ACTIVATION_ADMIN);
            (bool ok, bytes memory ret) = ACTIVATION_REGISTRY_ADDR.call(activateData);
            if (!ok) {
                console2.logBytes(ret);
                revert("activate reverted; see revert bytes above");
            }
        }
        assertTrue(
            ACTIVATION_REGISTRY.isActivated(FEATURE_TOKEN_FACTORY),
            "TokenFactory must be activated"
        );

        uint8 decimals = 18;
        bytes32 salt = keccak256("foundry-base.base_token_factory_integration");

        ITokenFactory.B20CreateParams memory params = ITokenFactory.B20CreateParams({
            version: 1,
            name: "Foundry-Base Test Token",
            symbol: "FBT",
            initialAdmin: address(this),
            decimals: decimals
        });

        bytes[] memory initCalls = new bytes[](0);

        address expected = _expectedTokenAddress(
            address(this),
            uint8(ITokenFactory.TokenVariant.DEFAULT),
            decimals,
            salt
        );

        address token = TOKEN_FACTORY.createToken(
            ITokenFactory.TokenVariant.DEFAULT,
            salt,
            abi.encode(params),
            initCalls
        );

        assertEq(token, expected, "deterministic address mismatch");
        assertTrue(TOKEN_FACTORY.isB20(token), "factory must recognize token");
        assertEq(
            uint256(TOKEN_FACTORY.getTokenVariant(token)),
            uint256(ITokenFactory.TokenVariant.DEFAULT),
            "variant mismatch"
        );
        assertGt(token.code.length, 0, "expected bytecode installed at token address");
    }
}
   "#,
    );

    cmd.args(["test", "--mt", "test_TokenFactory_createToken_returnsExpectedAddress", "-vvv"])
        .assert_success();
});
