use std::path::{Path, PathBuf};

use crate::{
	api::UFODatabase,
	blobstore::{
		api::{BlobHandle, Blobstore, BlobstoreTmpWriter},
		fs::store::FsBlobstore,
	},
	metastore::{api::Metastore, errors::MetastoreError, sqlite::db::SQLiteMetastore},
	pipestore::{api::Pipestore, fs::FsPipestore},
};

pub struct Database<BlobstoreType: Blobstore, MetastoreType: Metastore, PipestoreType: Pipestore> {
	blobstore: BlobstoreType,
	metastore: MetastoreType,
	pipestore: PipestoreType,
}

// TODO: modular locks
unsafe impl<BlobStoreType: Blobstore, MetaDbType: Metastore, PipestoreType: Pipestore> Send
	for Database<BlobStoreType, MetaDbType, PipestoreType>
{
}

unsafe impl<BlobStoreType: Blobstore, MetaDbType: Metastore, PipestoreType: Pipestore> Sync
	for Database<BlobStoreType, MetaDbType, PipestoreType>
{
}

impl<BlobStoreType: Blobstore, MetaDbType: Metastore, PipestoreType: Pipestore> UFODatabase
	for Database<BlobStoreType, MetaDbType, PipestoreType>
{
	fn get_metastore(&mut self) -> &mut dyn Metastore {
		&mut self.metastore
	}

	fn get_pipestore(&self) -> &dyn Pipestore {
		&self.pipestore
	}

	fn new_blob(&mut self, mime: &ufo_util::mime::MimeType) -> BlobstoreTmpWriter {
		self.blobstore.new_blob(mime)
	}

	fn finish_blob(&mut self, blob: BlobstoreTmpWriter) -> BlobHandle {
		self.blobstore.finish_blob(blob)
	}
}

impl Database<FsBlobstore, SQLiteMetastore, FsPipestore> {
	pub fn create(db_root: &Path) -> Result<(), MetastoreError> {
		// `db_root` must exist and be empty
		if db_root.is_dir() {
			if db_root.read_dir().unwrap().next().is_some() {
				panic!("TODO: proper error")
			}
		} else if db_root.exists() {
			panic!()
		}

		// TODO: make sure all dirs exist, create if not
		//let pipeline_dir = db_root.join("pipelines");
		//std::fs::create_dir(&pipeline_dir).unwrap();

		/*

				block_on(
			sqlx::query("INSERT INTO meta_meta (var, val) VALUES (?, ?);")
				.bind("ufo_version")
				.bind(env!("CARGO_PKG_VERSION"))
				.execute(&mut conn),
		)
		.unwrap();
		 */

		// Make blobstore
		FsBlobstore::create(
			db_root,
			&PathBuf::from("blobstore.sqlite"),
			&PathBuf::from("blobstore"),
		)
		.unwrap();

		// Make metadata store
		SQLiteMetastore::create(&db_root.join("metadata.sqlite")).unwrap();

		// Make pipeline store
		FsPipestore::create(&db_root.join("pipelines")).unwrap();

		Ok(())
	}

	pub fn open(db_root: &Path) -> Result<Self, ()> {
		let blobstore = FsBlobstore::open(db_root, "blobstore.sqlite").unwrap();
		let metastore = SQLiteMetastore::open(&db_root.join("metadata.sqlite")).unwrap();
		let pipestore = FsPipestore::open(&db_root.join("pipelines")).unwrap();

		Ok(Self {
			metastore,
			blobstore,
			pipestore,
		})
	}
}
