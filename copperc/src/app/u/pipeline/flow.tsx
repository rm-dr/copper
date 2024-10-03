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
	IsValidConnection,
	useReactFlow,
	getOutgoers,
} from "@xyflow/react";

import style from "./flow.module.scss";
import "@xyflow/react/dist/style.css";

import { nodeDefinitions, nodeTypes } from "./_nodes";

export function useFlow(params: { onModify: () => void }) {
	const [nodes, setNodes] = useState<Node[]>([]);
	const [edges, setEdges] = useState<Edge[]>([]);
	const [rfInstance, setRfInstance] = useState<null | ReactFlowInstance>(null);
	const edgeReconnectSuccessful = useRef(true);
	const { getNodes, getEdges } = useReactFlow();

	const onReconnectStart = useCallback(() => {
		params.onModify();
		edgeReconnectSuccessful.current = false;
	}, [params]);

	const onReconnect: OnReconnect = useCallback(
		(oldEdge, newConnection) => {
			params.onModify();
			edgeReconnectSuccessful.current = true;
			setEdges((els) => reconnectEdge(oldEdge, newConnection, els));
		},
		[params],
	);

	const onReconnectEnd = useCallback(
		(_: unknown, edge: Edge) => {
			params.onModify();
			if (!edgeReconnectSuccessful.current) {
				setEdges((eds) => eds.filter((e) => e.id !== edge.id));
			}

			edgeReconnectSuccessful.current = true;
		},
		[params],
	);

	const onNodesChange: OnNodesChange = useCallback(
		(changes) => {
			params.onModify();
			setNodes((nds) => applyNodeChanges(changes, nds));
		},
		[setNodes, params],
	);

	const onEdgesChange: OnEdgesChange = useCallback(
		(changes) => {
			params.onModify();
			setEdges((eds) => applyEdgeChanges(changes, eds));
		},
		[setEdges, params],
	);

	const onConnect: OnConnect = useCallback(
		(connection) => {
			params.onModify();
			setEdges((eds) => addEdge(connection, eds));
		},
		[setEdges, params],
	);

	const isValidConnection: IsValidConnection = useCallback(
		(connection) => {
			const nodes = getNodes();
			const edges = getEdges();

			const target = nodes.find((node) => node.id === connection.target);
			if (target === undefined) return false;

			const source = nodes.find((node) => node.id === connection.source);
			if (source === undefined) return false;

			// Do not allow cycles
			const hasCycle = (node: Node, visited: Set<string>) => {
				if (visited.has(node.id)) return true;
				visited.add(node.id);

				for (const out of getOutgoers(node, nodes, edges)) {
					if (out.id === source.id) return true;
					if (hasCycle(out, visited)) return true;
				}
			};
			if (hasCycle(target, new Set([source.id]))) {
				return false;
			}

			return true;
		},
		[getEdges, getNodes],
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
				onNodeDragStart={params.onModify}
				onNodeDrag={params.onModify}
				onNodeDragStop={params.onModify}
				onNodeClick={params.onModify}
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
				isValidConnection={isValidConnection}
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
						const nodedef = Object.entries(nodeDefinitions).find(
							(x) => x[1].xyflow_node_type === node.type,
						);

						if (
							nodedef === undefined ||
							nodedef[1].minimap_color === undefined
						) {
							return "var(--mantine-color-dark-2)";
						}

						return nodedef[1].minimap_color!;
					}}
				/>
			</ReactFlow>
		),
	};
}
