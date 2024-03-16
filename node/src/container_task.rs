use cumulus_primitives_core::{
	relay_chain::BlockNumber as RelayBlockNumber, ParaId, PersistedValidationData,
};
use cumulus_relay_chain_interface::{RelayChainInterface, RelayChainResult};
use futures::{lock::Mutex, pin_mut, select, FutureExt, Stream, StreamExt};
use parity_scale_codec::Decode;
use polkadot_primitives::OccupiedCoreAssumption;
use primitives_container::{ContainerRuntimeApi, DownloadInfo};
use reqwest::{
	self,
	header::{HeaderValue, CONTENT_LENGTH, RANGE},
	StatusCode,
};
use ring::digest::{Context, Digest, SHA256};
use sc_client_api::UsageProvider;
use sc_service::TaskManager;
use sp_api::ProvideRuntimeApi;
use sp_core::H256;
use sp_keystore::KeystorePtr;
use sp_runtime::traits::{Block as BlockT, Header as HeaderT};
use sp_runtime::AccountId32;
use std::{
	error::Error,
	fs::Permissions,
	os::unix::fs::PermissionsExt,
	path::{Path, PathBuf},
	sync::Arc,
};
use std::{
	fs,
	fs::File,
	io::{BufReader, Read},
	process::{Command, Stdio},
	str::FromStr,
};
struct PartialRangeIter {
	start: u64,
	end: u64,
	buffer_size: u32,
}

impl PartialRangeIter {
	pub fn new(start: u64, end: u64, buffer_size: u32) -> Result<Self, Box<dyn Error>> {
		if buffer_size == 0 {
			Err("invalid buffer_size, give a value greater than zero.")?;
		}
		Ok(PartialRangeIter { start, end, buffer_size })
	}
}

impl Iterator for PartialRangeIter {
	type Item = HeaderValue;
	fn next(&mut self) -> Option<Self::Item> {
		if self.start > self.end {
			None
		} else {
			let prev_start = self.start;
			self.start += std::cmp::min(self.buffer_size as u64, self.end - self.start + 1);
			Some(
				HeaderValue::from_str(&format!("bytes={}-{}", prev_start, self.start - 1))
					.expect("string provided by format!"),
			)
		}
	}
}

async fn sha256_digest<R: Read>(mut reader: R) -> Result<Digest, Box<dyn Error + Send + Sync>> {
	let mut context = Context::new(&SHA256);
	let mut buffer = [0; 1024];

	loop {
		let count = reader.read(&mut buffer)?;
		if count == 0 {
			break;
		}
		context.update(&buffer[..count]);
	}

	Ok(context.finish())
}

async fn download_sdk(
	data_path: PathBuf,
	app_info: DownloadInfo,
	url: &str,
) -> Result<(), Box<dyn Error + Send + Sync>> {
	//firt create dir
	let path_str = format!("{}/sdk", data_path.as_os_str().to_str().ok_or("path error!")?);
	let download_dir = Path::new(&path_str);
	if !download_dir.exists() {
		fs::create_dir(&download_dir)?
	}

	const CHUNK_SIZE: u32 = 1024000; // 1 M

	let client = reqwest::blocking::Client::new();
	let web_path = format!("{}/{}", url, std::str::from_utf8(&app_info.file_name)?);
	log::info!("=============download:{:?}", web_path);
	let response = client.head(&web_path).send()?;
	let length = response
		.headers()
		.get(CONTENT_LENGTH)
		.ok_or("response doesn't include the content length")?;
	let length = u64::from_str(length.to_str()?).map_err(|_| "invalid Content-Length header")?;
	log::info!("==========total length:{:?}", length);
	let download_path = format!("{}/{}", path_str, std::str::from_utf8(&app_info.file_name)?);
	log::info!("=============download_path:{:?}", download_path);
	let download_dir = Path::new(&download_path);

	let mut output_file = File::create(download_dir)?;

	println!("starting download...");
	for range in PartialRangeIter::new(0, length - 1, CHUNK_SIZE).unwrap() {
		println!("range {:?}", range);
		let mut response = client.get(&web_path).header(RANGE, range).send()?;

		let status = response.status();
		if !(status == StatusCode::OK || status == StatusCode::PARTIAL_CONTENT) {
			log::info!("Unexpected server response: {:?}", status);
			break;
		}
		std::io::copy(&mut response, &mut output_file)?;
	}

	let content = response.text()?;
	std::io::copy(&mut content.as_bytes(), &mut output_file)?;

	println!("Finished with success!");
	output_file.set_permissions(Permissions::from_mode(0o777))?;
	//check file hash
	println!("check file hash");
	let input = File::open(download_dir)?;
	let reader = BufReader::new(input);
	let digest = sha256_digest(reader).await?;

	println!("SHA-256 digest is {:?}", digest);
	if digest.as_ref() == app_info.app_hash.as_bytes() {
		println!("check ok");
	} else {
		println!("check fail");
	}

	Ok(())
}
async fn need_download(
	data_path: &str,
	app_hash: H256,
) -> Result<bool, Box<dyn Error + Send + Sync>> {
	let input =
		if let Ok(open_file) = File::open(data_path) { open_file } else { return Ok(true) };
	let reader = BufReader::new(input);
	let digest = sha256_digest(reader).await?;

	println!("SHA-256 digest is {:?}", digest);
	if digest.as_ref() == app_hash.as_bytes() {
		Ok(false)
	} else {
		Ok(true)
	}
}

