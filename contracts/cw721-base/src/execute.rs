use cw_ownable::OwnershipError;
use serde::de::DeserializeOwned;
use serde::Serialize;

use cosmwasm_std::{
    Addr, Api, BankMsg, Binary, Coin, CustomMsg, Deps, DepsMut, Env, MessageInfo, Response,
    Storage, Uint128,
};

use cw721::{ContractInfoResponse, Cw721Execute, Cw721ReceiveMsg, Expiration};

use crate::error::ContractError;
use crate::msg::{ExecuteMsg, InstantiateMsg, SaleConfigResponse};
use crate::state::{Approval, Cw721Contract, TokenInfo};

impl<'a, T, C, E, Q> Cw721Contract<'a, T, C, E, Q>
where
    T: Serialize + DeserializeOwned + Clone,
    C: CustomMsg,
    E: CustomMsg,
    Q: CustomMsg,
{
    pub fn instantiate(
        &self,
        deps: DepsMut,
        _env: Env,
        info: MessageInfo,
        msg: InstantiateMsg,
    ) -> Result<Response<C>, ContractError> {
        let contract_info = ContractInfoResponse {
            name: msg.name,
            symbol: msg.symbol,
        };
        self.contract_info.save(deps.storage, &contract_info)?;

        // Set owner and withdraw address as the sender instantiating the contract
        let owner = info.sender;
        cw_ownable::initialize_owner(deps.storage, deps.api, Some(owner.as_ref()))?;
        self.set_withdraw_address(deps.storage, deps.api, &owner, owner.to_string())?;

        // Set base token uri
        self.base_token_uri
            .save(deps.storage, &msg.base_token_uri)?;

        // Set collection size
        self.collection_size
            .save(deps.storage, &msg.collection_size)?;

        // Set sale config
        let sale_config = SaleConfigResponse {
            max_per_public: msg.max_per_public,
            max_per_allowlist: msg.max_per_allowlist,
            public_price: msg.public_price,
            allowlist_price: msg.allowlist_price,
            public_sale_open: false,
            allowlist_sale_open: false,
        };
        self.sale_config.save(deps.storage, &sale_config)?;
        Ok(Response::default())
    }

    pub fn execute(
        &self,
        deps: DepsMut,
        env: Env,
        info: MessageInfo,
        msg: ExecuteMsg<T, E>,
    ) -> Result<Response<C>, ContractError> {
        match msg {
            ExecuteMsg::Mint { .. } => self.mint(),
            ExecuteMsg::MintTeam {
                quantity,
                extension,
            } => self.mint_team(deps, info, quantity, extension),
            ExecuteMsg::MintAllowlist {
                quantity,
                extension,
            } => self.mint_allowlist(deps, info, quantity, extension),
            ExecuteMsg::MintPublic {
                quantity,
                extension,
            } => self.mint_public(deps, info, quantity, extension),
            ExecuteMsg::Approve {
                spender,
                token_id,
                expires,
            } => self.approve(deps, env, info, spender, token_id, expires),
            ExecuteMsg::Revoke { spender, token_id } => {
                self.revoke(deps, env, info, spender, token_id)
            }
            ExecuteMsg::ApproveAll { operator, expires } => {
                self.approve_all(deps, env, info, operator, expires)
            }
            ExecuteMsg::RevokeAll { operator } => self.revoke_all(deps, env, info, operator),
            ExecuteMsg::TransferNft {
                recipient,
                token_id,
            } => self.transfer_nft(deps, env, info, recipient, token_id),
            ExecuteMsg::SendNft {
                contract,
                token_id,
                msg,
            } => self.send_nft(deps, env, info, contract, token_id, msg),
            ExecuteMsg::Burn { token_id } => self.burn(deps, env, info, token_id),
            ExecuteMsg::UpdateOwnership(action) => Self::update_ownership(deps, env, info, action),
            ExecuteMsg::Extension { msg: _ } => Ok(Response::default()),
            ExecuteMsg::SetWithdrawAddress { address } => {
                self.set_withdraw_address(deps.storage, deps.api, &info.sender, address)
            }
            ExecuteMsg::RemoveWithdrawAddress {} => {
                self.remove_withdraw_address(deps.storage, &info.sender)
            }
            ExecuteMsg::WithdrawFunds { amount } => self.withdraw_funds(deps.storage, &amount),
            ExecuteMsg::SetBaseTokenUri { base_token_uri } => {
                self.set_base_token_uri(deps, &info.sender, base_token_uri)
            }
            ExecuteMsg::SetSaleConfig {
                allowlist_price,
                public_price,
                max_per_allowlist,
                max_per_public,
            } => self.set_sale_config(
                deps,
                &info.sender,
                allowlist_price,
                public_price,
                max_per_allowlist,
                max_per_public,
            ),
            ExecuteMsg::SeedAllowlist { addresses } => {
                self.seed_allowlist(deps, &info.sender, addresses)
            }
            ExecuteMsg::SetAllowlistSale { open } => {
                self.set_allowlist_sale(deps, &info.sender, open)
            }
            ExecuteMsg::SetPublicSale { open } => self.set_public_sale(deps, &info.sender, open),
        }
    }
}

