use std::{
	fs::File,
	io::Write,
	path::{Path, PathBuf},
	sync::Mutex,
};

use futures::executor::block_on;
use smartstring::{LazyCompact, SmartString};
use sqlx::{Connection, Row, SqliteConnection};
use ufo_util::mime::MimeType;

use super::super::api::{BlobHandle, BlobStore};

pub struct FsBlobStoreCreateParams {
	pub root_dir: PathBuf,
}

#[derive(Debug, Clone)]
pub struct FsBlobHandle {
	name: SmartString<LazyCompact>,
	mime: MimeType,
}

impl BlobHandle for FsBlobHandle {
	fn to_db_str(&self) -> String {
		self.name.to_string()
	}

	fn from_db_str(s: &str) -> Self {
		Self {
			name: s.into(),
			mime: MimeType::Blob,
		}
	}

	fn get_type(&self) -> &MimeType {
		&self.mime
	}
}

pub struct FsBlobWriter {
	file: Option<File>,
	handle: FsBlobHandle,

	// Used for cleanup
	relative_path: PathBuf,
	blob_storage_root: PathBuf,
	is_finished: bool,
}

impl Write for FsBlobWriter {
	fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
		self.file.as_mut().unwrap().write(buf)
	}

	fn flush(&mut self) -> std::io::Result<()> {
		self.file.as_mut().unwrap().flush()
	}
}

// TODO: tmp dir
// TODO: test this
impl Drop for FsBlobWriter {
	fn drop(&mut self) {
		self.file.take().unwrap().flush().unwrap();

		// If we never finished this writer, delete the file.
		if !self.is_finished {
			std::fs::remove_file(self.blob_storage_root.join(&self.relative_path)).unwrap();
		}
	}
}

pub struct FsBlobStore {
	root: PathBuf,
	idx: Mutex<u32>,
	conn: Mutex<SqliteConnection>,
}

unsafe impl Send for FsBlobStore {}

impl BlobStore for FsBlobStore {
	type Handle = FsBlobHandle;
	type Writer = FsBlobWriter;
	type CreateParams = FsBlobStoreCreateParams;

	fn create(
		db_root_dir: &Path,
		blob_db_name: &str,
		params: FsBlobStoreCreateParams,
	) -> Result<(), ()> {
		if params.root_dir.exists() {
			return Err(());
		}
		std::fs::create_dir(db_root_dir.join(&params.root_dir)).unwrap();

		let database = db_root_dir.join(blob_db_name);
		let db_addr = format!("sqlite:{}?mode=rwc", database.to_str().unwrap());
		let mut conn = block_on(SqliteConnection::connect(&db_addr)).unwrap();

		block_on(sqlx::query(include_str!("./init.sql")).execute(&mut conn)).unwrap();
		block_on(
			sqlx::query("INSERT INTO meta (var, val) VALUES (?,?), (?,?), (?,?);")
				.bind("ufo_version")
				.bind(env!("CARGO_PKG_VERSION"))
				.bind("idx_counter")
				.bind(0)
				.bind("blob_dir")
				.bind(params.root_dir.to_str().unwrap())
				.execute(&mut conn),
		)
		.unwrap();

		Ok(())
	}

	fn open(db_root_dir: &Path, blob_db_name: &str) -> Result<Self, ()> {
		let database = db_root_dir.join(blob_db_name);
		let db_addr = format!("sqlite:{}?mode=rwc", database.to_str().unwrap());
		let mut conn = block_on(SqliteConnection::connect(&db_addr)).unwrap();

		let idx_counter: u32 = {
			let res = block_on(
				sqlx::query("SELECT val FROM meta WHERE var=\"idx_counter\";").fetch_one(&mut conn),
			)
			.unwrap();
			res.get::<String, _>("val").parse().unwrap()
		};

		let blob_dir = db_root_dir.join({
			let res = block_on(
				sqlx::query("SELECT val FROM meta WHERE var=\"blob_dir\";").fetch_one(&mut conn),
			)
			.unwrap();
			res.get::<String, _>("val")
		});

		Ok(Self {
			root: blob_dir,
			idx: Mutex::new(idx_counter),
			conn: Mutex::new(conn),
		})
	}

	fn new_blob(&mut self, mime: &MimeType) -> Self::Writer {
		let mut li = self.idx.lock().unwrap();
		let i = *li;
		*li += 1;

		block_on(
			sqlx::query("UPDATE meta SET val=? WHERE var=\"idx_counter\";")
				.bind(*li)
				.execute(&mut *self.conn.lock().unwrap()),
		)
		.unwrap();

		let blob_storage_root = self.root.clone();
		let relative_path = format!("{i}{}", mime.extension()).into();
		let f = File::create(blob_storage_root.join(&relative_path)).unwrap();

		FsBlobWriter {
			file: Some(f),
			handle: FsBlobHandle {
				name: format!("{i}").into(),
				mime: mime.clone(),
			},
			is_finished: false,

			blob_storage_root,
			relative_path,
		}
	}

	fn finish_blob(&mut self, mut blob: Self::Writer) -> FsBlobHandle {
		block_on(
			sqlx::query("INSERT INTO blobs (data_type, file_path) VALUES (?, ?);")
				.bind(blob.handle.mime.to_db_str())
				.bind(blob.relative_path.to_str().unwrap())
				.execute(&mut *self.conn.lock().unwrap()),
		)
		.unwrap();

		blob.is_finished = true;
		blob.handle.clone()
	}
}
