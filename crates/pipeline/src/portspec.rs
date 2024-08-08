use crate::{api::PipelineDataStub, labels::PipelinePortLabel};

/// Name and datatype for a set of ports.
#[derive(Debug, Clone)]
pub enum PipelinePortSpec<'a, DataStub: PipelineDataStub + 'static> {
	// TODO: make `&'static str` a `PipelinePortLabel` once we can
	// statically make SmartStrings.
	Static(&'static [(&'static str, DataStub)]),
	Vec(&'a Vec<(PipelinePortLabel, DataStub)>),
	VecOwned(Vec<(PipelinePortLabel, DataStub)>),
}

impl<'a, DataStub: PipelineDataStub> PipelinePortSpec<'a, DataStub> {
	pub fn len(&self) -> usize {
		match self {
			Self::Static(x) => x.len(),
			Self::Vec(x) => x.len(),
			Self::VecOwned(x) => x.len(),
		}
	}

	pub fn is_empty(&self) -> bool {
		self.len() == 0
	}

	pub fn find_with_name(&self, name: &PipelinePortLabel) -> Option<(usize, DataStub)> {
		match self {
			Self::Static(x) => x
				.iter()
				.enumerate()
				.find(|(_, (l, _))| *l == Into::<&str>::into(name))
				.map(|(i, (_, t))| (i, *t)),
			Self::Vec(x) => x
				.iter()
				.enumerate()
				.find(|(_, (l, _))| l == name)
				.map(|(i, (_, t))| (i, *t)),
			Self::VecOwned(x) => x
				.iter()
				.enumerate()
				.find(|(_, (l, _))| l == name)
				.map(|(i, (_, t))| (i, *t)),
		}
	}

	pub fn iter(&self) -> PipelineArgSpecIterator<DataStub> {
		match self {
			Self::Static(data) => PipelineArgSpecIterator::Static { data, idx: 0 },
			Self::Vec(data) => PipelineArgSpecIterator::Vec { data, idx: 0 },
			Self::VecOwned(data) => PipelineArgSpecIterator::Vec { data, idx: 0 },
		}
	}

	pub fn to_vec(&self) -> Vec<(PipelinePortLabel, DataStub)> {
		self.iter().collect()
	}
}

pub enum PipelineArgSpecIterator<'a, DataStub: PipelineDataStub + 'static> {
	Static {
		data: &'static [(&'static str, DataStub)],
		idx: usize,
	},
	Vec {
		data: &'a Vec<(PipelinePortLabel, DataStub)>,
		idx: usize,
	},
}

impl<'a, DataStub: PipelineDataStub> Iterator for PipelineArgSpecIterator<'a, DataStub> {
	type Item = (PipelinePortLabel, DataStub);

	fn next(&mut self) -> Option<Self::Item> {
		match self {
			Self::Static { data, idx } => {
				if *idx >= data.len() {
					None
				} else {
					let d = Some((data[*idx].0.into(), data[*idx].1));
					*idx += 1;
					d
				}
			}
			Self::Vec { data, idx } => {
				if *idx >= data.len() {
					None
				} else {
					let d = Some((data[*idx].0.clone(), data[*idx].1));
					*idx += 1;
					d
				}
			}
		}
	}
}

impl<'a, DataStub: PipelineDataStub> ExactSizeIterator for PipelineArgSpecIterator<'a, DataStub> {
	fn len(&self) -> usize {
		match self {
			Self::Static { data, .. } => data.len(),
			Self::Vec { data, .. } => data.len(),
		}
	}
}
