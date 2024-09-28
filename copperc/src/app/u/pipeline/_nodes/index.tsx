import { AddItemNode } from "./additem";
import { ConstantNode } from "./constant";
import { ExtractCoversNode } from "./extractcovers";
import { ExtractTagsNode } from "./extracttags";
import { HashNode } from "./hash";
import { IfNoneNode } from "./ifnone";
import { InputNode } from "./input";
import { StripTagsNode } from "./striptags";

export const nodeTypes = {
	// The "input" node class is already taken
	pipelineinput: InputNode,
	constant: ConstantNode,
	ifnone: IfNoneNode,
	hash: HashNode,

	striptags: StripTagsNode,
	extractcovers: ExtractCoversNode,
	extracttags: ExtractTagsNode,

	additem: AddItemNode,
} as const;
