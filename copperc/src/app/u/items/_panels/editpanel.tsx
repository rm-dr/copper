"use client";

import TitleBar from "@/components/titlebar";
import stylesRoot from "../items.module.scss";
import stylesEdit from "./edit.module.scss";
import { components } from "@/lib/api/openapi";
import { X } from "lucide-react";
import { getAttrTypeInfo } from "@/lib/attributes";
import {
	Fragment,
	ReactNode,
	useCallback,
	useEffect,
	useMemo,
	useState,
} from "react";
import { Button, Text } from "@mantine/core";
import { Wrapper } from "@/components/spinner";

export function EditPanel(params: {
	class: components["schemas"]["ClassInfo"] | null;
	selectedItems: components["schemas"]["ItemlistItemInfo"][];
}) {
	/**
	 * List of attributes that have the same value across
	 * all selected items
	 */
	const sharedAttributes = useMemo(() => {
		return params.class === null
			? []
			: params.class.attributes.filter((x) => {
					let value:
						| undefined
						| null
						| components["schemas"]["ItemlistItemInfo"]["attribute_values"][string] =
						undefined;
					for (const i of params.selectedItems) {
						const v = i.attribute_values[x.id] || null;

						if (value === undefined) {
							value = v;
						} else if (value !== v) {
							return false;
						}
					}

					return true;
				});
	}, [params.class, params.selectedItems]);

	//
	// MARK: panelAttr
	//

	const [panelAttr, _setPanelAttr] = useState<
		null | components["schemas"]["AttributeInfo"]
	>(null);

	const setPanelAttr = useCallback(
		(attr_id: number | null | "auto") => {
			if (attr_id === null || params.class === null) {
				_setPanelAttr(null);
				return;
			}

			for (const attr of params.class.attributes) {
				const attrdef = getAttrTypeInfo(attr.data_type.type);

				if (
					// When changing class / dataset, select the first
					// panel-display attribute (if there is one)
					(attr_id === "auto" && attrdef.editor.type === "panel") ||
					// Or, if we're given an id, select it.
					attr.id === attr_id
				) {
					_setPanelAttr(attr);
					return;
				}
			}

			_setPanelAttr(null);
		},
		[params.class, _setPanelAttr],
	);

	useEffect(() => {
		// Auto-select attribute on load
		setPanelAttr("auto");
	}, [setPanelAttr]);

	//
	// MARK: edge cases
	//

	if (params.class === null) {
		return (
			<div className={stylesRoot.panel} style={{ height: "100%" }}>
				<TitleBar text="Edit items" />
				<div className={stylesRoot.panel_content}>
					<Wrapper>
						<X size="3rem" color="var(--mantine-color-dimmed)" />
						<Text size="1.3rem" c="dimmed">
							No class selected
						</Text>
					</Wrapper>
				</div>
			</div>
		);
	} else if (params.selectedItems.length === 0) {
		return (
			<div className={stylesRoot.panel} style={{ height: "100%" }}>
				<TitleBar text="Edit items" />
				<div className={stylesRoot.panel_content}>
					<Wrapper>
						<X size="3rem" color="var(--mantine-color-dimmed)" />
						<Text size="1.3rem" c="dimmed">
							No items selected
						</Text>
					</Wrapper>
				</div>
			</div>
		);
	}

	//
	// MARK: jsx
	//

	return (
		<div className={stylesRoot.panel} style={{ height: "100%" }}>
			<TitleBar text="Edit items" />
			<div className={stylesRoot.panel_content} style={{ height: "100%" }}>
				<div className={stylesEdit.edit_container}>
					<AttrList
						class={params.class}
						selectedItems={params.selectedItems}
						attr={panelAttr}
						sharedAttributes={sharedAttributes}
						setPanelAttr={setPanelAttr}
					/>

					<Panel
						class={params.class}
						selectedItems={params.selectedItems}
						attr={panelAttr}
						sharedAttributes={sharedAttributes}
					/>
				</div>
			</div>
		</div>
	);
}

//
// MARK: helpers
//

