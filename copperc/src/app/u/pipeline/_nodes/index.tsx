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

export type NodeDef<NodeType extends Node> = {
	key: string;
	node: (props: NodeProps<NodeType>) => JSX.Element;
	initialData: NodeType["data"];

	// Returns null if error. TODO: provide a message
	export: (
		node: NodeType,
	) => components["schemas"]["PipelineJson"]["nodes"][string] | null;
};

// eslint-disable-next-line
export const nodeDefinitions: Record<string, NodeDef<any>> = {
	[InputNode.key]: InputNode,
	[ConstantNode.key]: ConstantNode,
	[IfNoneNode.key]: IfNoneNode,
	[HashNode.key]: HashNode,

	[StripTagsNode.key]: StripTagsNode,
	[ExtractCoversNode.key]: ExtractCoversNode,
	[ExtractTagsNode.key]: ExtractTagsNode,

	[AddItemNode.key]: AddItemNode,
} as const;

export const nodeTypes = Object.keys(nodeDefinitions).reduce((res, key) => {
	res[key] = nodeDefinitions[key as keyof typeof nodeDefinitions]!.node;
	return res;
	// eslint-disable-next-line
}, {} as Record<string, NodeDef<any>["node"]>) as NodeTypes;