struct RunningApp {
	group_id: u32,
}
async fn process_rollup_task(
	data_path: PathBuf,
	app_info: DownloadInfo,
) -> Result<(), Box<dyn Error + Send + Sync>> {
	let url = std::str::from_utf8(&app_info.url).unwrap();
	let mut start_flag = false;
	let download_path = format!(
		"{}/sdk/{}",
		data_path.as_os_str().to_str().unwrap(),
		std::str::from_utf8(&app_info.file_name).unwrap()
	);
	let need_download = need_download(&download_path, app_info.app_hash).await;
	log::info!("need_download:{:?}", need_download);
	if let Ok(need_down) = need_download {
		if need_down {
			let result = download_sdk(data_path.clone(), app_info.clone(), url).await;
			if result.is_ok() {
				start_flag = true;
			} else {
				log::info!("download sdk error:{:?}", result);
			}
		} else {
			//only download then start
			// start_flag = true;
		}
	}

	if start_flag {
		// let args: Vec<&str> =
		// 	app_info.args.iter().map(|arg| std::str::from_utf8(&arg).unwrap()).collect();
		let args: Vec<&str> = std::str::from_utf8(&app_info.args)
			.unwrap()
			.split(' ')
			.into_iter()
			.map(|arg| std::str::from_utf8(arg.as_bytes()).unwrap())
			.collect();
		let log_file_name;
		let log_file_buf;
		if app_info.log.is_none() {
			log_file_name = std::str::from_utf8(&app_info.file_name).unwrap();
		} else {
			log_file_buf = app_info.log.unwrap();
			log_file_name = std::str::from_utf8(&log_file_buf).unwrap();
		}
		log::info!("log_file_name:{:?}", log_file_name);
		log::info!("args:{:?}", args);
		let outputs = File::create(log_file_name)?;
		let errors = outputs.try_clone()?;
		Command::new(download_path)
			.stdin(Stdio::piped())
			.stderr(Stdio::from(outputs))
			.stdout(Stdio::from(errors))
			.args(args)
			.spawn()
			.expect("failed to execute process");
	}
	Ok(())
}

async fn handle_new_best_parachain_head<P, Block>(
	validation_data: PersistedValidationData,
	height: RelayBlockNumber,
	parachain: &P,
	keystore: KeystorePtr,
	relay_chain: impl RelayChainInterface + Clone,
	p_hash: H256,
	para_id: ParaId,
	data_path: PathBuf,
	running_app: Arc<Mutex<RunningApp>>,
) -> Result<(), Box<dyn Error>>
where
	Block: BlockT,
	P: ProvideRuntimeApi<Block> + UsageProvider<Block>,
	P::Api: ContainerRuntimeApi<Block, AccountId32>,
	<Block::Header as HeaderT>::Number: Into<u32>,
{
	// Check if there is a download task
	let head = validation_data.clone().parent_head.0;
	let parachain_head = match <<Block as BlockT>::Header>::decode(&mut &head[..]) {
		Ok(header) => header,
		Err(err) => return Err(format!("get parachain head error:{:?}", err).into()),
	};
	let hash = parachain_head.hash();
	let xx = keystore.sr25519_public_keys(sp_application_crypto::key_types::AURA)[0];
	let should_load: Option<DownloadInfo> =
		parachain.runtime_api().shuld_load(hash, xx.into()).unwrap();
	let number = (*parachain_head.number()).into();
	let mut app = running_app.lock().await;
	let old_group_id = app.group_id;
	match should_load {
		Some(app_info) => {
			let new_group = app_info.group;
			if old_group_id != new_group {
				tokio::spawn(process_rollup_task(data_path, app_info));
				app.group_id = new_group;
			}
		},
		None => log::info!("None"),
	}
	Ok(())
}

