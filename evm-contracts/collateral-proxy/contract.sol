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

 
    function deposit(
        openbrush_contracts_traits_types_Id  id
    ) public
    returns (Result)
    {
        bytes4 selector = 0x2d10c9bd;
        bytes memory contract_address = abi.encodePacked(ink_address);
        bytes memory buffer = bytes.concat(
            selector,
            encode_openbrush_contracts_traits_types_Id(id)
        );

        XVM_PRECOMPILE.xvm_call("\x1f\x00", contract_address, buffer);
        return Result.Ok;
    }




    // mapped id Number(2) -> openbrush_contracts_traits_types_Id
    enum openbrush_contracts_traits_types_Id {
        U8, // = 0
        U16, // = 1
        U32, // = 2
        U64, // = 3
        U128, // = 4
        Bytes // = 5
    }


    // mapped id Number(8) -> Result
    enum Result {
        Ok, // = 0
        Err // = 1
    }



    function encode_uint128(uint128 value) private pure returns (bytes memory) {
        return abi.encodePacked(value);
    }

}

