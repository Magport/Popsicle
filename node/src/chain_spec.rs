use cumulus_primitives_core::ParaId;
use runtime_common::{AccountId, AuraId, Signature};
use sc_chain_spec::{ChainSpecExtension, ChainSpecGroup};
use sc_service::ChainType;
use serde::{Deserialize, Serialize};
use sp_core::{sr25519, Pair, Public};
use sp_runtime::traits::{IdentifyAccount, Verify};

/// Specialized `ChainSpec` for the normal parachain runtime.
pub type PopsicleChainSpec =
	sc_service::GenericChainSpec<popsicle_runtime::RuntimeGenesisConfig, Extensions>;

/// The default XCM version to set in genesis config.
const SAFE_XCM_VERSION: u32 = xcm::prelude::XCM_VERSION;

const PARA_ID: u32 = 2000;

/// Helper function to generate a crypto pair from seed
pub fn get_from_seed<TPublic: Public>(seed: &str) -> <TPublic::Pair as Pair>::Public {
	TPublic::Pair::from_string(&format!("//{}", seed), None)
		.expect("static values are valid; qed")
		.public()
}

/// The extensions for the [`ChainSpec`].
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ChainSpecGroup, ChainSpecExtension)]
#[serde(deny_unknown_fields)]
pub struct Extensions {
	/// The relay chain of the Parachain.
	pub relay_chain: String,
	/// The id of the Parachain.
	pub para_id: u32,
}

impl Extensions {
	/// Try to get the extension from the given `ChainSpec`.
	pub fn try_get(chain_spec: &dyn sc_service::ChainSpec) -> Option<&Self> {
		sc_chain_spec::get_extension(chain_spec.extensions())
	}
}

type AccountPublic = <Signature as Verify>::Signer;

/// Generate collator keys from seed.
///
/// This function's return type must always match the session keys of the chain in tuple format.
pub fn get_collator_keys_from_seed(seed: &str) -> AuraId {
	get_from_seed::<AuraId>(seed)
}

/// Helper function to generate an account ID from seed
pub fn get_account_id_from_seed<TPublic: Public>(seed: &str) -> AccountId
where
	AccountPublic: From<<TPublic::Pair as Pair>::Public>,
{
	AccountPublic::from(get_from_seed::<TPublic>(seed)).into_account()
}

/// Generate the session keys from individual elements.
///
/// The input must be a tuple of individual keys (a single arg for now since we have just one key).
pub fn popsicle_session_keys(keys: AuraId) -> popsicle_runtime::SessionKeys {
	popsicle_runtime::SessionKeys { aura: keys }
}

pub mod popsicle {
	use super::*;
	pub fn development_config() -> PopsicleChainSpec {
		// Give your base currency a unit name and decimal places
		let mut properties = sc_chain_spec::Properties::new();
		properties.insert("tokenSymbol".into(), "UNIT".into());
		properties.insert("tokenDecimals".into(), 12.into());
		properties.insert("ss58Format".into(), 42.into());

		PopsicleChainSpec::from_genesis(
			// Name
			"Popsicle Development",
			// ID
			"popsicle_dev",
			ChainType::Development,
			move || {
				popsicle_genesis(
					// initial collators.
					vec![
						(
							get_account_id_from_seed::<sr25519::Public>("Alice"),
							get_collator_keys_from_seed("Alice"),
						),
						(
							get_account_id_from_seed::<sr25519::Public>("Bob"),
							get_collator_keys_from_seed("Bob"),
						),
					],
					vec![
						get_account_id_from_seed::<sr25519::Public>("Alice"),
						get_account_id_from_seed::<sr25519::Public>("Bob"),
						get_account_id_from_seed::<sr25519::Public>("Charlie"),
						get_account_id_from_seed::<sr25519::Public>("Dave"),
						get_account_id_from_seed::<sr25519::Public>("Eve"),
						get_account_id_from_seed::<sr25519::Public>("Ferdie"),
						get_account_id_from_seed::<sr25519::Public>("Alice//stash"),
						get_account_id_from_seed::<sr25519::Public>("Bob//stash"),
						get_account_id_from_seed::<sr25519::Public>("Charlie//stash"),
						get_account_id_from_seed::<sr25519::Public>("Dave//stash"),
						get_account_id_from_seed::<sr25519::Public>("Eve//stash"),
						get_account_id_from_seed::<sr25519::Public>("Ferdie//stash"),
					],
					get_account_id_from_seed::<sr25519::Public>("Alice"),
					PARA_ID.into(),
				)
			},
			Vec::new(),
			None,
			None,
			None,
			None,
			Extensions {
				relay_chain: "rococo-local".into(), // You MUST set this to the correct network!
				para_id: PARA_ID,
			},
		)
	}

