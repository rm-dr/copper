"use client";
import React, { useCallback, useMemo } from "react";
import {
	addEdge,
	Background,
	Controls,
	Handle,
	MiniMap,
	Position,
	ReactFlow,
	useEdgesState,
	useNodesState,
} from "@xyflow/react";

import style from "./pipeline.module.scss";
import "@xyflow/react/dist/style.css";

import "./text.module.scss";

const initialNodes = [
	{ id: "1", position: { x: 0, y: 0 }, data: { label: "1" } },
	{ id: "2", position: { x: 0, y: 100 }, data: { label: "2" } },
	{
		id: "node-1",
		type: "textUpdater",
		position: { x: 0, y: 0 },
		data: { value: 123 },
	},
];
const initialEdges = [{ id: "e1-2", source: "1", target: "2" }];

function TextUpdaterNode({ data }) {
	const onChange = useCallback((evt) => {
		console.log(evt.target.value);
	}, []);

	return (
		<>
			<Handle type="target" position={Position.Left} />
			<div>
				<label htmlFor="text">Text:</label>
				<input id="text" name="text" onChange={onChange} className="nodrag" />
			</div>
			<Handle type="source" position={Position.Right} id="a" />
			<Handle
				type="source"
				position={Position.Right}
				id="b"
				style={{ right: 5 }}
			/>
		</>
	);
}

export default function Page() {
	const [nodes, setNodes, onNodesChange] = useNodesState(initialNodes);
	const [edges, setEdges, onEdgesChange] = useEdgesState(initialEdges);

	const nodeTypes = useMemo(() => ({ textUpdater: TextUpdaterNode }), []);

	const onConnect = useCallback(
		(params) => setEdges((eds) => addEdge(params, eds)),
		[setEdges],
	);

	return (
		<div className={style.react_flow_container}>
			<ReactFlow
				colorMode="dark"
				nodes={nodes}
				edges={edges}
				onNodesChange={onNodesChange}
				onEdgesChange={onEdgesChange}
				onConnect={onConnect}
				nodeTypes={nodeTypes}
			>
				<Controls />
				<Background variant={"dots"} gap={12} size={1} />

				<MiniMap
					nodeStrokeColor={(n) => {
						if (n.type === "input") return "#0041d0";
						if (n.type === "output") return "#ff0072";
						if (n.type === "textUpdater") return "#ff0072";
					}}
					nodeColor={(n) => {
						if (n.type === "textUpdater") return null;
						return "#fff";
					}}
				/>
			</ReactFlow>
		</div>
	);
}
