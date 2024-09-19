import { ConstantNode } from "./constant";
import { HashNode } from "./hash";
import { IfNoneNode } from "./ifnone";

export const nodeTypes = {
	constant: ConstantNode,
	ifnone: IfNoneNode,
	hash: HashNode,
} as const;
