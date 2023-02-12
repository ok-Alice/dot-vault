// compiler version must be greater than or equal to 0.8.13 and less than 0.9.0
pragma solidity ^0.8.13;

interface XVM {
    function xvm_call(
        bytes calldata context,
        bytes calldata to,
        bytes calldata input
    ) external;
}

contract collateral {
    XVM constant XVM_PRECOMPILE = XVM(0x0000000000000000000000000000000000005005);

    address ink_address;

    constructor (address _ink_address) {
        ink_address = _ink_address;
    }

 
    //  Allows a user to deposit an NFT as collateral to increase it's loan limit
    function deposit_nft(
        bytes20  evm_address,
        uint32  id
    ) public
    returns (Result)
    {
        bytes4 selector = 0x420e0f3f;
        bytes memory contract_address = abi.encodePacked(ink_address);
        bytes memory buffer = bytes.concat(
            selector,
            encode_bytes20(evm_address),
            encode_uint32(id)
        );

        XVM_PRECOMPILE.xvm_call("\x1f\x00", contract_address, buffer);
        return Result.Ok;
    }

 
    //  Allows a user to reclaim NFT to decrease it's loan balance (as long as open loan is smaller)
    function withdraw_nft(
        bytes20  evm_address,
        uint32  id
    ) public
    returns (Result)
    {
        bytes4 selector = 0x018d1025;
        bytes memory contract_address = abi.encodePacked(ink_address);
        bytes memory buffer = bytes.concat(
            selector,
            encode_bytes20(evm_address),
            encode_uint32(id)
        );

        XVM_PRECOMPILE.xvm_call("\x1f\x00", contract_address, buffer);
        return Result.Ok;
    }

 
    //  Allows admin to add NFT collection to list of allowed collections
    function register_nft_collection(
        bytes20  evm_address,
        uint32  risk_factor,
        uint32  collateral_factor
    ) public
    returns (Result)
    {
        bytes4 selector = 0x1cef56d1;
        bytes memory contract_address = abi.encodePacked(ink_address);
        bytes memory buffer = bytes.concat(
            selector,
            encode_bytes20(evm_address),
            encode_uint32(risk_factor),
            encode_uint32(collateral_factor)
        );

        XVM_PRECOMPILE.xvm_call("\x1f\x00", contract_address, buffer);
        return Result.Ok;
    }


 
    //  Allows user to request transfer of SCoin as long as loan limit allows it
    function take_loan(
        uint128  _amount
    ) public
    returns (Result)
    {
        bytes4 selector = 0xde369d16;
        bytes memory contract_address = abi.encodePacked(ink_address);
        bytes memory buffer = bytes.concat(
            selector,
            encode_uint128(_amount)
        );

        XVM_PRECOMPILE.xvm_call("\x1f\x00", contract_address, buffer);
        return Result.Ok;
    }

 
    //  Allows user to repay previously claimed loan
    function repay_loan(
        uint128  _amount
    ) public
    returns (Result)
    {
        bytes4 selector = 0x2a01c432;
        bytes memory contract_address = abi.encodePacked(ink_address);
        bytes memory buffer = bytes.concat(
            selector,
            encode_uint128(_amount)
        );

        XVM_PRECOMPILE.xvm_call("\x1f\x00", contract_address, buffer);
        return Result.Ok;
    }



 
    //  Caculates and updates open loan for given user
    //  new open loan = old open loan + (last loan change - now) * interest rate
    function updates_loan_balance(
        ink_env_types_AccountId memory _user
    ) public
    returns (Result)
    {
        bytes4 selector = 0x353f564e;
        bytes memory contract_address = abi.encodePacked(ink_address);
        bytes memory buffer = bytes.concat(
            selector,
            encode_ink_env_types_AccountId(_user)
        );

        XVM_PRECOMPILE.xvm_call("\x1f\x00", contract_address, buffer);
        return Result.Ok;
    }

 
    //  Leaves the contract without owner. It will not be possible to call
    //  owner's functions anymore. Can only be called by the current owner.
    // 
    //  NOTE: Renouncing ownership will leave the contract without an owner,
    //  thereby removing any functionality that is only available to the owner.
    // 
    //  On success a `OwnershipTransferred` event is emitted.
    // 
    //  # Errors
    // 
    //  Panics with `CallerIsNotOwner` error if caller is not owner
    function Ownable::renounce_ownership(
    ) public
    returns (Result)
    {
        bytes4 selector = 0x5e228753;
        bytes memory contract_address = abi.encodePacked(ink_address);
        bytes memory buffer = bytes.concat(
            selector
        );

        XVM_PRECOMPILE.xvm_call("\x1f\x00", contract_address, buffer);
        return Result.Ok;
    }

 
    //  Transfers ownership of the contract to a `new_owner`.
    //  Can only be called by the current owner.
    // 
    //  On success a `OwnershipTransferred` event is emitted.
    // 
    //  # Errors
    // 
    //  Panics with `CallerIsNotOwner` error if caller is not owner.
    // 
    //  Panics with `NewOwnerIsZero` error if new owner's address is zero.
    function Ownable::transfer_ownership(
        ink_env_types_AccountId memory new_owner
    ) public
    returns (Result)
    {
        bytes4 selector = 0x11f43efd;
        bytes memory contract_address = abi.encodePacked(ink_address);
        bytes memory buffer = bytes.concat(
            selector,
            encode_ink_env_types_AccountId(new_owner)
        );

        XVM_PRECOMPILE.xvm_call("\x1f\x00", contract_address, buffer);
        return Result.Ok;
    }





    // mapped id Number(1) -> bytes20


    // mapped id Number(2) -> uint8


    // mapped id Number(4) -> uint32


    // mapped id Number(6) -> bytes32


    // mapped id Number(8) -> ink_env_types_AccountId
    struct ink_env_types_AccountId {
        bytes32 f0;
    }


    function encode_ink_env_types_AccountId(ink_env_types_AccountId memory value) private pure returns (bytes memory) {
        return abi.encodePacked(
            value.f0
        );
    }

    // mapped id Number(10) -> uint128


    // mapped id Number(14) -> Result
    enum Result {
        Ok, // = 0
        Err // = 1
    }


    // mapped id Number(19) -> Result
    enum Result {
        Ok, // = 0
        Err // = 1
    }



    function encode_uint128(uint128 value) private pure returns (bytes memory) {
        return abi.encodePacked(value);
    }

}

