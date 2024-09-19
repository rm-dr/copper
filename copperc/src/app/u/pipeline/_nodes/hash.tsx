import style from "./nodes.module.scss";
import { Select } from "@mantine/core";
import { Handle, Node, NodeProps, Position } from "@xyflow/react";
import { useState } from "react";
import { BaseNode } from "./base";

const HashTypes = ["SHA256", "SHA512", "MD5"] as const;

type HashNodeType = Node<Record<string, never>, "hash">;

export function HashNode({ id }: NodeProps<HashNodeType>) {
	const [hashType, setHashType] =
		useState<(typeof HashTypes)[number]>("SHA256");

	return (
		<>
			<Handle
				className={style.node_handle}
				type="target"
				position={Position.Left}
				id="data"
			/>

			<BaseNode id={id} title={"Checksum"}>
				<Select
					label="Hash type"
					data={HashTypes}
					onChange={(value) => {
						// eslint-disable-next-line
						if (value !== null && HashTypes.includes(value as any)) {
							setHashType(value as (typeof HashTypes)[number]);
						}
					}}
					value={hashType}
				/>
			</BaseNode>

			<Handle
				className={style.node_handle}
				type="source"
				position={Position.Right}
				id="hash"
			/>
		</>
	);
}
