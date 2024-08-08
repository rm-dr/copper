use std::path::{Path, PathBuf};

use crate::{
	api::UFODatabase,
	blobstore::{
		api::{BlobHandle, Blobstore, BlobstoreTmpWriter},
		fs::store::FsBlobstore,
	},
	metastore::{api::Metastore, errors::MetastoreError, sqlite::db::SQLiteMetastore},
};

pub struct Database<BlobstoreType: Blobstore, MetastoreType: Metastore> {
	blobstore: BlobstoreType,
	metastore: MetastoreType,
}

unsafe impl<BlobStoreType: Blobstore, MetaDbType: Metastore> Send
	for Database<BlobStoreType, MetaDbType>
{
}

unsafe impl<BlobStoreType: Blobstore, MetaDbType: Metastore> Sync
	for Database<BlobStoreType, MetaDbType>
{
}

impl<BlobStoreType: Blobstore, MetaDbType: Metastore> UFODatabase
	for Database<BlobStoreType, MetaDbType>
{
	fn get_metastore(&mut self) -> &mut dyn Metastore {
		&mut self.metastore
	}

	fn new_blob(&mut self, mime: &ufo_util::mime::MimeType) -> BlobstoreTmpWriter {
		self.blobstore.new_blob(mime)
	}

	fn finish_blob(&mut self, blob: BlobstoreTmpWriter) -> BlobHandle {
		self.blobstore.finish_blob(blob)
	}
}

impl Database<FsBlobstore, SQLiteMetastore> {
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

		SQLiteMetastore::create(&db_root.join("metadata.sqlite")).unwrap();

		std::fs::create_dir(db_root.join("pipelines")).unwrap();

		Ok(())
	}

	pub fn open(db_root: &Path) -> Result<Self, ()> {
		let blobstore = FsBlobstore::open(db_root, "blobstore.sqlite").unwrap();
		let metastore = SQLiteMetastore::open(&db_root.join("metadata.sqlite")).unwrap();

		Ok(Self {
			metastore,
			blobstore,
		})
	}
}
