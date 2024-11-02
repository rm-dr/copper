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
	IsValidConnection,
	useReactFlow,
	getOutgoers,
} from "@xyflow/react";

import style from "./flow.module.scss";
import "@xyflow/react/dist/style.css";

// Custom node styles.
// This is not a css module so that classes are not renamed.
import "./flow.css";

import { nodeDefinitions, nodeTypes } from "./_nodes";
import { Overlay } from "@mantine/core";

export function useFlow(params: { disabled: boolean; onModify: () => void }) {
	const [nodes, setNodes] = useState<Node[]>([]);
	const [edges, setEdges] = useState<Edge[]>([]);
	const [rfInstance, setRfInstance] = useState<null | ReactFlowInstance>(null);
	const edgeReconnectSuccessful = useRef(true);
	const { getNodes, getEdges } = useReactFlow();

	const onReconnectStart = useCallback(
		(_: unknown, edge: Edge) => {
			params.onModify();
			edgeReconnectSuccessful.current = false;

			// delete the edge we moved so that `isValid` works like we expect
			// (if we don't do this, the pre-drag edge will exist in the flow)
			setEdges((eds) => eds.filter((e) => e.id !== edge.id));
		},
		[params],
	);

	const onReconnect: OnReconnect = useCallback(() => {
		params.onModify();
		edgeReconnectSuccessful.current = true;
	}, [params]);

	const onReconnectEnd = useCallback(
		(_: unknown, edge: Edge) => {
			params.onModify();
			if (edgeReconnectSuccessful.current) {
				setEdges((eds) => [...eds, edge]);
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
			const c_nodes = getNodes();
			const c_edges = getEdges();

			// Get source node details
			const source = c_nodes.find((node) => node.id === connection.source);
			if (source === undefined) return false;
			const sourcedef = nodeDefinitions[source.type!];
			if (sourcedef === undefined) return false;
			const sourceoutput = sourcedef
				.getOutputs(source.data)
				.find((x) => x.id === connection.sourceHandle);
			if (sourceoutput === undefined) return false;

			// Get target node details
			const target = c_nodes.find((node) => node.id === connection.target);
			if (target === undefined) return false;
			const targetdef = nodeDefinitions[target.type!];
			if (targetdef === undefined) return false;
			const targetinput = targetdef
				.getInputs(target.data)
				.find((x) => x.id === connection.targetHandle);
			if (targetinput === undefined) return false;

			// Make sure types match
			if (sourceoutput.type !== targetinput.type) {
				return false;
			}

			// Make sure target handle is not already connected
			const incoming_edges = c_edges.filter((x) => x.target === target.id);
			const repeat_input = incoming_edges.reduce(
				(prev, edge) => prev || edge.targetHandle === targetinput.id,
				false,
			);
			if (repeat_input) return false;

			// Do not allow cycles
			const hasCycle = (node: Node, visited: Set<string>) => {
				if (visited.has(node.id)) return true;
				visited.add(node.id);

				for (const out of getOutgoers(node, c_nodes, c_edges)) {
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
				// Disable interaction
				edgesReconnectable={!params.disabled}
				edgesFocusable={!params.disabled}
				nodesDraggable={!params.disabled}
				nodesConnectable={!params.disabled}
				nodesFocusable={!params.disabled}
				panOnDrag={!params.disabled}
				elementsSelectable={!params.disabled}
				zoomOnDoubleClick={!params.disabled}
				minZoom={params.disabled ? 1 : 0.5}
				maxZoom={params.disabled ? 1 : 3}
			>
				{
					/* Blur editor while disabled */
					!params.disabled ? null : (
						<Overlay color="#000000" backgroundOpacity={0.25} blur={5} />
					)
				}

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