impl<'a, T, C, E, Q> Cw721Contract<'a, T, C, E, Q>
where
    T: Serialize + DeserializeOwned + Clone,
    C: CustomMsg,
    E: CustomMsg,
    Q: CustomMsg,
{
    pub fn mint(&self) -> Result<Response<C>, ContractError> {
        Err(ContractError::MintDisabled)
    }

    pub fn mint_team(
        &self,
        deps: DepsMut,
        info: MessageInfo,
        quantity: u64,
        extension: T,
    ) -> Result<Response<C>, ContractError> {
        cw_ownable::assert_owner(deps.storage, &info.sender)?;

        // Make sure number of tokens doesn't exceed collection size
        let collection_size = self.collection_size.load(deps.storage)?;
        let token_count = self.token_count(deps.storage)?;
        if token_count + quantity > collection_size {
            return Err(ContractError::MaxSupplyReached {});
        }

        // Create tokens based on quantity
        for i in 0..quantity {
            // Create the token
            let token_id = (token_count + i).to_string();
            let token = TokenInfo {
                owner: info.sender.clone(),
                approvals: vec![],
                extension: extension.clone(),
            };
            self.tokens
                .update(deps.storage, &token_id, |old| match old {
                    Some(_) => Err(ContractError::Claimed {}),
                    None => Ok(token),
                })?;
        }

        // Update the total minted count
        self.increment_tokens(deps.storage, quantity)?;

        Ok(Response::new()
            .add_attribute("action", "mint")
            .add_attribute("minter", info.sender)
            .add_attribute("quantity", quantity.to_string()))
    }

    pub fn mint_allowlist(
        &self,
        deps: DepsMut,
        info: MessageInfo,
        quantity: u64,
        extension: T,
    ) -> Result<Response<C>, ContractError> {
        let sale_config = self.sale_config.load(deps.storage)?;

        // Verify if the sender is on the allowlist
        let is_allowed = self
            .allowlist
            .may_load(deps.storage, &info.sender)?
            .unwrap_or(false);
        if !is_allowed {
            return Err(ContractError::NotOnAllowlist {});
        }

        // Make sure quantity doesn't exceed max per allowlist
        if quantity > sale_config.max_per_allowlist {
            return Err(ContractError::MaxMintReached {});
        }

        // Make sure number of tokens doesn't exceed collection size
        let collection_size = self.collection_size.load(deps.storage)?;
        let token_count = self.token_count(deps.storage)?;
        if token_count + quantity > collection_size {
            return Err(ContractError::MaxSupplyReached {});
        }

        // Create tokens based on quantity
        for i in 0..quantity {
            let token_id = (token_count + i).to_string();
            let token = TokenInfo {
                owner: info.sender.clone(),
                approvals: vec![],
                extension: extension.clone(),
            };
            self.tokens
                .update(deps.storage, &token_id, |old| match old {
                    Some(_) => Err(ContractError::Claimed {}),
                    None => Ok(token),
                })?;
        }

        // Update the total minted count
        self.increment_tokens(deps.storage, quantity)?;
        // Remove from allowlist
        self.allowlist.remove(deps.storage, &info.sender);

        Ok(Response::new()
            .add_attribute("action", "mint_allowlist")
            .add_attribute("minter", info.sender)
            .add_attribute("quantity", quantity.to_string()))
    }

    pub fn mint_public(
        &self,
        deps: DepsMut,
        info: MessageInfo,
        quantity: u64,
        extension: T,
    ) -> Result<Response<C>, ContractError> {
        // Make sure quantity doesn't exceed max
        let sale_config = self.sale_config.load(deps.storage)?;
        if quantity > sale_config.max_per_public {
            return Err(ContractError::MaxMintReached {});
        }

        // Make sure number of tokens doesn't exceed collection size
        let collection_size = self.collection_size.load(deps.storage)?;
        let token_count = self.token_count(deps.storage)?;
        if token_count + quantity > collection_size {
            return Err(ContractError::MaxSupplyReached {});
        }

        // Make sure enough funds are sent
        let total_price = sale_config.public_price.multiply_ratio(quantity, 1u64);
        let sent_amount = info
            .funds
            .iter()
            .find(|coin| coin.denom == "usei")
            .map_or(Uint128::zero(), |coin| coin.amount);
        if sent_amount < total_price {
            return Err(ContractError::InsufficientFunds {});
        }

        // Create tokens based on quantity
        for i in 0..quantity {
            // Create the token
            let token_id = (token_count + i).to_string();
            let token = TokenInfo {
                owner: info.sender.clone(),
                approvals: vec![],
                extension: extension.clone(),
            };
            self.tokens
                .update(deps.storage, &token_id, |old| match old {
                    Some(_) => Err(ContractError::Claimed {}),
                    None => Ok(token),
                })?;
        }

        // Update the total minted count
        self.increment_tokens(deps.storage, quantity)?;

        Ok(Response::new()
            .add_attribute("action", "mint")
            .add_attribute("minter", info.sender)
            .add_attribute("quantity", quantity.to_string()))
    }

    pub fn update_ownership(
        deps: DepsMut,
        env: Env,
        info: MessageInfo,
        action: cw_ownable::Action,
    ) -> Result<Response<C>, ContractError> {
        let ownership = cw_ownable::update_ownership(deps, &env.block, &info.sender, action)?;
        Ok(Response::new().add_attributes(ownership.into_attributes()))
    }

    pub fn set_withdraw_address(
        &self,
        storage: &mut dyn Storage,
        api: &dyn Api,
        sender: &Addr,
        address: String,
    ) -> Result<Response<C>, ContractError> {
        cw_ownable::assert_owner(storage, sender)?;
        api.addr_validate(&address)?;
        self.withdraw_address.save(storage, &address)?;
        Ok(Response::new()
            .add_attribute("action", "set_withdraw_address")
            .add_attribute("address", address))
    }

    pub fn remove_withdraw_address(
        &self,
        storage: &mut dyn Storage,
        sender: &Addr,
    ) -> Result<Response<C>, ContractError> {
        cw_ownable::assert_owner(storage, sender)?;
        let address = self.withdraw_address.may_load(storage)?;
        match address {
            Some(address) => {
                self.withdraw_address.remove(storage);
                Ok(Response::new()
                    .add_attribute("action", "remove_withdraw_address")
                    .add_attribute("address", address))
            }
            None => Err(ContractError::NoWithdrawAddress {}),
        }
    }

    pub fn withdraw_funds(
        &self,
        storage: &mut dyn Storage,
        amount: &Coin,
    ) -> Result<Response<C>, ContractError> {
        let address = self.withdraw_address.may_load(storage)?;
        match address {
            Some(address) => {
                let msg = BankMsg::Send {
                    to_address: address,
                    amount: vec![amount.clone()],
                };
                Ok(Response::new()
                    .add_message(msg)
                    .add_attribute("action", "withdraw_funds")
                    .add_attribute("amount", amount.amount.to_string())
                    .add_attribute("denom", amount.denom.to_string()))
            }
            None => Err(ContractError::NoWithdrawAddress {}),
        }
    }

    pub fn set_base_token_uri(
        &self,
        deps: DepsMut,
        sender: &Addr,
        base_token_uri: String,
    ) -> Result<Response<C>, ContractError> {
        cw_ownable::assert_owner(deps.storage, sender)?;

        self.base_token_uri.save(deps.storage, &base_token_uri)?;

        Ok(Response::new()
            .add_attribute("action", "set_base_token_uri")
            .add_attribute("base_token_uri", base_token_uri))
    }

    pub fn set_sale_config(
        &self,
        deps: DepsMut,
        sender: &Addr,
        allowlist_price: Uint128,
        public_price: Uint128,
        max_per_allowlist: u64,
        max_per_public: u64,
    ) -> Result<Response<C>, ContractError> {
        cw_ownable::assert_owner(deps.storage, sender)?;

        let mut sale_config = self.sale_config.load(deps.storage)?;
        sale_config.allowlist_price = allowlist_price;
        sale_config.public_price = public_price;
        sale_config.max_per_allowlist = max_per_allowlist;
        sale_config.max_per_public = max_per_public;
        self.sale_config.save(deps.storage, &sale_config)?;

        Ok(Response::new()
            .add_attribute("action", "set_sale_config")
            .add_attribute("allowlist_price", allowlist_price)
            .add_attribute("public_price", public_price)
            .add_attribute("max_per_allowlist", max_per_allowlist.to_string())
            .add_attribute("max_per_public", max_per_public.to_string()))
    }

    pub fn seed_allowlist(
        &self,
        deps: DepsMut,
        sender: &Addr,
        addresses: Vec<String>,
    ) -> Result<Response<C>, ContractError> {
        cw_ownable::assert_owner(deps.storage, sender)?;
        for address in addresses.clone() {
            let allowlist_addr = deps.api.addr_validate(&address)?;
            self.allowlist.save(deps.storage, &allowlist_addr, &true)?;
        }
        Ok(Response::new()
            .add_attribute("action", "seed_allowlist")
            .add_attribute("num_addresses", addresses.len().to_string()))
    }

    pub fn set_allowlist_sale(
        &self,
        deps: DepsMut,
        sender: &Addr,
        open: bool,
    ) -> Result<Response<C>, ContractError> {
        cw_ownable::assert_owner(deps.storage, sender)?;

        let mut sale_config = self.sale_config.load(deps.storage)?;
        sale_config.allowlist_sale_open = open;
        self.sale_config.save(deps.storage, &sale_config)?;

        Ok(Response::new()
            .add_attribute("action", "set_allowlist_sale")
            .add_attribute("open", open.to_string()))
    }

    pub fn set_public_sale(
        &self,
        deps: DepsMut,
        sender: &Addr,
        open: bool,
    ) -> Result<Response<C>, ContractError> {
        cw_ownable::assert_owner(deps.storage, sender)?;

        let mut sale_config = self.sale_config.load(deps.storage)?;
        sale_config.public_sale_open = open;
        self.sale_config.save(deps.storage, &sale_config)?;

        Ok(Response::new()
            .add_attribute("action", "set_public_sale")
            .add_attribute("open", open.to_string()))
    }
}