async fn new_best_heads(
	relay_chain: impl RelayChainInterface + Clone,
	para_id: ParaId,
) -> RelayChainResult<impl Stream<Item = (u32, PersistedValidationData, H256)>> {
	let new_best_notification_stream =
		relay_chain.new_best_notification_stream().await?.filter_map(move |n| {
			let relay_chain = relay_chain.clone();
			async move {
				let relay_head: PersistedValidationData = relay_chain
					.persisted_validation_data(n.hash(), para_id, OccupiedCoreAssumption::TimedOut)
					.await
					.map(|s| s.map(|s| s))
					.ok()
					.flatten()?;
				Some((n.number, relay_head, n.hash()))
			}
		});

	Ok(new_best_notification_stream)
}

async fn relay_chain_notification<P, R, Block>(
	para_id: ParaId,
	parachain: Arc<P>,
	relay_chain: R,
	keystore: KeystorePtr,
	data_path: PathBuf,
) where
	R: RelayChainInterface + Clone,
	Block: BlockT,
	P: ProvideRuntimeApi<Block> + UsageProvider<Block>,
	P::Api: ContainerRuntimeApi<Block, AccountId32>,
	<Block::Header as HeaderT>::Number: Into<u32>,
{
	let new_best_heads = match new_best_heads(relay_chain.clone(), para_id).await {
		Ok(best_heads_stream) => best_heads_stream.fuse(),
		Err(_err) => {
			return;
		},
	};
	pin_mut!(new_best_heads);
	let runing_app = Arc::new(Mutex::new(RunningApp { group_id: 0 }));
	loop {
		select! {
			h = new_best_heads.next() => {
				match h {
					Some((height, head, hash)) => {
						let _ = handle_new_best_parachain_head(head,height, &*parachain,keystore.clone(), relay_chain.clone(), hash, para_id, data_path.clone(), runing_app.clone()).await;
					},
					None => {
						return;
					}
				}
			},
		}
	}
}

pub async fn run_container_task<P, R, Block>(
	para_id: ParaId,
	parachain: Arc<P>,
	relay_chain: R,
	keystore: KeystorePtr,
	data_path: PathBuf,
) where
	R: RelayChainInterface + Clone,
	Block: BlockT,
	P: ProvideRuntimeApi<Block> + UsageProvider<Block>,
	P::Api: ContainerRuntimeApi<Block, AccountId32>,
	<Block::Header as HeaderT>::Number: Into<u32>,
{
	let relay_chain_notification =
		relay_chain_notification(para_id, parachain.clone(), relay_chain, keystore, data_path);
	select! {
		_ = relay_chain_notification.fuse() => {},
	}
}

pub fn spawn_container_task<T, R, Block>(
	parachain: Arc<T>,
	para_id: ParaId,
	relay_chain: R,
	task_manager: &TaskManager,
	keystore: KeystorePtr,
	data_path: PathBuf,
) -> sc_service::error::Result<()>
where
	Block: BlockT,
	R: RelayChainInterface + Clone + 'static,
	T: Send + Sync + 'static + ProvideRuntimeApi<Block> + UsageProvider<Block>,
	T::Api: ContainerRuntimeApi<Block, AccountId32>,
	<Block::Header as HeaderT>::Number: Into<u32>,
{
	let container_task =
		run_container_task(para_id, parachain.clone(), relay_chain.clone(), keystore, data_path);
	task_manager
		.spawn_essential_handle()
		.spawn_blocking("container_task", None, container_task);
	Ok(())
}
