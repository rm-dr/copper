import { attrTypeInfo } from "..";
import { ToggleRight } from "lucide-react";
import { BasicForm } from "../_basicform";
import { Select } from "@mantine/core";

export const _booleanAttrType: attrTypeInfo<"Boolean"> = {
	pretty_name: "Boolean",
	serialize_as: "Boolean",
	icon: <ToggleRight />,
	create_params: {
		form: (params) => BasicForm({ attr_type: { type: "Boolean" }, ...params }),
	},

	table_cell: ({ value }) => {
		return (
			<div
				style={{
					paddingLeft: "0.5rem",
					width: "100%",
					overflow: "hidden",
					textOverflow: "ellipsis",
					whiteSpace: "nowrap",
					color: "var(--mantine-color-white)",
				}}
			>
				{value.value}
			</div>
		);
	},

	editor: {
		type: "inline",

		old_value: (value) => {
			return (
				<div
					style={{
						paddingLeft: "0.5rem",
						width: "100%",
						overflow: "hidden",
						textOverflow: "ellipsis",
						whiteSpace: "nowrap",
						color: "var(--mantine-color-white)",
					}}
				>
					{value.value}
				</div>
			);
		},

		new_value: (params) => {
			return (
				<Select
					placeholder="unset boolean"
					data={["true", "false"]}
					value={
						params.value === null
							? undefined
							: params.value.value
								? "true"
								: "false"
					}
				/>
			);
		},
	},
};
