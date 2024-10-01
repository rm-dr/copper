import { Select } from "@mantine/core";
import { Node, NodeProps } from "@xyflow/react";
import { useState } from "react";
import { BaseNode } from "./base";

const HashTypes = ["SHA256", "SHA512", "MD5"] as const;

type HashNodeType = Node<Record<string, never>, "hash">;

export function HashNode({ id }: NodeProps<HashNodeType>) {
	const [hashType, setHashType] =
		useState<(typeof HashTypes)[number]>("SHA256");

	return (
		<>
			<BaseNode
				id={id}
				title={"Checksum"}
				inputs={[{ id: "data", type: "Blob", tooltip: "Input data" }]}
				outputs={[{ id: "hash", type: "Hash", tooltip: "Checksum" }]}
			>
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
		</>
	);
}
