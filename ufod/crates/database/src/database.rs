use std::{
	path::{Path, PathBuf},
	sync::Arc,
};

use ufo_db_blobstore::{api::Blobstore, fs::store::FsBlobstore};
use ufo_db_metastore::{api::Metastore, errors::MetastoreError, sqlite::db::SQLiteMetastore};
use ufo_db_pipestore::{api::Pipestore, fs::FsPipestore};

use crate::api::UFODatabase;

pub struct Database<BlobstoreType: Blobstore, MetastoreType: Metastore, PipestoreType: Pipestore> {
	blobstore: Arc<BlobstoreType>,
	metastore: Arc<MetastoreType>,
	pipestore: Arc<PipestoreType>,
}

impl<
		BlobStoreType: Blobstore + 'static,
		MetaDbType: Metastore + 'static,
		PipestoreType: Pipestore + 'static,
	> UFODatabase for Database<BlobStoreType, MetaDbType, PipestoreType>
{
	fn get_metastore(&self) -> Arc<dyn Metastore> {
		self.metastore.clone()
	}

	fn get_blobstore(&self) -> Arc<dyn Blobstore> {
		self.blobstore.clone()
	}

	fn get_pipestore(&self) -> Arc<dyn Pipestore> {
		self.pipestore.clone()
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
			metastore: Arc::new(metastore),
			blobstore: Arc::new(blobstore),
			pipestore: Arc::new(pipestore),
		})
	}
}