impl<'a, T, C, E, Q> Cw721Execute<T, C> for Cw721Contract<'a, T, C, E, Q>
where
    T: Serialize + DeserializeOwned + Clone,
    C: CustomMsg,
    E: CustomMsg,
    Q: CustomMsg,
{
    type Err = ContractError;

    fn transfer_nft(
        &self,
        deps: DepsMut,
        env: Env,
        info: MessageInfo,
        recipient: String,
        token_id: String,
    ) -> Result<Response<C>, ContractError> {
        self._transfer_nft(deps, &env, &info, &recipient, &token_id)?;

        Ok(Response::new()
            .add_attribute("action", "transfer_nft")
            .add_attribute("sender", info.sender)
            .add_attribute("recipient", recipient)
            .add_attribute("token_id", token_id))
    }

    fn send_nft(
        &self,
        deps: DepsMut,
        env: Env,
        info: MessageInfo,
        contract: String,
        token_id: String,
        msg: Binary,
    ) -> Result<Response<C>, ContractError> {
        // Transfer token
        self._transfer_nft(deps, &env, &info, &contract, &token_id)?;

        let send = Cw721ReceiveMsg {
            sender: info.sender.to_string(),
            token_id: token_id.clone(),
            msg,
        };

        // Send message
        Ok(Response::new()
            .add_message(send.into_cosmos_msg(contract.clone())?)
            .add_attribute("action", "send_nft")
            .add_attribute("sender", info.sender)
            .add_attribute("recipient", contract)
            .add_attribute("token_id", token_id))
    }

    fn approve(
        &self,
        deps: DepsMut,
        env: Env,
        info: MessageInfo,
        spender: String,
        token_id: String,
        expires: Option<Expiration>,
    ) -> Result<Response<C>, ContractError> {
        self._update_approvals(deps, &env, &info, &spender, &token_id, true, expires)?;

        Ok(Response::new()
            .add_attribute("action", "approve")
            .add_attribute("sender", info.sender)
            .add_attribute("spender", spender)
            .add_attribute("token_id", token_id))
    }

    fn revoke(
        &self,
        deps: DepsMut,
        env: Env,
        info: MessageInfo,
        spender: String,
        token_id: String,
    ) -> Result<Response<C>, ContractError> {
        self._update_approvals(deps, &env, &info, &spender, &token_id, false, None)?;

        Ok(Response::new()
            .add_attribute("action", "revoke")
            .add_attribute("sender", info.sender)
            .add_attribute("spender", spender)
            .add_attribute("token_id", token_id))
    }

    fn approve_all(
        &self,
        deps: DepsMut,
        env: Env,
        info: MessageInfo,
        operator: String,
        expires: Option<Expiration>,
    ) -> Result<Response<C>, ContractError> {
        // reject expired data as invalid
        let expires = expires.unwrap_or_default();
        if expires.is_expired(&env.block) {
            return Err(ContractError::Expired {});
        }

        // set the operator for us
        let operator_addr = deps.api.addr_validate(&operator)?;
        self.operators
            .save(deps.storage, (&info.sender, &operator_addr), &expires)?;

        Ok(Response::new()
            .add_attribute("action", "approve_all")
            .add_attribute("sender", info.sender)
            .add_attribute("operator", operator))
    }

    fn revoke_all(
        &self,
        deps: DepsMut,
        _env: Env,
        info: MessageInfo,
        operator: String,
    ) -> Result<Response<C>, ContractError> {
        let operator_addr = deps.api.addr_validate(&operator)?;
        self.operators
            .remove(deps.storage, (&info.sender, &operator_addr));

        Ok(Response::new()
            .add_attribute("action", "revoke_all")
            .add_attribute("sender", info.sender)
            .add_attribute("operator", operator))
    }

    fn burn(
        &self,
        _deps: DepsMut,
        _env: Env,
        _info: MessageInfo,
        _token_id: String,
    ) -> Result<Response<C>, ContractError> {
        Err(ContractError::BurnDisabled {})
    }
}

