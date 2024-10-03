import { components } from "@/lib/api/openapi";
import { AddItemNode } from "./additem";
import { ConstantNode } from "./constant";
import { ExtractCoversNode } from "./extractcovers";
import { ExtractTagsNode } from "./extracttags";
import { HashNode } from "./hash";
import { IfNoneNode } from "./ifnone";
import { InputNode } from "./input";
import { StripTagsNode } from "./striptags";
import { Node, NodeProps, NodeTypes } from "@xyflow/react";
import { attrTypes } from "@/lib/attributes";

export type NodeDef<NodeType extends Node> = {
	xyflow_node_type: string;
	copper_node_type: string;

	minimap_color?: string;

	node: (props: NodeProps<NodeType>) => JSX.Element;
	initialData: NodeType["data"];

	getInputs: (data: NodeType["data"]) => {
		id: string;
		type: (typeof attrTypes)[number]["serialize_as"];
	}[];

	getOutputs: (data: NodeType["data"]) => {
		id: string;
		type: (typeof attrTypes)[number]["serialize_as"];
	}[];

	/**
	 * Transform this `ReactFlow` node into the parameters of a `PipelineJson` node.
	 */
	serialize: (
		node: NodeType,
	) => components["schemas"]["PipelineJson"]["nodes"][string]["params"] | null;

	/**
	 * Transform a `PipelineJson` node into a `ReactFlow` node's data object.
	 * This _only_ produces the data object. All other fields are filled automatically.
	 */
	deserialize: (
		serialized: components["schemas"]["PipelineJson"]["nodes"][string],
	) => NodeType["data"] | null;
};

// eslint-disable-next-line
export const nodeDefinitions: Record<string, NodeDef<any>> = {
	[InputNode.xyflow_node_type]: InputNode,
	[ConstantNode.xyflow_node_type]: ConstantNode,
	[IfNoneNode.xyflow_node_type]: IfNoneNode,
	[HashNode.xyflow_node_type]: HashNode,

	[StripTagsNode.xyflow_node_type]: StripTagsNode,
	[ExtractCoversNode.xyflow_node_type]: ExtractCoversNode,
	[ExtractTagsNode.xyflow_node_type]: ExtractTagsNode,

	[AddItemNode.xyflow_node_type]: AddItemNode,
} as const;

export const nodeTypes = Object.keys(nodeDefinitions).reduce((res, key) => {
	res[key] = nodeDefinitions[key as keyof typeof nodeDefinitions]!.node;
	return res;
	// eslint-disable-next-line
}, {} as Record<string, NodeDef<any>["node"]>) as NodeTypes;
