import nodestyle from "./_nodes/nodes.module.scss";

import { components } from "@/lib/api/openapi";
import { Edge, Node, ReactFlowJsonObject } from "@xyflow/react";
import { nodeDefinitions } from "./_nodes";

/**
 * Transform a reactflow pipeline into a `PipelineJson`
 * we can send to the backend server.
 *
 * @returns `{ result: "ok", ... }` if we successfully serialized the given pipeline,
 * `{result: "error", ... }` if we encountered an error.
 */
export function serializePipeline(
	pipeline: ReactFlowJsonObject<Node, Edge>,
):
	| { result: "ok"; value: components["schemas"]["PipelineJson"] }
	| { result: "error"; message: string } {
	const nodes: components["schemas"]["PipelineJson"]["nodes"] = {};
	pipeline.nodes.forEach((node) => {
		if (node.type === undefined) {
			return;
		}

		const nodedef = nodeDefinitions[node.type];
		if (nodedef === undefined) {
			return;
		}

		const ex = nodedef.serialize(node);
		if (ex === null) {
			return;
		}

		nodes[node.id] = {
			node_type: nodedef.copper_node_type,
			position: node.position,
			params: ex,
		};
	});

	const edges: components["schemas"]["PipelineJson"]["edges"] = {};
	pipeline.edges.forEach((edge) => {
		let sourcePort = edge.sourceHandle;
		if (sourcePort === null || sourcePort === undefined) {
			const node = pipeline.nodes.find((x) => x.id === edge.source);
			if (node === undefined) {
				return {
					result: "error",
					message: `Could not find node ${edge.source} (referenced by edge ${edge.id})`,
				};
			}

			if (node.handles === undefined || node.handles.length === 0) {
				return {
					result: "error",
					message: `Node ${edge.source} has no handles, but is connected to an edge ${edge.id}`,
				};
			}

			const firsthandle = node.handles[0];
			if (firsthandle === undefined || node.handles.length !== 1) {
				return {
					result: "error",
					message: `Edge ${edge.id} does not give an explicit handle for node ${edge.source}, but that node has multiple edges.`,
				};
			}

			sourcePort = firsthandle.id;

			if (sourcePort === null || sourcePort === undefined) {
				return {
					result: "error",
					message: `Handle of ${edge.id} on ${edge.source} doesn't have an id.`,
				};
			}
		}
		const source: components["schemas"]["InputPort"] = {
			node: edge.source,
			port: sourcePort,
		};

		let targetPort = edge.targetHandle;
		if (targetPort === null || targetPort === undefined) {
			const node = pipeline.nodes.find((x) => x.id === edge.target);
			if (node === undefined) {
				return {
					result: "error",
					message: `Could not find node ${edge.target} (referenced by edge ${edge.id})`,
				};
			}

			if (node.handles === undefined || node.handles.length === 0) {
				return {
					result: "error",
					message: `Node ${edge.target} has no handles, but is connected to an edge ${edge.id}`,
				};
			}

			const firsthandle = node.handles[0];
			if (firsthandle === undefined || node.handles.length !== 1) {
				return {
					result: "error",
					message: `Edge ${edge.id} does not give an explicit handle for node ${edge.target}, but that node has multiple edges.`,
				};
			}

			targetPort = firsthandle.id;

			if (targetPort === null || targetPort === undefined) {
				return {
					result: "error",
					message: `Handle of ${edge.id} on ${edge.target} doesn't have an id.`,
				};
			}
		}
		const target: components["schemas"]["OutputPort"] = {
			node: edge.target,
			port: targetPort,
		};

		edges[edge.id] = {
			source,
			target,
		};
	});

	return {
		result: "ok",
		value: { nodes, edges },
	};
}

export async function deserializePipeline(
	pipeline: components["schemas"]["PipelineJson"],
): Promise<{ nodes: Node[]; edges: Edge[] }> {
	const nodes = [];

	for (const x of Object.entries(pipeline.nodes)) {
		const v = x[1];

		const nodedef = Object.entries(nodeDefinitions).find(
			(y) => y[1].copper_node_type === v.node_type,
		);
		if (nodedef === undefined) {
			console.error(`Unknown node type ${v.node_type}`);
			throw new Error(`Unknown node type ${v.node_type}`);
		}

		const des = await nodedef[1].deserialize(v);
		if (des === null) {
			continue;
		}

		const node: Node = {
			id: x[0],
			type: nodedef[1].xyflow_node_type,
			position: v.position,
			data: des,
			origin: [0.5, 0.0],
			dragHandle: `.${nodestyle.node_top_label}`,
		};

		nodes.push(node);
	}

	const edges = Object.entries(pipeline.edges).map((x) => {
		const v = x[1];
		const edge: Edge = {
			type: "default",
			id: x[0],
			source: v.source.node,
			sourceHandle: v.source.port,
			target: v.target.node,
			targetHandle: v.target.port,
		};

		return edge;
	});

	return { nodes, edges };
}
