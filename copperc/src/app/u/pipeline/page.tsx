"use client";
import React, { useCallback, useRef, useState } from "react";
import {
	addEdge,
	applyEdgeChanges,
	applyNodeChanges,
	Background,
	Controls,
	MiniMap,
	Panel,
	ReactFlow,
	ReactFlowInstance,
	ReactFlowProvider,
	type Node,
	type Edge,
	type OnConnect,
	type OnNodesChange,
	type OnEdgesChange,
	type OnReconnect,
	BackgroundVariant,
	ConnectionMode,
	ConnectionLineType,
	reconnectEdge,
} from "@xyflow/react";

import style from "./pipeline.module.scss";
import "@xyflow/react/dist/style.css";

import { nodeTypes } from "./_nodes";

const initialNodes: Node[] = [
	{
		id: "node-1",
		type: "constant",
		position: { x: 0, y: 0 },
		data: { value: 123 },
	},

	{
		id: "node-2",
		type: "ifnone",
		position: { x: 0, y: 0 },
		data: { value: 123 },
	},

	{
		id: "node-3",
		type: "hash",
		position: { x: 0, y: 0 },
		data: { value: 123 },
	},
];
const initialEdges: Edge[] = [
	{
		type: "smoothstep",
		id: "e1-2",
		source: "node-2",
		target: "node-3",
	},
];

export default function Page() {
	return (
		<div className={style.react_flow_container}>
			<ReactFlowProvider>
				<Flow />
			</ReactFlowProvider>
		</div>
	);
}

function Flow() {
	const [nodes, setNodes] = useState<Node[]>(initialNodes);
	const [edges, setEdges] = useState<Edge[]>(initialEdges);
	// const { setViewport } = useReactFlow();
	const [rfInstance, setRfInstance] = useState<null | ReactFlowInstance>(null);
	const edgeReconnectSuccessful = useRef(true);

	const onReconnectStart = useCallback(() => {
		edgeReconnectSuccessful.current = false;
	}, []);

	const onReconnect: OnReconnect = useCallback((oldEdge, newConnection) => {
		edgeReconnectSuccessful.current = true;
		setEdges((els) => reconnectEdge(oldEdge, newConnection, els));
	}, []);

	// eslint-disable-next-line
	const onReconnectEnd = useCallback((_: unknown, edge: any) => {
		if (!edgeReconnectSuccessful.current) {
			setEdges((eds) => eds.filter((e) => e.id !== edge.id));
		}

		edgeReconnectSuccessful.current = true;
	}, []);

	const onNodesChange: OnNodesChange = useCallback(
		(changes) => setNodes((nds) => applyNodeChanges(changes, nds)),
		[setNodes],
	);
	const onEdgesChange: OnEdgesChange = useCallback(
		(changes) => setEdges((eds) => applyEdgeChanges(changes, eds)),
		[setEdges],
	);
	const onConnect: OnConnect = useCallback(
		(connection) => setEdges((eds) => addEdge(connection, eds)),
		[setEdges],
	);

	const onSave = useCallback(() => {
		if (rfInstance) {
			const flow = rfInstance.toObject();
			console.log(JSON.stringify(flow));
		}
	}, [rfInstance]);

	const onRestore = useCallback(() => {
		const restoreFlow = async () => {
			/*
			const flow = JSON.parse(localStorage.getItem(flowKey));

			if (flow) {
				const { x = 0, y = 0, zoom = 1 } = flow.viewport;
				setNodes(flow.nodes || []);
				setEdges(flow.edges || []);
				setViewport({ x, y, zoom });
			}
			*/
		};

		restoreFlow();
	}, []);

	return (
		<ReactFlow
			className={style.react_flow}
			nodes={nodes}
			edges={edges}
			onNodesChange={onNodesChange}
			onEdgesChange={onEdgesChange}
			onInit={setRfInstance}
			onConnect={onConnect}
			nodeTypes={nodeTypes}
			defaultEdgeOptions={{ type: "smoothstep" }}
			connectionMode={ConnectionMode.Strict}
			connectionLineType={ConnectionLineType.SmoothStep}
			snapToGrid
			onReconnect={onReconnect}
			onReconnectStart={onReconnectStart}
			onReconnectEnd={onReconnectEnd}
		>
			<Controls />
			<Background
				variant={BackgroundVariant.Dots}
				gap={12}
				size={1}
				color="var(--mantine-color-dark-3)"
			/>

			<Panel position="top-right">
				<button onClick={onSave}>save</button>
				<button onClick={onRestore}>restore</button>
			</Panel>

			<MiniMap />
		</ReactFlow>
	);
}
