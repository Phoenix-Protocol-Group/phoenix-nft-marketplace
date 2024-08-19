
# Collections

## Main functionality
This is a contract for managing collections of tokens, similar to the ERC-1155 standard. It allows for the creation, transfer, and management of multiple token types within a single contract.

## Messages:

`initialize`

Params:
- `admin`: `Address` of the administrator for the collection
- `name`: `String` name of the collection
- `symbol`: `String` symbol for the collection

Return type:
`Result<(), ContractError>`

Description:
Initializes the collection contract with an admin, name, and symbol.

<hr>

`balance_of`

Params:
- `account`: `Address` of the account to check
- `id`: `u64` ID of the token type

Return type:
`Result<u64, ContractError>`

Description:
Returns the balance of a specific token type for a given account.

<hr>

`balance_of_batch`

Params:
- `accounts`: `Vec<Address>` list of accounts to check
- `ids`: `Vec<u64>` list of token type IDs

Return type:
`Result<Vec<u64>, ContractError>`

Description:
Returns the balances of multiple token types for multiple accounts.

<hr>

`set_approval_for_all`

Params:
- `sender`: `Address` of the account granting approval
- `operator`: `Address` of the account being approved
- `approved`: `bool` approval status

Return type:
`Result<(), ContractError>`

Description:
Grants or revokes permission for an operator to manage the sender's tokens.

<hr>

`is_approved_for_all`

Params:
- `owner`: `Address` of the token owner
- `operator`: `Address` of the potential operator

Return type:
`Result<bool, ContractError>`

Description:
Checks if an operator is approved to manage an owner's tokens.

<hr>

`safe_transfer_from`

Params:
- `from`: `Address` of the sender
- `to`: `Address` of the recipient
- `id`: `u64` ID of the token type
- `transfer_amount`: `u64` amount to transfer

Return type:
`Result<(), ContractError>`

Description:
Transfers tokens of a specific type from one address to another.

<hr>

`safe_batch_transfer_from`

Params:
- `from`: `Address` of the sender
- `to`: `Address` of the recipient
- `ids`: `Vec<u64>` list of token type IDs
- `amounts`: `Vec<u64>` list of amounts to transfer

Return type:
`Result<(), ContractError>`

Description:
Transfers multiple types and amounts of tokens from one address to another.

<hr>

`mint`

Params:
- `sender`: `Address` of the minting authority
- `to`: `Address` of the recipient
- `id`: `u64` ID of the token type
- `amount`: `u64` amount to mint

Return type:
`Result<(), ContractError>`

Description:
Mints new tokens of a specific type to a recipient.

<hr>

`mint_batch`

Params:
- `sender`: `Address` of the minting authority
- `to`: `Address` of the recipient
- `ids`: `Vec<u64>` list of token type IDs
- `amounts`: `Vec<u64>` list of amounts to mint

Return type:
`Result<(), ContractError>`

Description:
Mints multiple types and amounts of tokens to a recipient.

<hr>

`burn`

Params:
- `from`: `Address` of the token holder
- `id`: `u64` ID of the token type
- `amount`: `u64` amount to burn

Return type:
`Result<(), ContractError>`

Description:
Burns (destroys) tokens of a specific type from an address.

<hr>

`burn_batch`

Params:
- `from`: `Address` of the token holder
- `ids`: `Vec<u64>` list of token type IDs
- `amounts`: `Vec<u64>` list of amounts to burn

Return type:
`Result<(), ContractError>`

Description:
Burns multiple types and amounts of tokens from an address.

<hr>

`set_uri`

Params:
- `sender`: `Address` of the authority
- `id`: `u64` ID of the token type
- `uri`: `Bytes` URI for the token type

Return type:
`Result<(), ContractError>`

Description:
Sets the URI for a specific token type.

<hr>

`set_collection_uri`

Params:
- `uri`: `Bytes` URI for the collection

Return type:
`Result<(), ContractError>`

Description:
Sets the main image (logo) URI for the entire collection.

<hr>

`uri`

Params:
- `id`: `u64` ID of the token type

Return type:
`Result<URIValue, ContractError>`

Description:
Retrieves the URI for a specific token type.

<hr>

`collection_uri`

Params:
None

Return type:
`Result<URIValue, ContractError>`

Description:
Retrieves the URI for the entire collection.

<hr>

`upgrade`

Params:
- `new_wasm_hash`: `BytesN<32>` hash of the new contract WASM

Return type:
`Result<(), ContractError>`

Description:
Upgrades the contract to a new WASM implementation.

<hr>

## Internal Structs

```rust
pub struct Config {
    pub name: String,
    pub symbol: String,
}

pub struct URIValue {
    pub uri: Bytes,
}

pub struct OperatorApprovalKey {
    pub owner: Address,
    pub operator: Address,
}
```
