use ufo_util::data::PipelineDataType;

use crate::syntax::labels::PipelinePortLabel;

/// Name and datatype for a set of ports.
#[derive(Debug, Clone, Copy)]
pub enum PipelinePortSpec<'a> {
	// TODO: make `&'static str` a `PipelinePortLabel` once we can
	// statically make SmartStrings.
	Static(&'static [(&'static str, PipelineDataType)]),
	Vec(&'a Vec<(PipelinePortLabel, PipelineDataType)>),
}

impl<'a> PipelinePortSpec<'a> {
	pub fn len(&self) -> usize {
		match self {
			Self::Static(x) => x.len(),
			Self::Vec(x) => x.len(),
		}
	}

	pub fn find_with_name(&self, name: &PipelinePortLabel) -> Option<(usize, PipelineDataType)> {
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
		}
	}

	pub fn iter(&self) -> PipelineArgSpecIterator {
		match self {
			Self::Static(data) => PipelineArgSpecIterator::Static { data, idx: 0 },
			Self::Vec(data) => PipelineArgSpecIterator::Vec { data, idx: 0 },
		}
	}
}

pub enum PipelineArgSpecIterator<'a> {
	Static {
		data: &'static [(&'static str, PipelineDataType)],
		idx: usize,
	},
	Vec {
		data: &'a Vec<(PipelinePortLabel, PipelineDataType)>,
		idx: usize,
	},
}

impl<'a> Iterator for PipelineArgSpecIterator<'a> {
	type Item = (PipelinePortLabel, PipelineDataType);

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

impl<'a> ExactSizeIterator for PipelineArgSpecIterator<'a> {
	fn len(&self) -> usize {
		match self {
			Self::Static { data, .. } => data.len(),
			Self::Vec { data, .. } => data.len(),
		}
	}
}