	pub fn local_testnet_config() -> PopsicleChainSpec {
		// Give your base currency a unit name and decimal places
		let mut properties = sc_chain_spec::Properties::new();
		properties.insert("tokenSymbol".into(), "UNIT".into());
		properties.insert("tokenDecimals".into(), 12.into());
		properties.insert("ss58Format".into(), 42.into());

		PopsicleChainSpec::from_genesis(
			// Name
			"Popsicle Local Testnet",
			// ID
			"popsicle_local_testnet",
			ChainType::Local,
			move || {
				popsicle_genesis(
					// initial collators.
					vec![
						(
							get_account_id_from_seed::<sr25519::Public>("Alice"),
							get_collator_keys_from_seed("Alice"),
						),
						(
							get_account_id_from_seed::<sr25519::Public>("Bob"),
							get_collator_keys_from_seed("Bob"),
						),
					],
					vec![
						get_account_id_from_seed::<sr25519::Public>("Alice"),
						get_account_id_from_seed::<sr25519::Public>("Bob"),
						get_account_id_from_seed::<sr25519::Public>("Charlie"),
						get_account_id_from_seed::<sr25519::Public>("Dave"),
						get_account_id_from_seed::<sr25519::Public>("Eve"),
						get_account_id_from_seed::<sr25519::Public>("Ferdie"),
						get_account_id_from_seed::<sr25519::Public>("Alice//stash"),
						get_account_id_from_seed::<sr25519::Public>("Bob//stash"),
						get_account_id_from_seed::<sr25519::Public>("Charlie//stash"),
						get_account_id_from_seed::<sr25519::Public>("Dave//stash"),
						get_account_id_from_seed::<sr25519::Public>("Eve//stash"),
						get_account_id_from_seed::<sr25519::Public>("Ferdie//stash"),
					],
					get_account_id_from_seed::<sr25519::Public>("Alice"),
					PARA_ID.into(),
				)
			},
			// Bootnodes
			Vec::new(),
			// Telemetry
			None,
			// Protocol ID
			Some("popsicle-local"),
			// Fork ID
			None,
			// Properties
			Some(properties),
			// Extensions
			Extensions {
				relay_chain: "rococo-local".into(), // You MUST set this to the correct network!
				para_id: PARA_ID,
			},
		)
	}

	fn popsicle_genesis(
		invulnerables: Vec<(AccountId, AuraId)>,
		endowed_accounts: Vec<AccountId>,
		root_key: AccountId,
		id: ParaId,
	) -> popsicle_runtime::RuntimeGenesisConfig {
		use popsicle_runtime::EXISTENTIAL_DEPOSIT;
		let alice = get_from_seed::<sr25519::Public>("Alice");
		let bob = get_from_seed::<sr25519::Public>("Bob");

		// If root key is not in endowed accounts, add it.
		let mut endowed_accounts = endowed_accounts;
		if !endowed_accounts.contains(&root_key) {
			endowed_accounts.push(root_key.clone());
		}

		popsicle_runtime::RuntimeGenesisConfig {
			system: popsicle_runtime::SystemConfig {
				code: popsicle_runtime::WASM_BINARY
					.expect("WASM binary was not build, please build it!")
					.to_vec(),
				..Default::default()
			},
			balances: popsicle_runtime::BalancesConfig {
				balances: endowed_accounts.iter().cloned().map(|k| (k, 1 << 60)).collect(),
			},
			// Configure two assets ALT1 & ALT2 with two owners, alice and bob respectively
			assets: popsicle_runtime::AssetsConfig {
				assets: vec![
					(1, alice.into(), true, 100_000_000_000),
					(2, bob.into(), true, 100_000_000_000),
				],
				// Genesis metadata: Vec<(id, name, symbol, decimals)>
				metadata: vec![
					(1, "asset-1".into(), "ALT1".into(), 10),
					(2, "asset-2".into(), "ALT2".into(), 10),
				],
				// Genesis accounts: Vec<(id, account_id, balance)>
				accounts: vec![
					(1, alice.into(), 500_000_000_000),
					(2, bob.into(), 500_000_000_000),
				],
			},
			parachain_info: popsicle_runtime::ParachainInfoConfig {
				parachain_id: id,
				..Default::default()
			},
			collator_selection: popsicle_runtime::CollatorSelectionConfig {
				invulnerables: invulnerables.iter().cloned().map(|(acc, _)| acc).collect(),
				candidacy_bond: EXISTENTIAL_DEPOSIT * 16,
				..Default::default()
			},
			session: popsicle_runtime::SessionConfig {
				keys: invulnerables
					.into_iter()
					.map(|(acc, aura)| {
						(
							acc.clone(),                 // account id
							acc,                         // validator id
							popsicle_session_keys(aura), // session keys
						)
					})
					.collect(),
			},
			// no need to pass anything to aura, in fact it will panic if we do. Session will take care
			// of this.
			aura: Default::default(),
			aura_ext: Default::default(),
			sudo: popsicle_runtime::SudoConfig { key: Some(root_key) },
			parachain_system: Default::default(),
			polkadot_xcm: popsicle_runtime::PolkadotXcmConfig {
				safe_xcm_version: Some(SAFE_XCM_VERSION),
				..Default::default()
			},
			transaction_payment: Default::default(),
		}
	}
}
