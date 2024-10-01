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
};

// eslint-disable-next-line
export const nodes: Record<string, NodeDef<any>> = {
	[InputNode.key]: InputNode,
	[ConstantNode.key]: ConstantNode,
	[IfNoneNode.key]: IfNoneNode,
	[HashNode.key]: HashNode,

	[StripTagsNode.key]: StripTagsNode,
	[ExtractCoversNode.key]: ExtractCoversNode,
	[ExtractTagsNode.key]: ExtractTagsNode,

	[AddItemNode.key]: AddItemNode,
} as const;

export const nodeTypes = Object.keys(nodes).reduce((res, key) => {
	res[key] = nodes[key as keyof typeof nodes]!.node;
	return res;
	// eslint-disable-next-line
}, {} as Record<string, NodeDef<any>["node"]>) as NodeTypes;
