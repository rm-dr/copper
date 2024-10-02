"use client";
import React, { useCallback, useRef, useState } from "react";
import {
	addEdge,
	applyEdgeChanges,
	applyNodeChanges,
	Background,
	Controls,
	MiniMap,
	ReactFlow,
	ReactFlowInstance,
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

import style from "./flow.module.scss";
import "@xyflow/react/dist/style.css";

import { nodeTypes } from "./_nodes";

export function useFlow() {
	const [nodes, setNodes] = useState<Node[]>([]);
	const [edges, setEdges] = useState<Edge[]>([]);
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

	return {
		getFlow: () => {
			if (rfInstance === null) {
				return {
					nodes: [],
					edges: [],
					viewport: { x: 0, y: 0, zoom: 1 },
				};
			}

			return rfInstance.toObject();
		},

		flow: (
			<ReactFlow
				className={style.react_flow}
				nodes={nodes}
				edges={edges}
				onNodesChange={onNodesChange}
				onEdgesChange={onEdgesChange}
				onInit={setRfInstance}
				onConnect={onConnect}
				nodeTypes={nodeTypes}
				defaultEdgeOptions={{ type: "default" }}
				connectionMode={ConnectionMode.Strict}
				connectionLineType={ConnectionLineType.Bezier}
				onReconnect={onReconnect}
				onReconnectStart={onReconnectStart}
				onReconnectEnd={onReconnectEnd}
				colorMode="dark"
			>
				<Controls className={style.controls} />

				<Background
					variant={BackgroundVariant.Dots}
					gap={12}
					size={1}
					color="var(--mantine-color-dark-3)"
				/>

				<MiniMap
					nodeColor={(node) => {
						if ("color" in node.data && typeof node.data.color === "string") {
							return node.data.color;
						} else {
							return "var(--mantine-color-dark-2)";
						}
					}}
				/>
			</ReactFlow>
		),
	};
}