// helpers
impl<'a, T, C, E, Q> Cw721Contract<'a, T, C, E, Q>
where
    T: Serialize + DeserializeOwned + Clone,
    C: CustomMsg,
    E: CustomMsg,
    Q: CustomMsg,
{
    pub fn _transfer_nft(
        &self,
        deps: DepsMut,
        env: &Env,
        info: &MessageInfo,
        recipient: &str,
        token_id: &str,
    ) -> Result<TokenInfo<T>, ContractError> {
        let mut token = self.tokens.load(deps.storage, token_id)?;
        // ensure we have permissions
        self.check_can_send(deps.as_ref(), env, info, &token)?;
        // set owner and remove existing approvals
        token.owner = deps.api.addr_validate(recipient)?;
        token.approvals = vec![];
        self.tokens.save(deps.storage, token_id, &token)?;
        Ok(token)
    }

    #[allow(clippy::too_many_arguments)]
    pub fn _update_approvals(
        &self,
        deps: DepsMut,
        env: &Env,
        info: &MessageInfo,
        spender: &str,
        token_id: &str,
        // if add == false, remove. if add == true, remove then set with this expiration
        add: bool,
        expires: Option<Expiration>,
    ) -> Result<TokenInfo<T>, ContractError> {
        let mut token = self.tokens.load(deps.storage, token_id)?;
        // ensure we have permissions
        self.check_can_approve(deps.as_ref(), env, info, &token)?;

        // update the approval list (remove any for the same spender before adding)
        let spender_addr = deps.api.addr_validate(spender)?;
        token.approvals.retain(|apr| apr.spender != spender_addr);

        // only difference between approve and revoke
        if add {
            // reject expired data as invalid
            let expires = expires.unwrap_or_default();
            if expires.is_expired(&env.block) {
                return Err(ContractError::Expired {});
            }
            let approval = Approval {
                spender: spender_addr,
                expires,
            };
            token.approvals.push(approval);
        }

        self.tokens.save(deps.storage, token_id, &token)?;

        Ok(token)
    }

    /// returns true iff the sender can execute approve or reject on the contract
    pub fn check_can_approve(
        &self,
        deps: Deps,
        env: &Env,
        info: &MessageInfo,
        token: &TokenInfo<T>,
    ) -> Result<(), ContractError> {
        // owner can approve
        if token.owner == info.sender {
            return Ok(());
        }
        // operator can approve
        let op = self
            .operators
            .may_load(deps.storage, (&token.owner, &info.sender))?;
        match op {
            Some(ex) => {
                if ex.is_expired(&env.block) {
                    Err(ContractError::Ownership(OwnershipError::NotOwner))
                } else {
                    Ok(())
                }
            }
            None => Err(ContractError::Ownership(OwnershipError::NotOwner)),
        }
    }

    /// returns true iff the sender can transfer ownership of the token
    pub fn check_can_send(
        &self,
        deps: Deps,
        env: &Env,
        info: &MessageInfo,
        token: &TokenInfo<T>,
    ) -> Result<(), ContractError> {
        // owner can send
        if token.owner == info.sender {
            return Ok(());
        }

        // any non-expired token approval can send
        if token
            .approvals
            .iter()
            .any(|apr| apr.spender == info.sender && !apr.is_expired(&env.block))
        {
            return Ok(());
        }

        // operator can send
        let op = self
            .operators
            .may_load(deps.storage, (&token.owner, &info.sender))?;
        match op {
            Some(ex) => {
                if ex.is_expired(&env.block) {
                    Err(ContractError::Ownership(OwnershipError::NotOwner))
                } else {
                    Ok(())
                }
            }
            None => Err(ContractError::Ownership(OwnershipError::NotOwner)),
        }
    }
}
