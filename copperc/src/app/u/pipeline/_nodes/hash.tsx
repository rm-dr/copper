import style from "./nodes.module.scss";
import { Select } from "@mantine/core";
import { Handle, Position } from "@xyflow/react";
import { useState } from "react";

export function HashNode({}) {
	const HashTypes = ["SHA256", "SHA512", "MD5"] as const;

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

			<div className={style.node_body}>
				<div className={style.node_label}>
					<label>Checksum</label>
				</div>
				<div className={style.node_content}>
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
				</div>
			</div>

			<Handle
				className={style.node_handle}
				type="source"
				position={Position.Right}
				id="hash"
			/>
		</>
	);
}
