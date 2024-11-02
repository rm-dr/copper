import { Fragment, ReactNode, useCallback, useState } from "react";
import { TreeEntry } from "./entry";
import styles from "./entry/tree_entry.module.scss";

export type TreeNode<D> = {
	// How to draw this node
	left: ReactNode;
	body: ReactNode;
	right?: ReactNode;

	selectable: boolean;

	// The unique id of this node
	uid: string;

	// The index of this node's parent.
	// If null, this node has no parent (and is at the root)
	parent: number | null;

	// If true, this node might have children.
	// If false, this node has no children. Don't draw
	// an "expand" arrow, and ignore all nodes that have
	// this node as a parent
	can_have_children: boolean;

	// Extra data for this node. Not used by the tree.
	data: D;
};

export function useTree<D>(params: { defaultOpen?: boolean }) {
	const [data, setData] = useState<TreeNode<D>[]>([]);
	const [opened, setOpened] = useState<Set<string>>(new Set([]));
	const [selected, setSelected] = useState<string | null>(null);

	const node = (
		<BuildTree
			data={data}
			parent={null}
			opened={opened}
			selected={selected}
			defaultOpen={params.defaultOpen === true}
			select_node={setSelected}
			set_opened={(uid, open) => {
				const post_open = params.defaultOpen === true ? !open : open;
				if (post_open) {
					setOpened((o) => {
						const s = new Set(o);
						s.add(uid);
						return s;
					});
				} else {
					setOpened((o) => {
						const s = new Set(o);
						s.delete(uid);
						return s;
					});
				}
			}}
		/>
	);

	const setTreeData = useCallback((new_data: TreeNode<D>[]) => {
		// Note that we don't de-select anything when data
		// changes. This is intentional.
		setOpened((o) => {
			// Auto-close nodes that were removed from the tree
			const s = new Set(o);
			for (const i in s.keys) {
				if (!new_data.some((x) => x.uid === i)) {
					s.delete(i);
				}
			}
			return s;
		});
		setData(new_data);
	}, []);

	return { node, data, setTreeData, selected };
}

function BuildTree<D>(params: {
	parent: number | null;
	data: TreeNode<D>[];
	defaultOpen: boolean;

	opened: Set<string>;
	selected: string | null;
	select_node: (uid: string | null) => void;
	set_opened: (uid: string, open: boolean) => void;
}) {
	return params.data
		.map((node, idx) => {
			if (node.parent !== params.parent) {
				return null;
			}

			const is_open = params.defaultOpen
				? !params.opened.has(node.uid)
				: params.opened.has(node.uid);
			const has_children =
				node.can_have_children && params.data.some((x) => x.parent === idx);

			let children = null;
			if (is_open && has_children) {
				children = (
					<div className={styles.tree_indent}>
						<BuildTree
							parent={idx}
							data={params.data}
							opened={params.opened}
							defaultOpen={params.defaultOpen}
							selected={params.selected}
							select_node={params.select_node}
							set_opened={params.set_opened}
						/>
					</div>
				);
			}

			return (
				<Fragment key={node.uid}>
					<TreeEntry
						left={node.left}
						body={node.body}
						right={node.right}
						is_selected={params.selected === node.uid}
						is_expanded={is_open}
						selectable={node.selectable}
						expandable={node.can_have_children}
						onExpandClick={() => {
							params.set_opened(node.uid, !is_open);
						}}
						onSelectClick={() => {
							if (params.selected === node.uid) {
								params.select_node(null);
							} else {
								params.set_opened(node.uid, true);
								params.select_node(node.uid);
							}
						}}
					/>
					{children}
				</Fragment>
			);
		})
		.filter((x) => x !== null);
}