function AttrList(params: {
	class: components["schemas"]["ClassInfo"];
	selectedItems: components["schemas"]["ItemlistItemInfo"][];
	attr: components["schemas"]["AttributeInfo"] | null;
	sharedAttributes: components["schemas"]["AttributeInfo"][];
	setPanelAttr: (attr_id: number | null | "auto") => void;
}) {
	// Key here is important, it makes sure we get a new panel each time we select an item
	// (this makes sure that inputs update when we change our selection)
	return (
		<div
			key={`attrlist-${params.selectedItems.map((x) => x.id).join(",")}`}
			className={stylesEdit.attr_container}
		>
			{params.class.attributes.map((attr) => {
				const shared =
					params.sharedAttributes.find((x) => x.id === attr.id) !== undefined;

				const attrdef = getAttrTypeInfo(attr.data_type.type);

				if (shared) {
					const value =
						params.selectedItems[0]!.attribute_values[attr.id] || null;

					if (!(value === null || value.type === attr.data_type.type)) {
						throw new Error("Attribute type mismatch");
					}

					return (
						<div key={`attr-row-${attr.id}`} className={stylesEdit.attr_row}>
							<div className={stylesEdit.row_icon}>{attrdef.icon}</div>
							<div className={stylesEdit.row_name}>{attr.name}</div>
							<div className={stylesEdit.row_value_old}>
								{attrdef.editor.type === "panel" ? (
									params.attr?.id === attr.id ? (
										<Button fullWidth disabled>
											Viewing in panel
										</Button>
									) : (
										<Button
											fullWidth
											onClick={() => {
												params.setPanelAttr(attr.id);
											}}
										>
											View in panel
										</Button>
									)
								) : value === null ? (
									<div
										style={{
											paddingLeft: "0.5rem",
											width: "100%",
											overflow: "hidden",
											textOverflow: "ellipsis",
											whiteSpace: "nowrap",
											color: "var(--mantine-color-dimmed)",
											fontStyle: "italic",
										}}
									>
										Unset
									</div>
								) : (
									attrdef.editor.old_value(value)
								)}
							</div>
							<div className={stylesEdit.row_value_new}>
								{attrdef.editor.type === "panel" ? (
									params.attr?.id === attr.id ? (
										<Button fullWidth disabled>
											Viewing in panel
										</Button>
									) : (
										<Button
											fullWidth
											onClick={() => {
												params.setPanelAttr(attr.id);
											}}
										>
											View in panel
										</Button>
									)
								) : (
									attrdef.editor.new_value({
										value,
										onChange: console.log,
									})
								)}
							</div>
						</div>
					);
				} else {
					return (
						<div key={`attr-row-${attr.id}`} className={stylesEdit.attr_row}>
							<div className={stylesEdit.row_icon}>{attrdef.icon}</div>
							<div className={stylesEdit.row_name}>{attr.name}</div>
							<div className={stylesEdit.row_value_old}>
								{attrdef.editor.type === "panel" ? (
									<Button fullWidth disabled>
										Differs
									</Button>
								) : (
									<div
										style={{
											paddingLeft: "0.5rem",
											width: "100%",
											overflow: "hidden",
											textOverflow: "ellipsis",
											whiteSpace: "nowrap",
											color: "var(--mantine-color-dimmed)",
											fontStyle: "italic",
										}}
									>
										Differs
									</div>
								)}
							</div>
							<div className={stylesEdit.row_value_new}>
								{attrdef.editor.type === "panel" ? (
									<Button fullWidth disabled>
										Differs
									</Button>
								) : (
									<div
										style={{
											paddingLeft: "0.5rem",
											width: "100%",
											overflow: "hidden",
											textOverflow: "ellipsis",
											whiteSpace: "nowrap",
											color: "var(--mantine-color-dimmed)",
											fontStyle: "italic",
										}}
									>
										Differs
									</div>
								)}
							</div>
						</div>
					);
				}
			})}
		</div>
	);
}

function Panel(params: {
	class: components["schemas"]["ClassInfo"];
	selectedItems: components["schemas"]["ItemlistItemInfo"][];
	attr: components["schemas"]["AttributeInfo"] | null;
	sharedAttributes: components["schemas"]["AttributeInfo"][];
}) {
	if (params.sharedAttributes.length === 0) {
		throw new Error(
			"Entered unreachable code: tried to draw panel with nothing selected",
		);
	}

	let icon: ReactNode = <X />;
	let title: string = "";
	let body: ReactNode = "no attribute selected";

	if (params.attr !== null) {
		const attrdef = getAttrTypeInfo(params.attr.data_type.type);

		if (
			params.sharedAttributes.find((x) => x.id === params.attr!.id) !==
			undefined
		) {
			const value =
				params.selectedItems[0]?.attribute_values[params.attr.id] || null;

			if (value !== null) {
				// Show selected attribute value
				body = (
					attrdef.editor as typeof attrdef.editor & { type: "panel" }
				).panel_body({
					value,
					attr_id: params.attr.id,
					item_id: params.selectedItems[0]!.id,
				});
			} else {
				/// The selected attribute is unset
				body = "unset";
			}
		} else {
			// The selected attribute has different values
			// across selected items
			body = "differs";
		}

		icon = attrdef.icon;
		title = params.attr.name;
	}

	// Key here is important, it makes sure we get a new panel each time we select an item
	return (
		<Fragment
			key={`panel-${params.attr?.id}-${params.selectedItems.map((x) => x.id).join(",")}`}
		>
			<div className={stylesEdit.panel_container}>
				<div className={stylesEdit.panel_title}>
					<div className={stylesEdit.panel_title_icon}>{icon}</div>
					<div className={stylesEdit.panel_title_name}>{title}</div>
				</div>
				<div className={stylesEdit.panel_body}>{body}</div>
			</div>
		</Fragment>
	);
}
