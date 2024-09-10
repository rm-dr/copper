use super::{PipelineData, PipelineJobContext, PortName, ProcessSignalError, RunNodeError};

/// The state of a [`PipelineNode`] at a point in time.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NodeState {
	/// This node has more work to do
	/// and is waiting to be `run()`.
	///
	/// This status always comes with a message, telling us
	/// why this node isn't `Done` yet.
	Pending(&'static str),

	/// This node has output all its data
	/// and will not be run again.
	Done,
}

impl NodeState {
	/// Is this [`NodeState::Pending`]?
	pub fn is_pending(&self) -> bool {
		matches!(self, Self::Pending(_))
	}

	/// Is this [`NodeState::Done`]?
	pub fn is_done(&self) -> bool {
		matches!(self, Self::Done)
	}
}

pub enum NodeSignal<DataType: PipelineData> {
	/// This signal is sent once for each edge that is connected to
	/// an input port of this node. All instances of `ConnectPort` should
	/// happen before `Input` or `DisconnectPort` are sent or `run` is called.
	///
	/// This signal allows nodes to detect disconnected inputs and
	/// set state appropriately. Any port that has not been connected
	/// before a call to `take_input` of `run` should be considered disconnected.
	///
	/// `ConnectInput` should never be received with the same port twice,
	/// and should cause an `unreachable!` panic.
	ConnectInput { port: PortName },

	/// This signal is sent an input edge connected to this node
	/// has it's source node finish. All instances of `ConnectPort` should
	/// happen after all instances of `ConnectPort` are received
	///
	/// This signal allows nodes to detect disconnected inputs
	/// not connected to any other node and set state appropriately.
	///
	/// If a required input is disconnected before receiving data, nodes should
	/// throw a [`ProcessSignalError::InputNotConnected`]. If an optional input
	/// is disconnected before receiving data, nodes should automatically set it
	/// to a default value. The exact implementation of this depends on the node.
	///
	/// `ConnectInput` should never be received with the same port twice,
	/// and should never be received with a port that hasn't yet been connected.
	/// Both should cause an `unreachable!` panic.
	DisconnectInput { port: PortName },

	/// Receive input on the specified port
	///
	/// `ReceiveInput` should never be received with a port that hasn't yet been connected.
	/// This should cause an `unreachable!` panic.
	ReceiveInput { port: PortName, data: DataType },
}

pub trait Node<DataType: PipelineData, ContextType: PipelineJobContext<DataType>>:
	Sync + Send
{
	/// If true, run this node in the main loop instead of starting a thread.
	///
	/// This should be `true` for nodes that do no heavy computation, and
	/// `false` for everything else. If this is true, `run` will block the
	/// async event loop, and thus cannot take a long time to run.
	fn quick_run(&self) -> bool {
		false
	}

	fn process_signal(
		&mut self,
		ctx: &ContextType,
		signal: NodeSignal<DataType>,
	) -> Result<(), ProcessSignalError>;

	/// Run this node.
	/// This is always run in a worker thread.
	fn run(
		&mut self,
		ctx: &ContextType,
		send_data: &dyn Fn(PortName, DataType) -> Result<(), RunNodeError>,
	) -> Result<NodeState, RunNodeError>;
}
